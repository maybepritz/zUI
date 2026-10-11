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

    let about = section(tr("О приложении", "About"))
        .child(Label::new(format!(
            "{}: {}",
            tr("Версия", "Version"),
            state.update.current_version()
        )))
        .child(
            button_row()
                .child(
                    Button::new("check-update")
                        .outline()
                        .icon(IconName::Info)
                        .label(tr("Проверить обновления", "Check for updates"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.update.check();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("show-whats-new")
                        .outline()
                        .label(tr("Что нового", "What's new"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_whats_new = true;
                            cx.notify();
                        })),
                ),
        )
        .child(
            Button::new("open-runtime")
                .outline()
                .icon(IconName::Folder)
                .label(tr("Открыть папку данных", "Open data folder"))
                .on_click(cx.listener(|this, _, _, cx| cx.reveal_path(&this.state.data_dir()))),
        );

    let report = section(tr("Сообщить о проблеме", "Report a problem"))
        .child(
            hint(Label::new(tr(
            "Отчёт содержит версию, результаты проверок, статистику и последние 250 строк журнала. Секреты прокси не включаются. Проверьте текст перед публикацией.",
            "The report includes the version, checks, statistics and the last 250 log entries. Proxy secrets are excluded. Review the text before publishing."
        )), muted))
        .child(button_row()
            .child(Button::new("export-report").primary()
                .label(tr("Сохранить отчёт", "Save report"))
                .on_click(cx.listener(|this, _, _, cx| {
                    match this.state.export_report() {
                        Some(path) => {
                            cx.open_with_system(&path);
                            this.state.settings_status = tr("Отчёт сохранён", "Report saved").into();
                        }
                        None => {
                            this.state.settings_status =
                                tr("Не удалось сохранить отчёт", "Could not save the report").into();
                        }
                    }
                    cx.notify();
                })))
            .child(Button::new("copy-report").outline()
                .label(tr("Копировать для GitHub", "Copy for GitHub"))
                .on_click(cx.listener(|this, _, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(this.state.issue_report()));
                    this.state.settings_status =
                        tr("Текст скопирован. Проверьте его перед отправкой", "Copied. Review before submitting").into();
                    cx.notify();
                })))
            .child(Button::new("open-issue").outline()
                .label("GitHub issue")
                .on_click(|_, _, cx| cx.open_url("https://github.com/maybepritz/zui/issues/new"))));
    div()
        .v_flex()
        .gap_4()
        .child(about)
        .child(report)
        .into_any_element()
}
