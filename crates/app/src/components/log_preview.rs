use crate::i18n::tr;
use crate::root::{RootView, Tab};
use gpui_kit::component::{
    button::*,
    group_box::{GroupBox, GroupBoxVariants},
    *,
};
use gpui_kit::*;

pub fn log_preview(lines: Vec<String>, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let body: Vec<AnyElement> = if lines.is_empty() {
        vec![
            div()
                .text_xs()
                .text_color(muted)
                .child(tr("Логи пока пусты. Запустите Zapret для начала работы.", "No events yet. Start Zapret to begin."))
                .into_any_element(),
        ]
    } else {
        lines
            .into_iter()
            .map(|l| {
                div()
                    .text_xs()
                    .font_family("Consolas")
                    .child(l)
                    .into_any_element()
            })
            .collect()
    };
    GroupBox::new()
        .outline()
        .title(tr("Последние события", "Recent events"))
        .gap_2()
        .child(div().v_flex().gap_1().children(body))
        .child(
            div().h_flex().justify_end().child(
                Button::new("go-logs")
                    .ghost()
                    .small()
                    .label(tr("Все логи →", "All logs →"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.tab = Tab::Logs;
                        cx.notify();
                    })),
            ),
        )
        .into_any_element()
}
