#![windows_subsystem = "windows"]

pub mod config;
pub mod theme;
pub mod audio_engine;
pub mod ui;
pub mod startup;
pub mod i18n;

use config::AppConfig;
use audio_engine::AudioEngine;
use ui::{Tab, UiState};

fn main() -> eframe::Result<()> {
    // Single instance mutex
    #[cfg(windows)]
    let _single_instance_mutex = unsafe {
        use windows::Win32::System::Threading::CreateMutexW;
        use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
        use windows::core::w;

        let handle_res = CreateMutexW(None, true, w!("Local\\SoundLock_Suite_SingleInstance"));
        if let Ok(handle) = handle_res {
            if GetLastError() == ERROR_ALREADY_EXISTS {
                return Ok(());
            }
            Some(handle)
        } else {
            None
        }
    };

    let mut vp = egui::ViewportBuilder::default()
        .with_inner_size([720.0, 480.0])
        .with_min_inner_size([640.0, 420.0])
        .with_title("SoundLock - Esports Audio Optimizer")
        .with_decorations(true)
        .with_transparent(false);

    if let Ok(icon_data) = eframe::icon_data::from_png_bytes(include_bytes!("../app_icon_256.png")) {
        vp = vp.with_icon(icon_data);
    }

    let options = eframe::NativeOptions {
        viewport: vp,
        vsync: true,
        ..Default::default()
    };

    let mut config = AppConfig::load();
    // Sync startup toggle with real Windows registry
    if let Ok(enabled) = startup::is_startup_enabled() {
        config.run_on_startup = enabled;
    }

    let audio_engine = AudioEngine::new(config.clone());

    eframe::run_native(
        "SoundLock Suite",
        options,
        Box::new(|cc| {
            setup_system_fonts(&cc.egui_ctx);
            let colors = theme::ThemeColors::new(&config.accent_hex);
            theme::apply_theme(&cc.egui_ctx, &colors);
            Box::new(SoundLockApp {
                config,
                active_tab: Tab::General,
                audio_engine,
                ui_state: UiState::default(),
                should_quit: false,
            })
        }),
    )
}

fn setup_system_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Check Windows system fonts for CJK (Chinese, Japanese, Korean) and Cyrillic (Russian)
    let candidates = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\malgun.ttf",
        r"C:\Windows\Fonts\segoeui.ttf",
    ];

    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let name = std::path::Path::new(path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            fonts.font_data.insert(name.clone(), egui::FontData::from_owned(bytes));
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push(name.clone());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push(name);
        }
    }

    ctx.set_fonts(fonts);
}

struct SoundLockApp {
    config: AppConfig,
    active_tab: Tab,
    audio_engine: AudioEngine,
    ui_state: UiState,
    should_quit: bool,
}

impl eframe::App for SoundLockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.should_quit {
            std::process::exit(0);
        }

        ui::render_ui(
            ctx,
            &mut self.config,
            &mut self.active_tab,
            &self.audio_engine,
            &mut self.ui_state,
            &mut self.should_quit,
        );

        // Smooth 60 FPS repainting when active or listening
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}
