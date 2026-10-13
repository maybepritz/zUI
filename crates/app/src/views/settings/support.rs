//E:\zui\crates\app\src\views\settings\support.rs
use crate::{
    components::hint::hint,
    components::section::{button_row, section},
    i18n::tr,
    root::RootView,
    state::AppState,
};
use gpui_kit::{
    component::{button::*, label::Label, *},
    *,
};

pub fn render(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;

    let about = section(tr("ui-about"))
        .child(Label::new(crate::i18n::tr_args(
            "version-label",
            &[(
                "version",
                format!(
                    "{} {}",
                    state.update.current_version(),
                    crate::release::codename()
                )
                .into(),
            )],
        )))
        .child(
            button_row()
                .child(
                    Button::new("check-update")
                        .outline()
                        .icon(IconName::Info)
                        .label(tr("ui-check-for-updates"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_check_update(cx);
                        })),
                )
                .child(
                    Button::new("show-whats-new")
                        .outline()
                        .label(tr("ui-what-s-new"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_show_whats_new(cx);
                        })),
                ),
        )
        .child(
            Button::new("open-runtime")
                .outline()
                .icon(IconName::Folder)
                .label(tr("ui-open-data-folder"))
                .on_click(cx.listener(|this, _, _, cx| this.on_open_runtime(cx))),
        );

    let report = section(tr("ui-report-a-problem"))
        .child(hint(
            Label::new(tr("ui-the-report-includes-the-version-checks-statistics")),
            muted,
        ))
        .child(
            button_row()
                .child(
                    Button::new("export-report")
                        .primary()
                        .label(tr("ui-save-report"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_export_report(cx);
                        })),
                )
                .child(
                    Button::new("copy-report")
                        .outline()
                        .label(tr("ui-copy-for-github"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.on_copy_report(cx);
                        })),
                )
                .child(
                    Button::new("open-issue")
                        .outline()
                        .label("GitHub issue")
                        .on_click(cx.listener(|this, _, _, cx| this.on_open_issue(cx))),
                ),
        );
    div()
        .v_flex()
        .gap_4()
        .child(about)
        .child(report)
        .into_any_element()
}
