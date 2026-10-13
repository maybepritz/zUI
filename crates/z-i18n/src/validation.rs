use crate::{Catalog, FluentArgs};
use fluent_syntax::ast::{self, Expression, InlineExpression, PatternElement};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Node {
    variables: BTreeSet<String>,
    references: Vec<(String, BTreeSet<String>)>,
    errors: Vec<String>,
}

fn pattern(pattern: &ast::Pattern<&str>, node: &mut Node) {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            expression_vars(expression, node);
        }
    }
}

fn expression_vars(expression: &Expression<&str>, node: &mut Node) {
    match expression {
        Expression::Inline(value) => inline(value, node),
        Expression::Select { selector, variants } => {
            inline(selector, node);
            for variant in variants {
                pattern(&variant.value, node);
            }
        }
    }
}

fn arguments(args: &ast::CallArguments<&str>, node: &mut Node) {
    for value in &args.positional {
        inline(value, node);
    }
    for value in &args.named {
        inline(&value.value, node);
    }
}

fn reference(id: &str, attribute: &Option<ast::Identifier<&str>>, term: bool) -> String {
    let prefix = if term { "-" } else { "" };
    match attribute {
        Some(attribute) => format!("{prefix}{id}.{}", attribute.name),
        None => format!("{prefix}{id}"),
    }
}

fn inline(value: &InlineExpression<&str>, node: &mut Node) {
    match value {
        InlineExpression::VariableReference { id } => {
            node.variables.insert(id.name.into());
        }
        InlineExpression::Placeable { expression } => expression_vars(expression, node),
        InlineExpression::FunctionReference {
            id,
            arguments: args,
        } => {
            arguments(args, node);
            if id.name != "NUMBER" {
                node.errors
                    .push(format!("unsupported function {}", id.name));
            }
        }
        InlineExpression::MessageReference { id, attribute } => {
            node.references
                .push((reference(id.name, attribute, false), BTreeSet::new()));
        }
        InlineExpression::TermReference {
            id,
            attribute,
            arguments: args,
        } => {
            let mut bound = BTreeSet::new();
            if let Some(args) = args {
                arguments(args, node);
                bound.extend(args.named.iter().map(|arg| arg.name.name.to_owned()));
            }
            node.references
                .push((reference(id.name, attribute, true), bound));
        }
        _ => {}
    }
}

fn resolve(
    key: &str,
    nodes: &BTreeMap<String, Node>,
    visiting: &mut BTreeSet<String>,
) -> Result<BTreeSet<String>, String> {
    if !visiting.insert(key.into()) {
        return Err(format!("{key}: circular reference"));
    }
    let node = nodes
        .get(key)
        .ok_or_else(|| format!("{key}: unknown reference"))?;
    if !node.errors.is_empty() {
        return Err(format!("{key}: {}", node.errors.join(", ")));
    }
    let mut variables = node.variables.clone();
    for (reference, bound) in &node.references {
        variables.extend(
            resolve(reference, nodes, visiting)?
                .difference(bound)
                .cloned(),
        );
    }
    visiting.remove(key);
    Ok(variables)
}

fn schema(text: &str) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let resource = fluent_syntax::parser::parse(text).map_err(|(_, e)| format!("{e:?}"))?;
    let mut nodes = BTreeMap::new();
    let mut messages = BTreeSet::new();
    for entry in &resource.body {
        let (key, value, attributes) = match entry {
            ast::Entry::Message(message) => {
                messages.insert(message.id.name.to_owned());
                (
                    message.id.name.to_owned(),
                    message.value.as_ref(),
                    &message.attributes,
                )
            }
            ast::Entry::Term(term) => (
                format!("-{}", term.id.name),
                Some(&term.value),
                &term.attributes,
            ),
            _ => continue,
        };
        let value = value.ok_or_else(|| format!("{key}: missing value"))?;
        let mut node = Node::default();
        pattern(value, &mut node);
        if nodes.insert(key.clone(), node).is_some() {
            return Err(format!("{key}: duplicate message"));
        }
        for attribute in attributes {
            let mut node = Node::default();
            pattern(&attribute.value, &mut node);
            nodes.insert(format!("{key}.{}", attribute.id.name), node);
        }
    }
    messages
        .into_iter()
        .map(|key| {
            let variables = resolve(&key, &nodes, &mut BTreeSet::new())?;
            Ok((key, variables))
        })
        .collect()
}

/// Check key parity, variables and actual resolution, including references/selectors.
/// Partial runtime packs are allowed by Localizer, but contribution validation is strict.
pub fn validate(reference: &Catalog, translation: &Catalog) -> Result<(), Vec<String>> {
    let reference_schema = schema(&reference.resource).map_err(|e| vec![e])?;
    let translated = schema(&translation.resource).map_err(|e| vec![e])?;
    let mut errors = Vec::new();
    for (key, variables) in &reference_schema {
        match translated.get(key) {
            None => errors.push(format!("{key}: missing message")),
            Some(actual) if variables != actual => errors.push(format!(
                "{key}: expected variables {variables:?}, got {actual:?}"
            )),
            _ => {}
        }
        for sample in [0, 1, 2, 5, 21] {
            let mut args = FluentArgs::new();
            for variable in variables {
                args.set(variable.as_str(), sample);
            }
            if translated.contains_key(key) && translation.format(key, Some(&args)).is_none() {
                errors.push(format!("{key}: unresolved reference, variable or function"));
                break;
            }
        }
    }
    for key in translated
        .keys()
        .filter(|key| !reference_schema.contains_key(*key))
    {
        errors.push(format!("{key}: unknown message"));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
