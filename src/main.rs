#![windows_subsystem = "windows"]

pub mod config;
pub mod theme;
pub mod audio_engine;
pub mod ui;

use config::AppConfig;
use audio_engine::AudioEngine;
use ui::Tab;

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

    let vp = egui::ViewportBuilder::default()
        .with_inner_size([720.0, 470.0])
        .with_min_inner_size([640.0, 420.0])
        .with_title("SoundLock - Esports Audio Optimizer")
        .with_decorations(true)
        .with_transparent(false);

    let options = eframe::NativeOptions {
        viewport: vp,
        vsync: true,
        ..Default::default()
    };

    let config = AppConfig::load();
    let audio_engine = AudioEngine::new();

    eframe::run_native(
        "SoundLock Suite",
        options,
        Box::new(|cc| {
            let colors = theme::ThemeColors::new(&config.accent_hex);
            theme::apply_theme(&cc.egui_ctx, &colors);
            Box::new(SoundLockApp {
                config,
                active_tab: Tab::General,
                audio_engine,
                should_quit: false,
            })
        }),
    )
}

struct SoundLockApp {
    config: AppConfig,
    active_tab: Tab,
    audio_engine: AudioEngine,
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
            &mut self.should_quit,
        );

        // Smooth 60 FPS repainting when active
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}
