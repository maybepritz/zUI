use crate::{components::section::button_row, i18n::tr, root::RootView, state::AppState};
use gpui_kit::{component::{button::*, label::Label, scroll::ScrollableElement as _, *}, *};

pub fn render(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let hosts = &state.hosts;
    let blocked = hosts.busy() || !z_core::dns::is_elevated();
    div().v_flex().size_full().gap_4().overflow_y_scrollbar()
        .child(Label::new("Hosts").text_lg())
        .child(Label::new(tr(
            "Откройте шаблон и вставьте записи своего доверенного источника. Заглушки не применяются. Перед изменением создаётся резервная копия; чужие изменения при откате не перезаписываются.",
            "Open the template and paste records from your trusted source. Placeholders cannot be applied. A backup is created before changes; rollback does not overwrite external edits."
        )).text_sm())
        .child(Label::new(tr(
            "Hosts меняет разрешение доменов и не гарантирует обход DPI или геоблокировки. Используйте только доверенные адреса.",
            "Hosts changes name resolution and does not guarantee bypassing DPI or geographic restrictions. Use trusted addresses only."
        )).text_sm())
        .child(button_row()
            .child(Button::new("hosts-edit").outline()
                .label(tr("Открыть список", "Open rules"))
                .on_click(cx.listener(|this, _, _, cx| {
                    if this.state.hosts.prepare_rules() {
                        cx.open_with_system(&this.state.hosts.rules_path);
                    } cx.notify();
                })))
            .child(Button::new("hosts-apply").primary()
                .label(tr("Применить список", "Apply rules"))
                .disabled(blocked)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.hosts.run(false);
                    cx.notify();
                })))
            .child(Button::new("hosts-restore").outline()
                .label(tr("Откатить изменения", "Restore backup"))
                .disabled(blocked || !hosts.has_backup())
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.hosts.run(true);
                    cx.notify();
                }))))
        .child(Label::new(hosts.status.clone()))
        .child(super::onboarding::permissions(cx))
        .into_any_element()
}
