use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants};
use gpui_kit::component::*;
use gpui_kit::*;

pub fn section(title: &'static str) -> GroupBox {
    GroupBox::new().outline().title(title).gap_3()
}

pub fn button_row() -> Div {
    div().h_flex().flex_wrap().gap_2()
}
