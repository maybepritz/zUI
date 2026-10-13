//! Компоновка окна и выбор активного экрана.
use super::{RootView, Tab};
use crate::views;
use gpui_kit::{component::*, *};

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.state.settings.onboarded {
            return views::overlays::onboarding::render(
                &self.state,
                self.onboarding_step,
                &self.view_data,
                cx,
            );
        }
        if self.show_whats_new {
            return views::overlays::whats_new::render(cx);
        }
        let sidebar = self.sidebar(cx);

        let content = match self.tab {
            Tab::Home => views::home::render(&self.state, &self.view_data, cx),
            Tab::Strategies => {
                views::strategies::render(&self.state, &self.view_data, &self.strategy_filters, cx)
            }
            Tab::Dns => views::dns::render(&self.state, &self.view_data, cx),
            Tab::Telegram => views::telegram::render(&self.state, &self.view_data, cx),
            Tab::Statistics => views::statistics::render(&self.state, &self.view_data, cx),
            Tab::Logs => views::logs::render(&self.view_data, cx),
            Tab::Settings => views::settings::render(&self.state, cx),
        };

        div()
            .h_flex()
            .size_full()
            .child(sidebar)
            .child(div().flex_1().min_w_0().h_full().child(content))
            .into_any_element()
    }
}
