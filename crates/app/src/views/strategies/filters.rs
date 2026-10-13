//! Search and favorites controls; event handlers own all filter mutations.
use crate::{
    components::{rows::favorite_button::favorite_color, section::section},
    i18n::{tr, tr_args},
    root::{RootView, Tab, strategy_filters::StrategyFilters},
};
use gpui_kit::assets::IconName as LucideIcon;
use gpui_kit::component::Size as ComponentSize;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    component::{
        button::*,
        empty::{
            Empty as EmptyState, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia,
            EmptyMediaVariant, EmptyTitle,
        },
        input::Input,
        label::Label,
        *,
    },
    *,
};

pub(super) fn render(
    filters: &StrategyFilters,
    shown: usize,
    total: usize,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let favorite_ink = favorite_color(cx);
    let favorites = Toggle::new("strategies-favorites-only")
        .with_size(ComponentSize::Medium)
        .outline()
        .gap_2()
        .icon(if filters.favorites_only {
            LucideIcon::StarFill
        } else {
            LucideIcon::Star
        })
        .tooltip(tr("strategy-filter-favorites"))
        .checked(filters.favorites_only)
        .when(filters.favorites_only, |this| {
            this.bg(hsla(40.0 / 360.0, 0.85, 0.55, 0.12))
                .text_color(favorite_ink)
                .border_color(favorite_ink.opacity(0.35))
        })
        .on_click(
            cx.listener(|this, checked, _, cx| this.on_favorites_filter_changed(*checked, cx)),
        );

    section(tr("strategy-list-title"))
        .id("strategies-filters")
        .w_full()
        .child(
            div()
                .h_flex()
                .w_full()
                .items_center()
                .gap_2()
                .child(
                    div().flex_1().min_w_0().child(
                        Input::new(&filters.input)
                            .id("strategies-search")
                            .aria_label(tr("strategy-search-label"))
                            .with_size(ComponentSize::Medium)
                            .w_full()
                            .cleanable(true)
                            .prefix(Icon::new(LucideIcon::Search).size_4().text_color(muted)),
                    ),
                )
                .child(
                    div()
                        .h_flex()
                        .flex_shrink_0()
                        .items_center()
                        .gap_2()
                        .child(favorites)
                        .child(
                            Button::new("strategies-reset-filters")
                                .ghost()
                                .with_size(ComponentSize::Medium)
                                .icon(LucideIcon::RotateCcw)
                                .tooltip(tr("strategy-filter-reset"))
                                .disabled(!filters.is_active())
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.on_reset_strategy_filters(window, cx)
                                })),
                        ),
                ),
        )
        .child(
            Label::new(tr_args(
                "strategy-filter-count",
                &[("shown", shown.into()), ("total", total.into())],
            ))
            .text_xs()
            .text_color(muted),
        )
        .into_any_element()
}

pub(super) fn empty_state(
    filters: &StrategyFilters,
    total: usize,
    cx: &mut Context<RootView>,
) -> AnyElement {
    let (title, detail) = if total == 0 {
        ("strategy-list-empty", "strategy-list-empty-hint")
    } else if filters.favorites_only && filters.query.trim().is_empty() {
        ("strategy-favorites-empty", "strategy-favorites-empty-hint")
    } else {
        ("strategy-search-empty", "strategy-search-empty-hint")
    };
    let action = if total == 0 {
        Button::new("strategies-empty-home")
            .small()
            .outline()
            .label(tr("strategy-open-home"))
            .on_click(cx.listener(|this, _, _, cx| this.on_navigate(Tab::Home, cx)))
    } else {
        Button::new("strategies-empty-reset")
            .small()
            .outline()
            .label(tr("strategy-filter-reset"))
            .on_click(cx.listener(|this, _, window, cx| this.on_reset_strategy_filters(window, cx)))
    };

    EmptyState::new()
        .py_8()
        .header(
            EmptyHeader::new()
                .media(
                    EmptyMedia::new()
                        .with_variant(EmptyMediaVariant::Icon)
                        .child(Icon::new(LucideIcon::Search).size_4()),
                )
                .title(EmptyTitle::new().child(tr(title)))
                .description(EmptyDescription::new().child(tr(detail))),
        )
        .content(EmptyContent::new().child(action))
        .into_any_element()
}
