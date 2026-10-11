use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem,
};
use gpui_kit::component::*;
use gpui_kit::*;

use crate::i18n::{self, tr};
use crate::{state::AppState, tray::Tray, views};
use z_core::preferences::ThemePreference;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Strategies,
    Dns,
    Telegram,
    // Hosts,
    Statistics,
    Logs,
    Settings,
}

impl Tab {
    const ALL: [Tab; 7] = [
        Tab::Home,
        Tab::Strategies,
        Tab::Dns,
        Tab::Telegram,
        // Tab::Hosts,
        Tab::Statistics,
        Tab::Logs,
        Tab::Settings,
    ];

    fn title(self) -> &'static str {
        match self {
            Tab::Home => tr("Главная", "Home"),
            Tab::Strategies => tr("Стратегии", "Strategies"),
            Tab::Dns => "DNS",
            Tab::Telegram => "Telegram",
            // Tab::Hosts => "Hosts",
            Tab::Statistics => tr("Статистика", "Statistics"),
            Tab::Logs => tr("Логи", "Logs"),
            Tab::Settings => tr("Настройки", "Settings"),
        }
    }
}

pub struct RootView {
    pub state: AppState,
    pub tab: Tab,
    tray: Option<Tray>,
    pub onboarding_step: views::onboarding::Step,
    pub show_whats_new: bool,
}

impl RootView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut state = AppState::new();
        i18n::set_language(state.settings.language);
        let show_whats_new = state.settings.onboarded
            && state.settings.last_seen_version != env!("CARGO_PKG_VERSION");
        if state.settings.onboarded {
            state.start_selected_services();
        }
        let tray = match Tray::new(window) {
            Ok(tray) => Some(tray),
            Err(error) => {
                let message = format!(
                    "Трей недоступен: {error}. Для выхода используйте кнопку в настройках."
                );
                state.log.add(&message);
                state.settings_status = message.into();
                None
            }
        };
        let view = cx.entity().downgrade();
        window.on_window_should_close(cx, move |_, cx| {
            view.update(cx, Self::on_close_requested).is_err()
        });
        Theme::change(
            if Self::dark_theme(state.settings.theme) {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            },
            None,
            cx,
        );
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(1))
                    .await;
                if this
                    .update(cx, |this, cx| {
                        this.state.background.hidden = this.tray.as_ref().is_some_and(Tray::hidden);
                        let dark = Self::dark_theme(this.state.settings.theme);
                        if dark != cx.theme().mode.is_dark() {
                            let mode = if dark {
                                ThemeMode::Dark
                            } else {
                                ThemeMode::Light
                            };
                            Theme::change(mode, None, cx);
                        }
                        if this.tray.as_ref().is_some_and(Tray::take_quit_request) {
                            cx.defer(|cx| cx.quit());
                            return;
                        }
                        this.state.poll();
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        cx.on_app_quit(|this, _| {
            this.tray.take();
            this.state.shutdown();
            async {}
        })
        .detach();
        Self {
            state,
            tab: Tab::Home,
            tray,
            onboarding_step: views::onboarding::Step::default(),
            show_whats_new,
        }
    }

    fn dark_theme(preference: ThemePreference) -> bool {
        match preference {
            ThemePreference::System => crate::platform::system_dark(),
            ThemePreference::Dark => true,
            ThemePreference::Light => false,
        }
    }

    fn on_close_requested(&mut self, cx: &mut Context<Self>) {
        if !self.state.settings.close_to_tray {
            // Keep the root alive until on_app_quit has stopped owned processes.
            cx.defer(|cx| cx.quit());
            return;
        }

        if !self.tray.as_ref().is_some_and(Tray::hide) {
            self.state.settings_status = "Не удалось свернуть в трей. Для выхода выключите \
                сворачивание или нажмите «Выйти из приложения»."
                .into();
            self.tab = Tab::Settings;
            cx.notify();
        }
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut menu = SidebarMenu::new();

        for tab in Tab::ALL {
            menu = menu.child(
                SidebarMenuItem::new(tab.title())
                    .active(self.tab == tab)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.tab = tab;
                        cx.notify();
                    })),
            );
        }

        Sidebar::new("sidebar")
            .header(
                SidebarHeader::new().child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(img("icon.svg").size_10().rounded_sm())
                        .child(div().text_base().font_weight(FontWeight::BOLD).child("zUI")),
                ),
            )
            .child(SidebarGroup::new("").child(menu))
            .footer(SidebarFooter::new().child(env!("CARGO_PKG_VERSION")))
            .into_any_element()
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.state.settings.onboarded {
            return views::onboarding::render(self, cx);
        }
        if self.show_whats_new {
            return views::whats_new::render(cx);
        }
        let sidebar = self.sidebar(cx);

        let content = match self.tab {
            Tab::Home => views::home::render(&mut self.state, cx),
            Tab::Strategies => views::strategies::render(&mut self.state, cx),
            Tab::Dns => views::dns::render(&mut self.state, cx),
            Tab::Telegram => views::telegram::render(&mut self.state, cx),
            // Tab::Hosts => views::hosts::render(&self.state, cx),
            Tab::Statistics => views::statistics::render(&self.state, cx),
            Tab::Logs => views::logs::render(&mut self.state, cx),
            Tab::Settings => views::settings::render(&self.state, cx),
        };

        div()
            .h_flex()
            .size_full()
            .child(sidebar)
            .child(div().flex_1().min_w_0().h_full().p_6().child(content))
            .into_any_element()
    }
}
