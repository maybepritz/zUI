//! Разметка боковой навигации.

use super::{RootView, Tab};
use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem,
};
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
            .footer(SidebarFooter::new().child(crate::release::label()))
            .into_any_element()
    }
}
