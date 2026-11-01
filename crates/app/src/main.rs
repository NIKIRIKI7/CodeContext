#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod assets;
mod cli;
mod headless;

use std::path::PathBuf;

use cli::{LaunchMode, parse_args};
use core::system::ports::hotkey::GlobalHotkeyPort as _;
use core::system::ports::ipc::{PrimaryElectionResult, SingleInstanceIpcPort as _};
use di::AppContainer;
use gpui::{
    App, AppContext as _, Application, Bounds, Pixels, Size, TitlebarOptions, WindowBounds,
    WindowOptions, px, size,
};
use ui::features::main_window::view::MainWindowView;
use ui::theme::DEFAULT_LAYOUT;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    match parse_args(std::env::args()) {
        LaunchMode::Version => {
            println!("CodeContext {}", env!("CARGO_PKG_VERSION"));
        }
        LaunchMode::HeadlessPack { project_root } => {
            let _ = headless::run_headless_pack(&project_root);
        }
        LaunchMode::HeadlessApply { project_root } => {
            let _ = headless::run_headless_apply(&project_root);
        }
        LaunchMode::RegisterShell => {
            let _ = headless::register_os_context_menu();
        }
        LaunchMode::UnregisterShell => {
            let _ = headless::unregister_os_context_menu();
        }
        LaunchMode::SelfInstall => {
            let _ = headless::run_self_install();
        }
        LaunchMode::CheckUpdate => {
            let _ = headless::run_check_update();
        }
        LaunchMode::Gui { initial_project_root } => {
            let container = AppContainer::new();
            if matches!(
                container.ipc().ensure_primary_or_forward(initial_project_root.clone()),
                Ok(PrimaryElectionResult::ForwardedToPrimary)
            ) {
                return;
            }
            launch_gui(container, initial_project_root);
        }
    }
}

fn compute_adaptive_window_size(cx: &App) -> Size<Pixels> {
    let min_w = px(DEFAULT_LAYOUT.window_min_width);
    let min_h = px(DEFAULT_LAYOUT.window_min_height);
    let preferred_w = px(DEFAULT_LAYOUT.window_default_width);
    let preferred_h = px(DEFAULT_LAYOUT.window_default_height);

    if let Some(display) = cx.primary_display() {
        let screen_size = display.bounds().size;
        let max_safe_w = (screen_size.width * 0.86).max(min_w);
        let max_safe_h = (screen_size.height * 0.82).max(min_h);
        let target_w = preferred_w.min(max_safe_w);
        let target_h = preferred_h.min(max_safe_h);
        return size(target_w, target_h);
    }

    size(preferred_w, preferred_h)
}

fn launch_gui(container: AppContainer, initial_project_root: Option<PathBuf>) {
    Application::new().with_assets(assets::AppAssets).run(move |cx: &mut App| {
        let global_cfg = container.preset_service().load_global_config();
        let _ = container.hotkey().register_str(&global_cfg.global_hotkey);

        cx.set_global(container.clone());

        let window_size = compute_adaptive_window_size(cx);
        let bounds = Bounds::centered(None, window_size, cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(
                px(DEFAULT_LAYOUT.window_min_width),
                px(DEFAULT_LAYOUT.window_min_height),
            )),
            titlebar: Some(TitlebarOptions {
                title: Some(assets::APP_WINDOW_TITLE.into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..WindowOptions::default()
        };

        let _ = cx.open_window(options, move |_window, cx| {
            cx.new(|cx| MainWindowView::new(container, initial_project_root, cx))
        });
    });
}
