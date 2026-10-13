//! Разметка боковой навигации.

use super::{RootView, Tab};
use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem,
};
use gpui_kit::*;

impl RootView {
    pub(super) fn sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut menu = SidebarMenu::new();

        for tab in Tab::ALL {
            menu = menu.child(
                SidebarMenuItem::new(tab.title())
                    .icon(tab.icon())
                    .active(self.tab == tab)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.on_navigate(tab, cx);
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
            .footer(SidebarFooter::new().child(crate::release::label()))
            .into_any_element()
    }
}
