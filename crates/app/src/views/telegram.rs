use crate::i18n::tr;
use crate::components::section::{button_row, section};
use crate::{root::RootView, state::AppState};
use gpui_kit::component::{button::*, label::Label, scroll::ScrollableElement as _, *};
use gpui_kit::*;

fn intro() -> &'static str {
    tr("Официальный прокси Flowseal. Работает отдельно от Zapret; после запуска подключите его в Telegram Desktop.", "Flowseal proxy. Runs separately from Zapret; connect it in Telegram Desktop after starting.")
}
fn port_hint() -> &'static str {
    tr("Статус порта подтверждает запуск локального прокси. Доступность Telegram проверяется в самом Telegram.", "The port status confirms that the local proxy is running. Check Telegram connectivity in Telegram itself.")
}
fn media_hint() -> &'static str {
    tr("Остановите прокси, попробуйте только DC 4 и запустите снова. Если не помогло — очистите DC → IP.", "Stop the proxy, try DC 4 only, then start it again. If that does not help, clear DC → IP.")
}
fn msg_opened() -> &'static str {
    tr("Подтвердите подключение в Telegram. Если окно не открылось, скопируйте ссылку.", "Confirm the connection in Telegram. If no window appears, copy the link.")
}
fn msg_copied() -> &'static str {
    tr("Ссылка скопирована. Откройте её в «Избранном» Telegram.", "Link copied. Open it in Telegram Saved Messages.")
}

pub fn render(state: &mut AppState, cx: &mut Context<RootView>) -> AnyElement {
    let running = state.telegram.running();
    let muted = cx.theme().muted_foreground;

    div()
        .v_flex()
        .size_full()
        .gap_4()
        .overflow_y_scrollbar()
        .child(
            Label::new("Telegram · TG WS Proxy")
                .text_lg()
                .font_weight(FontWeight::BOLD),
        )
        .child(Label::new(intro()).text_sm().text_color(muted))
        .child(connection_section(state, running, muted, cx))
        .child(media_section(state, running, cx))
        .child(files_section(state, running, muted, cx))
        .into_any_element()
}

fn connection_section(
    state: &AppState,
    running: bool,
    muted: Hsla,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let tg = &state.telegram;
    let installed = tg.proxy.installed();
    let busy = tg.installing();
    let ready = tg.proxy.ready;

    let install_label = if busy {
        tr("Загрузка…", "Downloading…")
    } else if installed {
        tr("Установлен", "Installed")
    } else {
        tr("Установить с GitHub", "Install from GitHub")
    };

    section(tr("Подключение", "Connection"))
        .child(Label::new(tg.proxy.status.clone()).text_sm())
        .child(Label::new(tg.message.clone()).text_sm())
        .child(
            button_row()
                .child(
                    Button::new("tg-install")
                        .outline()
                        .label(install_label)
                        .disabled(busy || installed)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.telegram.install();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("tg-toggle")
                        .primary()
                        .label(if running {
                            tr("Остановить", "Stop")
                        } else {
                            tr("Запустить", "Start")
                        })
                        .disabled(busy || !installed)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.telegram.toggle();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("tg-connect")
                        .outline()
                        .label(tr("Подключить Telegram", "Connect Telegram"))
                        .disabled(!ready)
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(link) = this.state.telegram.link() {
                                cx.open_url(&link);
                                this.state.telegram.message = msg_opened().into();
                            }
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("tg-copy")
                        .ghost()
                        .label(tr("Копировать ссылку", "Copy link"))
                        .disabled(!ready)
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(link) = this.state.telegram.link() {
                                cx.write_to_clipboard(ClipboardItem::new_string(link));
                                this.state.telegram.message = msg_copied().into();
                            }
                            cx.notify();
                        })),
                ),
        )
        .child(Label::new(port_hint()).text_xs().text_color(muted))
}

fn media_section(state: &AppState, running: bool, cx: &mut Context<RootView>) -> impl IntoElement {
    let tg = &state.telegram;
    let blocked = running || tg.installing() || !tg.proxy.config_path().is_file();

    section(tr("Если фото и видео не загружаются", "If photos or videos fail to load"))
        .child(Label::new(media_hint()).text_sm())
        .child(
            button_row()
                .child(
                    Button::new("tg-media")
                        .outline()
                        .label(tr("Оставить только DC 4", "Use DC 4 only"))
                        .disabled(blocked)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.telegram.fix_media(false);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("tg-media-clear")
                        .outline()
                        .label(tr("Очистить DC → IP", "Clear DC → IP"))
                        .disabled(blocked)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.telegram.fix_media(true);
                            cx.notify();
                        })),
                ),
        )
}

fn files_section(
    state: &AppState,
    running: bool,
    muted: Hsla,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let tg = &state.telegram;
    let has_log = tg.proxy.log_path().is_file();
    let has_config = tg.proxy.config_path().is_file();

    section(tr("Файлы и диагностика", "Files and diagnostics"))
        .child(
            Label::new(format!("Каталог: {}", tg.proxy.dir.display()))
                .text_xs()
                .text_color(muted),
        )
        .child(
            button_row()
                .child(
                    Button::new("tg-log")
                        .outline()
                        .label(tr("Открыть журнал", "Open log"))
                        .disabled(!has_log)
                        .on_click(cx.listener(|this, _, _, cx| {
                            cx.open_with_system(&this.state.telegram.proxy.log_path());
                        })),
                )
                .child(
                    Button::new("tg-config")
                        .outline()
                        .label(tr("Открыть конфигурацию", "Open configuration"))
                        .disabled(running || !has_config)
                        .on_click(cx.listener(|this, _, _, cx| {
                            cx.open_with_system(&this.state.telegram.proxy.config_path());
                        })),
                )
                .child(
                    Button::new("tg-releases")
                        .ghost()
                        .label(tr("Релизы Flowseal", "Flowseal releases"))
                        .on_click(|_, _, cx| {
                            cx.open_url(z_core::telegram::RELEASES_URL);
                        }),
                ),
        )
}
