#![windows_subsystem = "windows"]

pub mod config;
pub mod theme;
pub mod audio_engine;
pub mod ui;
pub mod startup;
pub mod i18n;
pub mod updater;

use config::AppConfig;
use audio_engine::AudioEngine;
use updater::UpdateManager;
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
            setup_system_fonts(&cc.egui_ctx, &config.language);
            let colors = theme::ThemeColors::new(&config.accent_hex);
            theme::apply_theme(&cc.egui_ctx, &colors);
            let updater = UpdateManager::new();
            Box::new(SoundLockApp {
                config,
                active_tab: Tab::General,
                audio_engine,
                updater,
                ui_state: UiState::default(),
                should_quit: false,
                frame_count: 0,
                last_trim: std::time::Instant::now(),
                colors,
            })
        }),
    )
}

pub fn setup_system_fonts(ctx: &egui::Context, language: &str) {
    let mut fonts = egui::FontDefinitions::empty();
    let low = language.to_lowercase();
    let is_cjk = low.contains("chinese") || low.contains("中文")
        || low.contains("japan") || low.contains("日本")
        || low.contains("korean") || low.contains("한국");

    // On-demand lightweight font loading: only load heavy CJK font tables when using CJK
    let candidates: &[&str] = if is_cjk {
        &[r"C:\Windows\Fonts\msyh.ttc", r"C:\Windows\Fonts\segoeui.ttf"]
    } else {
        &[r"C:\Windows\Fonts\segoeui.ttf"]
    };

    let mut loaded_any = false;
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
            loaded_any = true;
        }
    }

    if !loaded_any {
        fonts = egui::FontDefinitions::default();
    }

    ctx.set_fonts(fonts);
}

pub fn trim_working_set() {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
        use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
        use windows::Win32::System::Memory::{GetProcessHeap, HeapCompact, HEAP_FLAGS};
        if let Ok(heap) = GetProcessHeap() {
            if !heap.is_invalid() {
                let _ = HeapCompact(heap, HEAP_FLAGS(0));
            }
        }
        let p = GetCurrentProcess();
        let _ = EmptyWorkingSet(p);
        let _ = SetProcessWorkingSetSize(p, usize::MAX, usize::MAX);
    }
}

struct SoundLockApp {
    config: AppConfig,
    active_tab: Tab,
    audio_engine: AudioEngine,
    updater: UpdateManager,
    ui_state: UiState,
    should_quit: bool,
    frame_count: u32,
    last_trim: std::time::Instant,
    colors: theme::ThemeColors,
}

impl eframe::App for SoundLockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.should_quit {
            self.audio_engine.stop();
            // Allow audio worker thread to cleanly restore all ducked volumes back to 100%
            std::thread::sleep(std::time::Duration::from_millis(80));
            std::process::exit(0);
        }

        ui::render_ui(
            ctx,
            &mut self.config,
            &mut self.active_tab,
            &self.audio_engine,
            &self.updater,
            &mut self.ui_state,
            &mut self.should_quit,
            &self.colors,
        );

        // Adaptive frame pacing: 30 FPS when active or game running, 150ms when focused, 1000ms in background
        let is_active = self.ui_state.is_recording_clutch_key
            || self.audio_engine.telemetry.lock().map_or(false, |t| t.is_ducking_active || t.is_game_running);
        let is_focused = ctx.input(|i| i.focused);

        let repaint_ms = if is_active {
            32
        } else if is_focused {
            150
        } else {
            1000
        };
        ctx.request_repaint_after(std::time::Duration::from_millis(repaint_ms));

        // Aggressive RAM Optimization: trim working set after first frames and periodically when idle
        self.frame_count = self.frame_count.saturating_add(1);
        let idle_trim_secs = if is_focused { 5 } else { 2 };
        if self.frame_count == 2 || self.frame_count == 5 || (self.last_trim.elapsed().as_secs() >= idle_trim_secs && !is_active) {
            trim_working_set();
            self.last_trim = std::time::Instant::now();
        }
    }
}
