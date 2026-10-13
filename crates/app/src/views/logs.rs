use crate::components::page::page_with_header;
use crate::i18n::tr;
use crate::root::RootView;
use crate::root::view_data::ViewData;
use gpui_kit::component::{
    button::*,
    group_box::{GroupBox, GroupBoxVariants},
    label::Label,
    *,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

fn empty() -> String {
    tr("ui-no-events-yet-start-zapret-on-the")
}

pub fn render(data: &ViewData, cx: &mut Context<RootView>) -> AnyElement {
    let logs = data.logs.clone();
    let count = logs.len();

    page_with_header(header(count, cx), div().child(log_view(logs, cx)))
}

fn header(count: usize, cx: &mut Context<RootView>) -> Div {
    div()
        .h_flex()
        .justify_between()
        .items_center()
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(
                    Label::new(tr("ui-event-log"))
                        .text_lg()
                        .font_weight(FontWeight::BOLD),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(crate::i18n::tr_args(
                            "logs-count",
                            &[("count", (count).into())],
                        )),
                ),
        )
        .child(
            Button::new("clear-logs-btn")
                .outline()
                .small()
                .label(tr("ui-clear-log"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.on_clear_logs(cx);
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

    GroupBox::new()
        .outline()
        .w_full()
        .child(div().v_flex().w_full().gap_1().children(lines))
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
