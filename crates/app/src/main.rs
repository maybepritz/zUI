#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod components;
mod controllers;
mod i18n;
mod notifications;
mod platform;
mod release;
mod root;
mod startup;
mod state;
mod tray;
mod views;

use gpui::{AssetSource, SharedString};
use gpui_kit::*;
use rust_embed::RustEmbed;
use startup::StartupView;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct EmbeddedAssets;

// Добавляем к стандартным иконкам GPUI Kit варианты Lucide, которые использует zUI.
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
        Power,
        Globe,
        MousePointer2,
        ArrowRight,
        Clock,
        ArrowRightLeft,
        Activity,
        TriangleAlert
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

/// Паника в релизе иначе не оставляет следов: консоли нет, журнал был только в памяти.
fn install_panic_hook() {
    let path = z_core::paths::log_file();
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let trace = std::backtrace::Backtrace::force_capture();
        z_core::log::append_line(&path, &format!("PANIC {info}\n{trace}"));
        previous(info);
    }));
}

fn main() {
    install_panic_hook();
    // Деинсталлятор: уборка без окна и без проверки единственного экземпляра.
    if platform::cleanup::requested() {
        platform::cleanup::run();
        return;
    }
    if !platform::wait_for_previous() {
        return;
    }
    // После ожидания: при перезапуске прежний экземпляр к этому моменту уже вышел.
    if !platform::claim_single_instance() {
        platform::activate_running();
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
                cx.new(|cx| StartupView::new(window, cx))
            })
            .expect("Failed to open window");
        });
}
