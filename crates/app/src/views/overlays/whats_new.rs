//! Обзор выпуска: простые карточки и прокрутка на всю ширину.
use crate::{i18n::tr, release, root::RootView};
use gpui_kit::assets::IconName as LucideIcon;
use gpui_kit::{
    component::{
        button::*,
        group_box::{GroupBox, GroupBoxVariants},
        label::Label,
        scroll::ScrollableElement as _,
        *,
    },
    *,
};

struct NotesSection {
    title: String,
    items: Vec<String>,
}

/// Переводчики сохраняют привычный формат «заголовок и пункты» в одном сообщении Fluent.
fn sections(text: &str) -> Vec<NotesSection> {
    let mut sections: Vec<NotesSection> = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(title) = line.strip_prefix("## ") {
            sections.push(NotesSection {
                title: title.to_owned(),
                items: Vec::new(),
            });
        } else {
            if sections.is_empty() {
                sections.push(NotesSection {
                    title: tr("ui-what-s-new"),
                    items: Vec::new(),
                });
            }
            if let Some(section) = sections.last_mut() {
                section
                    .items
                    .push(line.strip_prefix("- ").unwrap_or(line).to_owned());
            }
        }
    }
    sections
}

fn change_card(section: NotesSection, icon: LucideIcon, cx: &App) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    GroupBox::new()
        .outline()
        .w(px(320.))
        .flex_1()
        .min_w(px(280.))
        .child(
            div()
                .v_flex()
                .gap_2()
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(Icon::new(icon).size_4().text_color(muted))
                        .child(
                            Label::new(section.title)
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM),
                        ),
                )
                .children(
                    section
                        .items
                        .into_iter()
                        .map(|item| Label::new(item).text_sm().text_color(muted)),
                ),
        )
        .into_any_element()
}

pub fn render(cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let icons = [
        LucideIcon::Star,
        LucideIcon::Power,
        LucideIcon::Globe,
        LucideIcon::MousePointer2,
    ];
    let cards: Vec<_> = sections(&tr("whats-new-notes"))
        .into_iter()
        .enumerate()
        .map(|(ix, section)| change_card(section, icons[ix % icons.len()], cx))
        .collect();

    // Простой заголовок: экран держит название выпуска, а не цветная плашка.
    let hero = div()
        .v_flex()
        .w_full()
        .gap_2()
        .child(
            Label::new(format!("{} · v{}", tr("ui-what-s-new"), release::VERSION))
                .text_sm()
                .text_color(muted),
        )
        .child(
            Label::new(format!("zUI {}", release::codename()))
                .text_3xl()
                .font_weight(FontWeight::MEDIUM),
        )
        .child(
            Label::new(tr("whats-new-summary"))
                .text_sm()
                .text_color(muted),
        );

    let content = div()
        .v_flex()
        .w_full()
        .max_w(px(720.))
        .mx_auto()
        .gap_4()
        .child(hero)
        .child(div().h_flex().w_full().flex_wrap().gap_4().children(cards));

    let footer = div()
        .w_full()
        .flex_shrink_0()
        .border_t_1()
        .border_color(cx.theme().border)
        .p_4()
        .child(
            div()
                .h_flex()
                .w_full()
                .max_w(px(720.))
                .mx_auto()
                .items_center()
                .gap_4()
                .child(
                    div().flex_1().min_w_0().child(
                        Label::new(tr("whats-new-footer"))
                            .text_xs()
                            .text_color(muted),
                    ),
                )
                .child(
                    Button::new("dismiss-whats-new")
                        .primary()
                        .icon(LucideIcon::ArrowRight)
                        .label(tr("ui-continue"))
                        .on_click(cx.listener(|this, _, _, cx| this.on_dismiss_whats_new(cx))),
                ),
        );

    div()
        .v_flex()
        .size_full()
        .child(
            div()
                .flex_1()
                .min_h_0()
                .w_full()
                .overflow_y_scrollbar()
                .id("whats-new-scroll")
                .child(div().w_full().p_6().child(content)),
        )
        .child(footer)
        .into_any_element()
}
