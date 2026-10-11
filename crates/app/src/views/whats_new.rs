use crate::{i18n::tr, root::RootView};
use gpui_kit::{
    component::{button::*, label::Label, scroll::ScrollableElement as _, *},
    *,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn notes(text: &str, muted: Hsla) -> Div {
    let mut list = div().v_flex().gap_2().w_full();
    for line in text.lines() {
        let line = line.trim_end();
        list = if line.is_empty() {
            list.child(div().h_2())
        } else if let Some(title) = line.strip_prefix("## ") {
            list.child(Label::new(title.to_string()).font_weight(FontWeight::BOLD))
        } else if let Some(item) = line.strip_prefix("- ") {
            list.child(
                div()
                    .h_flex()
                    .items_start()
                    .gap_2()
                    .child(Label::new("•").text_color(muted))
                    .child(div().flex_1().child(Label::new(item.to_string()))),
            )
        } else {
            list.child(Label::new(line.to_string()))
        };
    }
    list
}

pub fn render(cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let text = tr(
        include_str!("../../assets/whats-new.ru.txt"),
        include_str!("../../assets/whats-new.en.txt"),
    );

    let header = div()
        .v_flex()
        .gap_1()
        .flex_shrink_0()
        .child(
            Label::new(tr("Что нового", "What's new"))
                .text_lg()
                .font_weight(FontWeight::BOLD),
        )
        .child(
            Label::new(format!("zUI {VERSION}"))
                .text_sm()
                .text_color(muted),
        );

    let body = div()
        .flex_1()
        .min_h_0()
        .w_full()
        .overflow_y_scrollbar()
        .child(
            div()
                .p_4()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .child(notes(text, muted)),
        );

    let footer = div().h_flex().justify_end().flex_shrink_0().child(
        Button::new("dismiss-whats-new")
            .primary()
            .label(tr("Продолжить", "Continue"))
            .on_click(cx.listener(|this, _, _, cx| {
                this.state.settings.last_seen_version = VERSION.into();
                this.show_whats_new = !this.state.save_settings();
                cx.notify();
            })),
    );

    div()
        .size_full()
        .p_6()
        .h_flex()
        .justify_center()
        .child(
            div()
                .v_flex()
                .size_full()
                .max_w(px(720.))
                .gap_4()
                .child(header)
                .child(body)
                .child(footer),
        )
        .into_any_element()
}
