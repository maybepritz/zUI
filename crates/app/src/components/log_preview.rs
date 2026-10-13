use crate::i18n::tr;
use crate::root::RootView;
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
                .child(tr("ui-no-events-yet-start-zapret-to-begin"))
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
        .title(tr("ui-recent-events"))
        .gap_2()
        .child(div().v_flex().gap_1().children(body))
        .child(
            div().h_flex().justify_end().child(
                Button::new("go-logs")
                    .ghost()
                    .small()
                    .label(tr("ui-all-logs"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.on_show_logs(cx);
                    })),
            ),
        )
        .into_any_element()
}
