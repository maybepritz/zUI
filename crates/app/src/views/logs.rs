use crate::i18n::tr;
use crate::{root::RootView, state::AppState};
use gpui_kit::component::{
    button::*,
    group_box::{GroupBox, GroupBoxVariants},
    label::Label,
    scroll::ScrollableElement as _,
    *,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

fn empty() -> &'static str {
    tr("Лог пуст. Запустите службу Zapret на вкладке «Главная» для просмотра событий.", "No events yet. Start Zapret on the Home page to view events.")
}

pub fn render(state: &mut AppState, cx: &mut Context<RootView>) -> AnyElement {
    let logs = state.log.get();
    let count = logs.len();

    div()
        .v_flex()
        .size_full()
        .gap_4()
        .child(header(count, cx))
        .child(log_view(logs, cx))
        .into_any_element()
}

fn header(count: usize, cx: &mut Context<RootView>) -> impl IntoElement {
    div()
        .h_flex()
        .justify_between()
        .items_center()
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(
                    Label::new(tr("Журнал событий", "Event log"))
                        .text_lg()
                        .font_weight(FontWeight::BOLD),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("Всего записей: {count}")),
                ),
        )
        .child(
            Button::new("clear-logs-btn")
                .outline()
                .small()
                .label(tr("Очистить журнал", "Clear log"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.log.clear();
                    cx.notify();
                })),
        )
}

fn log_view(logs: Vec<String>, cx: &mut Context<RootView>) -> impl IntoElement {
    let lines: Vec<AnyElement> = if logs.is_empty() {
        vec![
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(empty())
                .into_any_element(),
        ]
    } else {
        logs.into_iter().map(log_line).collect()
    };

    GroupBox::new().outline().flex_1().w_full().child(
        div()
            .v_flex()
            .size_full()
            .gap_1()
            .overflow_y_scrollbar()
            .children(lines),
    )
}

fn log_line(line: String) -> AnyElement {
    let is_err = is_error(&line);
    div()
        .text_xs()
        .font_family("Consolas")
        .when(is_err, |this| this.text_color(hsla(0.0, 0.70, 0.50, 1.0)))
        .child(line)
        .into_any_element()
}

fn is_error(line: &str) -> bool {
    line.contains("[err]") || line.contains("Ошибка") || line.contains("Error")
}
