#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod components;
mod controllers;
mod i18n;
mod notifications;
mod platform;
mod root;
mod state;
mod tray;
mod views;

use gpui::{AssetSource, SharedString};
use gpui_kit::*;
use root::RootView;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct EmbeddedAssets;

impl AssetSource for EmbeddedAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let clean_path = path.trim_start_matches('/');
        match EmbeddedAssets::get(clean_path) {
            Some(file) => Ok(Some(file.data)),
            None => Ok(None),
        }
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let clean_path = path.trim_start_matches('/');
        let files = EmbeddedAssets::iter()
            .filter(|p| p.starts_with(clean_path))
            .map(|p| SharedString::from(p.to_string()))
            .collect();
        Ok(files)
    }
}

fn main() {
    if !platform::wait_for_previous() {
        return;
    }
    platform::register_app_id();
    gpui_kit::application()
        .with_assets(EmbeddedAssets)
        .run(move |cx| {
            gpui_kit::init(cx);

            let bounds = Bounds::centered(None, size(px(900.), px(600.)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(750.), px(480.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("zUI".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| RootView::new(window, cx))
            })
            .expect("Failed to open window");
        });
}
