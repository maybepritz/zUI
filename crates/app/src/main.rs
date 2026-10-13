#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod components;
mod controllers;
mod i18n;
mod notifications;
mod platform;
mod release;
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

// Extend the default GPUI Kit icons with the Lucide variants used by zUI.
gpui_kit::assets::icon_assets!(
    AppIcons,
    [
        Star,
        StarFill,
        Search,
        RotateCcw,
        House,
        SlidersHorizontal,
        Network,
        Send,
        ChartNoAxesColumn,
        ScrollText,
        Settings,
        Sparkles,
        Power,
        Globe,
        MousePointer2,
        ArrowRight
    ]
);

impl AssetSource for EmbeddedAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let clean_path = path.trim_start_matches('/');
        if let Some(file) = EmbeddedAssets::get(clean_path) {
            return Ok(Some(file.data));
        }
        if let Some(data) = AppIcons.load(clean_path)? {
            return Ok(Some(data));
        }
        Ok(gpui_kit::assets::Assets::get(clean_path).map(|file| file.data))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let clean_path = path.trim_start_matches('/');
        let mut files: Vec<SharedString> = EmbeddedAssets::iter()
            .filter(|p| p.starts_with(clean_path))
            .map(|p| SharedString::from(p.to_string()))
            .collect();
        files.extend(AppIcons.list(clean_path)?);
        files.extend(gpui_kit::assets::Assets.list(clean_path)?);
        files.sort();
        files.dedup();
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
                    title: Some(release::title().into()),
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
