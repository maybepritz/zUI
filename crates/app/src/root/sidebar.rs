//! Разметка боковой навигации.

use super::{RootView, Tab};
use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem,
};
use gpui_kit::component::*;
use gpui_kit::*;

impl RootView {
    pub(super) fn sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let menu = |tabs: &[Tab]| {
            tabs.iter().fold(SidebarMenu::new(), |menu, &tab| {
                menu.child(
                    SidebarMenuItem::new(tab.title())
                        .icon(tab.icon())
                        .active(self.tab == tab)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.on_navigate(tab, cx);
                        })),
                )
            })
        };
        let main = menu(&Tab::MAIN);
        let advanced = menu(&Tab::ADVANCED);

        Sidebar::new("sidebar")
            .header(
                SidebarHeader::new().child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(img("icon.svg").size_10().rounded_sm())
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::MEDIUM)
                                .child("zUI"),
                        ),
                ),
            )
            .child(SidebarGroup::new("").child(main))
            .child(SidebarGroup::new(crate::i18n::tr("ui-advanced")).child(advanced))
            .footer(SidebarFooter::new().child(self.sidebar_status(cx)))
            .into_any_element()
    }
}

impl RootView {
    /// Состояние обхода видно на любой вкладке; под ним тихая строка версии.
    fn sidebar_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::components::{palette, tone};
        let running = self.view_data.zapret_running;
        let (dot, label) = if self.state.picking() {
            (tone::warning(cx), crate::i18n::tr("pick-running"))
        } else if running {
            // Та же живая точка, что на главной: мерцает, пока обход работает.
            let pulse = if cx.reduce_motion() {
                1.
            } else {
                tone::pulse()
            };
            (
                tone::ok(cx).opacity(pulse),
                crate::i18n::tr("home-bypass-on"),
            )
        } else {
            (palette::muted(), crate::i18n::tr("home-bypass-off"))
        };
        div()
            .v_flex()
            .w_full()
            .gap_3()
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .h_9()
                    .px_3()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(palette::hairline())
                    .bg(palette::color(palette::ramp().card))
                    .text_xs()
                    .text_color(cx.theme().foreground)
                    .child(tone::status_dot(dot))
                    .child(label),
            )
            .child(
                div()
                    .px_3()
                    .text_xs()
                    .font_family(tone::MONO)
                    .text_color(palette::muted())
                    .child(crate::release::label()),
            )
    }
}
