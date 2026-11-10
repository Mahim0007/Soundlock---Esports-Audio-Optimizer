use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use std::time::Instant;
use crate::config::AppConfig;
use crate::audio_engine::{AudioEngine, AudioEngineCommand, EngineTelemetry};
use crate::theme::{self, ThemeColors};
use crate::startup;
use crate::i18n::Language;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    General,
    SmartDucking,
    FootstepBooster,
    AppMixer,
    SpecificApps,
    Shortcuts,
    Developer,
    Information,
}

pub struct UiState {
    pub is_recording_clutch_key: bool,
    pub custom_app_input: String,
    pub status_msg: Option<(String, Instant)>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            is_recording_clutch_key: false,
            custom_app_input: String::new(),
            status_msg: None,
        }
    }
}

pub fn render_ui(
    ctx: &egui::Context,
    config: &mut AppConfig,
    active_tab: &mut Tab,
    audio_engine: &AudioEngine,
    updater: &crate::updater::UpdateManager,
    ui_state: &mut UiState,
    should_quit: &mut bool,
    colors: &ThemeColors,
) {
    let mut lang = Language::from_str(&config.language);

    // Global Key Capture for Clutch Hotkey rebinding
    if ui_state.is_recording_clutch_key {
        capture_key_press(ctx, config, ui_state, audio_engine);
    }

    let telemetry = audio_engine.telemetry.lock().unwrap_or_else(|e| e.into_inner()).clone();

    // ----------------------------------------------------
    // LEFT SIDEBAR (Dark Acrylic Sidebar matching screenshot)
    // ----------------------------------------------------
    egui::SidePanel::left("left_sidebar")
        .resizable(false)
        .exact_width(190.0)
        .frame(
            egui::Frame::none()
                .fill(colors.sidebar_bg)
                .inner_margin(egui::Margin::symmetric(10.0, 14.0)),
        )
        .show(ctx, |ui| {
            render_sidebar_tab(ui, active_tab, Tab::General, lang.tab_general(), SidebarIcon::Gear, &colors);
            render_sidebar_tab(ui, active_tab, Tab::SmartDucking, lang.tab_ducking(), SidebarIcon::Toolbar, &colors);
            render_sidebar_tab(ui, active_tab, Tab::FootstepBooster, lang.tab_booster(), SidebarIcon::Window, &colors);
            render_sidebar_tab(ui, active_tab, Tab::AppMixer, lang.tab_mixer(), SidebarIcon::Dock, &colors);
            render_sidebar_tab(ui, active_tab, Tab::SpecificApps, lang.tab_apps(), SidebarIcon::Target, &colors);
            render_sidebar_tab(ui, active_tab, Tab::Shortcuts, lang.tab_shortcuts(), SidebarIcon::Shortcuts, &colors);
            render_sidebar_tab(ui, active_tab, Tab::Developer, lang.tab_developer(), SidebarIcon::Developer, &colors);

            // Bottom Information Tab (matching screenshot)
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(4.0);
                render_sidebar_tab(ui, active_tab, Tab::Information, lang.tab_info(), SidebarIcon::Info, &colors);
            });
        });

    // ----------------------------------------------------
    // MAIN CONTENT PANEL
    // ----------------------------------------------------
    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(colors.bg_dark)
                .inner_margin(18.0),
        )
        .show(ctx, |ui| {
            // TOP HEADER: Section Title on left + "Cancel" & "Quit" on right
            ui.horizontal(|ui| {
                let tab_title = match *active_tab {
                    Tab::General => lang.tab_general(),
                    Tab::SmartDucking => lang.tab_ducking(),
                    Tab::FootstepBooster => lang.tab_booster(),
                    Tab::AppMixer => lang.tab_mixer(),
                    Tab::SpecificApps => lang.tab_apps(),
                    Tab::Shortcuts => lang.tab_shortcuts(),
                    Tab::Developer => lang.tab_developer(),
                    Tab::Information => lang.tab_info(),
                };

                ui.label(
                    egui::RichText::new(tab_title)
                        .strong()
                        .size(16.0)
                        .color(colors.text_primary),
                );

                // Right aligned action buttons matching screenshot
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let quit_btn = egui::Button::new(
                        egui::RichText::new(lang.btn_quit())
                            .strong()
                            .size(13.0)
                            .color(Color32::WHITE),
                    )
                    .fill(colors.accent)
                    .rounding(egui::Rounding::same(6.0));

                    if ui.add(quit_btn).clicked() {
                        *should_quit = true;
                    }

                    ui.add_space(8.0);

                    let cancel_btn = egui::Button::new(
                        egui::RichText::new(lang.btn_cancel())
                            .size(13.0)
                            .color(colors.text_secondary),
                    )
                    .fill(Color32::from_rgb(32, 36, 50))
                    .rounding(egui::Rounding::same(6.0));

                    if ui.add(cancel_btn).clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }

                    // Auto-updater status pill in top bar
                    let update_status = updater.status.lock().map(|s| s.clone()).unwrap_or(crate::updater::UpdateStatus::Idle);
                    match &update_status {
                        crate::updater::UpdateStatus::Available { version, download_url, .. } => {
                            ui.add_space(8.0);
                            let update_btn = egui::Button::new(
                                egui::RichText::new(format!("🚀 Update v{}", version))
                                    .strong()
                                    .size(11.5)
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(0, 150, 240))
                            .rounding(egui::Rounding::same(6.0));

                            if ui.add(update_btn).clicked() {
                                updater.download_and_install(download_url.clone());
                            }
                        }
                        crate::updater::UpdateStatus::Downloading(_) => {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("⬇ Downloading...")
                                    .size(11.5)
                                    .color(Color32::from_rgb(0, 200, 255)),
                            );
                        }
                        crate::updater::UpdateStatus::ReadyToRestart { new_exe, current_exe } => {
                            ui.add_space(8.0);
                            let restart_btn = egui::Button::new(
                                egui::RichText::new("🔄 Restart to Update")
                                    .strong()
                                    .size(11.5)
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(0, 180, 80))
                            .rounding(egui::Rounding::same(6.0));

                            if ui.add(restart_btn).clicked() {
                                let _ = updater.apply_and_restart(new_exe, current_exe);
                                audio_engine.stop();
                                std::thread::sleep(std::time::Duration::from_millis(80));
                                std::process::exit(0);
                            }
                        }
                        _ => {}
                    }
                });
            });

            ui.add_space(14.0);

            // RENDER SELECTED TAB
            egui::ScrollArea::vertical().show(ui, |ui| {
                match *active_tab {
                    Tab::General => render_general_view(ui, config, &colors, &telemetry, audio_engine, &mut lang),
                    Tab::SmartDucking => render_ducking_view(ui, config, &colors, &telemetry, audio_engine, lang),
                    Tab::FootstepBooster => render_footstep_view(ui, config, &colors, lang),
                    Tab::AppMixer => render_mixer_view(ui, &telemetry, audio_engine, config, &colors, lang),
                    Tab::SpecificApps => render_specific_apps_view(ui, config, ui_state, &telemetry, audio_engine, &colors, lang),
                    Tab::Shortcuts => render_shortcuts_view(ui, config, ui_state, &telemetry, audio_engine, &colors, lang),
                    Tab::Developer => render_developer_view(ui, &telemetry, &colors),
                    Tab::Information => render_information_view(ui, &colors, updater, audio_engine),
                }
            });
        });
}

// ----------------------------------------------------
// 1. GENERAL TAB (EXACTLY MATCHING USER'S SCREENSHOT!)
// ----------------------------------------------------
fn render_general_view(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    colors: &ThemeColors,
    telemetry: &EngineTelemetry,
    audio_engine: &AudioEngine,
    lang: &mut Language,
) {
    // CARD 1: Run on startup & Language
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(lang.run_on_startup())
                        .size(13.5)
                        .color(colors.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::toggle_switch(ui, &mut config.run_on_startup, colors.accent).changed() {
                        let _ = startup::set_startup_enabled(config.run_on_startup);
                        let _ = config.save();
                    }
                });
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(lang.language_label())
                        .size(13.5)
                        .color(colors.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_source("lang_combo")
                        .selected_text(lang.display_name())
                        .show_ui(ui, |ui| {
                            for l in Language::all() {
                                let is_active = *lang == *l;
                                if ui.selectable_label(is_active, l.display_name()).clicked() {
                                    config.language = l.display_name().to_string();
                                    *lang = *l;
                                    let _ = config.save();
                                    crate::setup_system_fonts(ui.ctx(), &config.language);
                                    crate::trim_working_set();
                                }
                            }
                        });
                });
            });
        });

    ui.add_space(14.0);

    // CARD 2: Game Wallpaper / Target Status Card (Matching Screenshot Preview Area!)
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Game Preview Thumbnail / Live Visualizer
                let (rect, _) = ui.allocate_exact_size(Vec2::new(180.0, 100.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, egui::Rounding::same(8.0), Color32::from_rgb(14, 16, 24));
                ui.painter().rect_stroke(rect, egui::Rounding::same(8.0), Stroke::new(1.0_f32, colors.card_stroke));

                // Stylized neon waveform inside thumbnail
                let center_y = rect.center().y;
                let step_x = rect.width() / 16.0;
                for i in 0..16 {
                    let h = if telemetry.is_game_running {
                        ((i as f32 * 0.8 + (telemetry.game_peak_level * 5.0)).sin().abs() * 30.0) + 8.0
                    } else {
                        4.0
                    };
                    let x = rect.min.x + (i as f32) * step_x + step_x * 0.5;
                    ui.painter().line_segment(
                        [Pos2::new(x, center_y - h), Pos2::new(x, center_y + h)],
                        Stroke::new(2.5_f32, colors.accent),
                    );
                }

                ui.add_space(20.0);

                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(lang.target_game())
                            .strong()
                            .size(14.0)
                            .color(colors.text_primary),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(format!("Active: {}", telemetry.detected_game))
                            .size(12.0)
                            .color(if telemetry.is_game_running { Color32::from_rgb(0, 240, 140) } else { colors.text_muted }),
                    );

                    ui.add_space(10.0);

                    // Dropdown for target game
                    egui::ComboBox::from_id_source("game_selector_combo")
                        .selected_text(&config.target_game)
                        .show_ui(ui, |ui| {
                            let games = [
                                "Auto (Any Active Game)",
                                "VALORANT",
                                "Counter-Strike 2 (cs2)",
                                "Call of Duty / Warzone",
                                "Apex Legends",
                                "Rainbow Six Siege",
                                "Fortnite",
                                "Overwatch 2",
                                "PUBG: BATTLEGROUNDS",
                                "Escape from Tarkov",
                                "Rust",
                                "The Finals",
                                "Dota 2",
                                "League of Legends",
                            ];
                            for g in games {
                                if ui.selectable_value(&mut config.target_game, g.to_string(), g).clicked() {
                                    audio_engine.update_config(config);
                                    let _ = config.save();
                                }
                            }
                        });
                });
            });
        });
}

// ----------------------------------------------------
// 2. SMART DUCKING TAB
// ----------------------------------------------------
fn render_ducking_view(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    colors: &ThemeColors,
    telemetry: &EngineTelemetry,
    audio_engine: &AudioEngine,
    lang: Language,
) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(lang.ducking_title())
                        .strong()
                        .size(15.0)
                        .color(colors.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::toggle_switch(ui, &mut config.ducking_enabled, colors.accent).changed() {
                        audio_engine.update_config(config);
                        let _ = config.save();
                    }
                });
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(lang.ducking_desc())
                    .size(12.5)
                    .color(colors.text_secondary),
            );

            ui.add_space(16.0);

            // Duck Volume Level Slider with White Knob
            ui.label(
                egui::RichText::new(lang.duck_volume_label())
                    .size(13.0)
                    .color(colors.text_primary),
            );
            let resp = theme::white_knob_slider(
                ui,
                &mut config.duck_level_percent,
                0.0,
                60.0,
                Some(5.0),
                "%",
                1,
                colors.accent,
            );
            if resp.changed() {
                audio_engine.update_config(config);
            }
            if resp.drag_stopped() {
                let _ = config.save();
            }

            ui.add_space(12.0);

            // Trigger Sensitivity Slider with White Knob (Footsteps: 0.01 - 0.04)
            ui.label(
                egui::RichText::new(format!("{} ({:.1}% Sensitivity)", lang.trigger_sensitivity_label(), config.trigger_threshold * 100.0))
                    .size(13.0)
                    .color(colors.text_primary),
            );
            let resp = theme::white_knob_slider(
                ui,
                &mut config.trigger_threshold,
                0.01,
                0.25,
                Some(0.005),
                "",
                3,
                colors.accent,
            );
            if resp.changed() {
                audio_engine.update_config(config);
            }
            if resp.drag_stopped() {
                let _ = config.save();
            }

            ui.add_space(16.0);

            // Live Ducking Status Badge + Test Button
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(lang.engine_status()).size(13.0).color(colors.text_secondary));
                if telemetry.is_ducking_active {
                    ui.label(
                        egui::RichText::new(lang.ducking_active_status())
                            .strong()
                            .size(13.0)
                            .color(colors.accent),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(lang.normal_volume_status())
                            .size(13.0)
                            .color(Color32::from_rgb(0, 240, 140)),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let test_btn = egui::Button::new(
                        egui::RichText::new(lang.test_ducking_btn())
                            .size(12.5)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(32, 42, 60))
                    .rounding(egui::Rounding::same(6.0));

                    if ui.add(test_btn).clicked() {
                        audio_engine.send_command(AudioEngineCommand::TestDucking {
                            duration: std::time::Duration::from_secs(3),
                        });
                    }
                });
            });
        });

    ui.add_space(14.0);

    // CARD 2: SoundLock Gunfire / Explosion Limiter (Hearing Protection)
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("🛡️ SoundLock Ear Guard (Gunfire Limiter)")
                        .strong()
                        .size(15.0)
                        .color(colors.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::toggle_switch(ui, &mut config.sound_limiter_enabled, colors.accent).changed() {
                        audio_engine.update_config(config);
                        let _ = config.save();
                    }
                });
            });

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Dynamically limits deafening gunshots, flashbangs, and explosions above ceiling level so you can turn your volume up to hear footsteps without ear pain.")
                    .size(12.5)
                    .color(colors.text_secondary),
            );

            ui.add_space(14.0);

            ui.label(
                egui::RichText::new(format!("Max Sound Ceiling ({:.0}%)", config.sound_limiter_ceiling))
                    .size(13.0)
                    .color(colors.text_primary),
            );
            let resp = theme::white_knob_slider(
                ui,
                &mut config.sound_limiter_ceiling,
                30.0,
                95.0,
                Some(5.0),
                "%",
                0,
                colors.accent,
            );
            if resp.changed() {
                audio_engine.update_config(config);
            }
            if resp.drag_stopped() {
                let _ = config.save();
            }

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Limiter Guard:").size(13.0).color(colors.text_secondary));
                if telemetry.is_limiter_active {
                    ui.label(
                        egui::RichText::new("⚡ CLAMPING LOUD GUNFIRE")
                            .strong()
                            .size(13.0)
                            .color(colors.accent),
                    );
                } else if config.sound_limiter_enabled {
                    ui.label(
                        egui::RichText::new("✓ PROTECTING EARS (ARMED)")
                            .size(13.0)
                            .color(Color32::from_rgb(0, 240, 140)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("OFF")
                            .size(13.0)
                            .color(colors.text_muted),
                    );
                }
            });
        });
}

// ----------------------------------------------------
// 3. FOOTSTEP BOOSTER TAB
// ----------------------------------------------------
fn render_footstep_view(ui: &mut egui::Ui, config: &mut AppConfig, colors: &ThemeColors, lang: Language) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(lang.booster_title())
                        .strong()
                        .size(15.0)
                        .color(colors.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::toggle_switch(ui, &mut config.footstep_boost_enabled, colors.accent).changed() {
                        let _ = config.save();
                    }
                });
            });

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Amplifies high-mid frequencies (2.5 kHz – 6.0 kHz) where shoe scuffs, wood creaks, and bomb defuse clicks register.")
                    .size(12.5)
                    .color(colors.text_secondary),
            );

            ui.add_space(14.0);

            // Equalizer Gain Slider
            ui.label(
                egui::RichText::new(lang.booster_gain_label())
                    .size(13.0)
                    .color(colors.text_primary),
            );
            let resp = theme::white_knob_slider(
                ui,
                &mut config.footstep_boost_db,
                0.0,
                12.0,
                Some(1.0),
                " dB",
                0,
                colors.accent,
            );
            if resp.drag_stopped() {
                let _ = config.save();
            }

            ui.add_space(14.0);

            // 3-Band Equalizer Cards
            ui.columns(3, |cols| {
                cols[0].group(|ui| {
                    ui.label(egui::RichText::new("Low Rumble (Explosions)").strong().size(12.0).color(colors.text_primary));
                    ui.label(egui::RichText::new("Dampened (-4 dB)").small().color(colors.text_muted));
                });
                cols[1].group(|ui| {
                    ui.label(egui::RichText::new("Voice / Mid (Discord)").strong().size(12.0).color(colors.text_primary));
                    ui.label(egui::RichText::new("Neutral (0 dB)").small().color(colors.text_muted));
                });
                cols[2].group(|ui| {
                    ui.label(egui::RichText::new("Footsteps (2-6 kHz)").strong().size(12.0).color(colors.accent));
                    ui.label(egui::RichText::new(format!("+{} dB Boost", config.footstep_boost_db)).small().color(colors.accent));
                });
            });

            ui.add_space(14.0);

            // Windows Sound Enhancements Launcher Button
            let win_eq_btn = egui::Button::new(
                egui::RichText::new(lang.open_sound_panel_btn())
                    .size(12.5)
                    .color(colors.text_primary),
            )
            .fill(Color32::from_rgb(28, 32, 46))
            .rounding(egui::Rounding::same(6.0));

            if ui.add(win_eq_btn).clicked() {
                let _ = std::process::Command::new("control.exe").arg("mmsys.cpl").spawn();
            }
        });
}

// ----------------------------------------------------
// 4. APP MIXER TAB (Live Active Audio Sessions with Real Volume Control!)
// ----------------------------------------------------
fn render_mixer_view(
    ui: &mut egui::Ui,
    telemetry: &EngineTelemetry,
    audio_engine: &AudioEngine,
    config: &mut AppConfig,
    colors: &ThemeColors,
    lang: Language,
) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(lang.tab_mixer())
                    .strong()
                    .size(15.0)
                    .color(colors.text_primary),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(lang.mixer_desc())
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(12.0);

            if telemetry.active_apps.is_empty() {
                ui.label(
                    egui::RichText::new("No active audio sessions currently playing sound.")
                        .size(12.5)
                        .color(colors.text_muted),
                );
            } else {
                for app in &telemetry.active_apps {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // Process Name
                            ui.label(egui::RichText::new(&app.name).strong().size(13.0).color(colors.text_primary));

                            if app.is_target_duck {
                                ui.label(egui::RichText::new("[Ducking Target]").small().color(colors.accent));
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Mute / Unmute Button
                                let mute_text = if app.is_muted { "🔇 Unmute" } else { "🔊 Mute" };
                                if ui.button(mute_text).clicked() {
                                    audio_engine.send_command(AudioEngineCommand::SetAppMute {
                                        pid: app.pid,
                                        mute: !app.is_muted,
                                    });
                                }

                                ui.add_space(8.0);

                                // Toggle Ducking Target button
                                let duck_action_text = if app.is_target_duck { "Remove Duck" } else { "+ Auto Duck" };
                                if ui.button(duck_action_text).clicked() {
                                    if app.is_target_duck {
                                        config.target_duck_apps.retain(|a| !a.eq_ignore_ascii_case(&app.name));
                                    } else {
                                        if !config.target_duck_apps.iter().any(|a| a.eq_ignore_ascii_case(&app.name)) {
                                            config.target_duck_apps.push(app.name.clone());
                                        }
                                    }
                                    audio_engine.update_config(config);
                                    let _ = config.save();
                                }
                            });
                        });

                        ui.add_space(4.0);

                        // Peak meter & volume slider
                        ui.horizontal(|ui| {
                            // Volume Slider
                            let mut vol = (app.volume * 100.0).round();
                            ui.label(egui::RichText::new("Vol:").size(12.0).color(colors.text_secondary));
                            if theme::white_knob_slider(
                                ui,
                                &mut vol,
                                0.0,
                                100.0,
                                Some(1.0),
                                "%",
                                0,
                                colors.accent,
                            ).changed() {
                                audio_engine.send_command(AudioEngineCommand::SetAppVolume {
                                    pid: app.pid,
                                    volume: vol / 100.0,
                                });
                            }

                            // Peak Level indicator
                            let peak_pct = (app.peak * 100.0).round() as u32;
                            ui.label(
                                egui::RichText::new(format!("Peak: {}%", peak_pct))
                                    .size(11.0)
                                    .color(if app.peak > 0.5 { colors.accent } else { Color32::from_rgb(0, 220, 180) }),
                            );
                        });
                    });
                    ui.add_space(6.0);
                }
            }
        });
}

// ----------------------------------------------------
// 5. SPECIFIC APPS TAB
// ----------------------------------------------------
fn render_specific_apps_view(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    ui_state: &mut UiState,
    telemetry: &EngineTelemetry,
    audio_engine: &AudioEngine,
    colors: &ThemeColors,
    lang: Language,
) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(lang.specific_apps_title())
                    .strong()
                    .size(15.0)
                    .color(colors.text_primary),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(lang.specific_apps_desc())
                    .size(12.5)
                    .color(colors.text_secondary),
            );

            ui.add_space(14.0);

            // Presets including Amazon Music and Apple Music!
            let presets = [
                "Spotify.exe",
                "Discord.exe",
                "Amazon Music.exe",
                "AppleMusic.exe",
                "chrome.exe",
                "brave.exe",
                "msedge.exe",
                "firefox.exe",
                "vlc.exe",
                "opera.exe",
            ];

            ui.columns(2, |cols| {
                for (idx, app_name) in presets.iter().enumerate() {
                    let col = &mut cols[idx % 2];
                    let mut is_included = config.target_duck_apps.iter().any(|a| a.eq_ignore_ascii_case(app_name));
                    if col.checkbox(&mut is_included, *app_name).changed() {
                        if is_included {
                            if !config.target_duck_apps.contains(&app_name.to_string()) {
                                config.target_duck_apps.push(app_name.to_string());
                            }
                        } else {
                            config.target_duck_apps.retain(|a| !a.eq_ignore_ascii_case(app_name));
                        }
                        audio_engine.update_config(config);
                        let _ = config.save();
                    }
                }
            });

            ui.add_space(16.0);

            // Custom Application Input
            ui.separator();
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(lang.add_custom_exe())
                    .strong()
                    .size(13.0)
                    .color(colors.text_primary),
            );

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut ui_state.custom_app_input);
                let add_btn = egui::Button::new(lang.btn_add_app())
                    .fill(Color32::from_rgb(32, 42, 60))
                    .rounding(egui::Rounding::same(6.0));

                if ui.add(add_btn).clicked() {
                    let clean = ui_state.custom_app_input.trim();
                    if !clean.is_empty() {
                        let formatted = if clean.ends_with(".exe") {
                            clean.to_string()
                        } else {
                            format!("{}.exe", clean)
                        };
                        if !config.target_duck_apps.iter().any(|a| a.eq_ignore_ascii_case(&formatted)) {
                            config.target_duck_apps.push(formatted);
                            audio_engine.update_config(config);
                            let _ = config.save();
                            ui_state.custom_app_input.clear();
                        }
                    }
                }
            });

            // Detected running apps quick-add
            let non_ducked_running: Vec<String> = telemetry
                .active_apps
                .iter()
                .filter(|a| !a.is_target_duck && !a.name.starts_with("PID"))
                .map(|a| a.name.clone())
                .collect();

            if !non_ducked_running.is_empty() {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Detected Active Apps (Click to add):")
                        .size(12.0)
                        .color(colors.text_secondary),
                );
                ui.horizontal_wrapped(|ui| {
                    for r_app in non_ducked_running {
                        if ui.button(format!("+ {}", r_app)).clicked() {
                            config.target_duck_apps.push(r_app);
                            audio_engine.update_config(config);
                            let _ = config.save();
                        }
                    }
                });
            }
        });
}

// ----------------------------------------------------
// 6. SHORTCUTS & CLUTCH TAB (MATCHING USER SCREENSHOT!)
// ----------------------------------------------------
fn render_shortcuts_view(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    ui_state: &mut UiState,
    telemetry: &EngineTelemetry,
    audio_engine: &AudioEngine,
    colors: &ThemeColors,
    lang: Language,
) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(lang.clutch_title())
                    .strong()
                    .size(15.0)
                    .color(colors.text_primary),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(lang.clutch_desc())
                    .size(12.5)
                    .color(colors.text_secondary),
            );

            ui.add_space(16.0);

            // Clickable Hotkey Binder Box (Matching Screenshot Style)
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(lang.clutch_key_label())
                        .size(13.0)
                        .color(colors.text_primary),
                );

                let key_text = if ui_state.is_recording_clutch_key {
                    lang.press_any_key()
                } else {
                    &config.clutch_hotkey
                };

                let key_btn = egui::Button::new(
                    egui::RichText::new(key_text)
                        .strong()
                        .monospace()
                        .size(13.0)
                        .color(if ui_state.is_recording_clutch_key { colors.accent } else { Color32::WHITE }),
                )
                .fill(Color32::from_rgb(18, 20, 28))
                .stroke(Stroke::new(1.0_f32, if ui_state.is_recording_clutch_key { colors.accent } else { colors.card_stroke }))
                .rounding(egui::Rounding::same(6.0))
                .min_size(Vec2::new(140.0, 28.0));

                if ui.add(key_btn).clicked() {
                    ui_state.is_recording_clutch_key = !ui_state.is_recording_clutch_key;
                }
            });

            ui.add_space(14.0);

            // Mode Selector: Hold to Duck vs Press to Toggle
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(lang.trigger_mode()).size(12.5).color(colors.text_secondary));

                if ui.radio_value(&mut config.clutch_hold_mode, true, lang.hold_key_mode()).clicked() {
                    audio_engine.update_config(config);
                    let _ = config.save();
                }

                ui.add_space(10.0);

                if ui.radio_value(&mut config.clutch_hold_mode, false, lang.toggle_mode()).clicked() {
                    audio_engine.update_config(config);
                    let _ = config.save();
                }
            });

            ui.add_space(16.0);

            // Live Clutch Status & Manual Toggle Test
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(lang.clutch_status()).size(13.0).color(colors.text_secondary));

                if telemetry.is_clutch_active {
                    ui.label(
                        egui::RichText::new(lang.clutch_active_status())
                            .strong()
                            .size(13.0)
                            .color(colors.accent),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(lang.clutch_idle_status())
                            .size(13.0)
                            .color(Color32::from_rgb(0, 240, 140)),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let clutch_toggle_btn = egui::Button::new(
                        egui::RichText::new(lang.test_clutch_btn())
                            .size(12.5)
                            .color(Color32::WHITE),
                    )
                    .fill(if telemetry.is_clutch_active { colors.accent } else { Color32::from_rgb(32, 42, 60) })
                    .rounding(egui::Rounding::same(6.0));

                    if ui.add(clutch_toggle_btn).clicked() {
                        audio_engine.send_command(AudioEngineCommand::ToggleClutch);
                    }
                });
            });
        });
}

// ----------------------------------------------------
// 7. DEVELOPER & TELEMETRY TAB (0.00% CPU Benchmark)
// ----------------------------------------------------
fn render_developer_view(ui: &mut egui::Ui, telemetry: &EngineTelemetry, colors: &ThemeColors) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Esports Architecture Telemetry")
                    .strong()
                    .size(15.0)
                    .color(colors.text_primary),
            );
            ui.add_space(12.0);

            let (tier_label, tier_color) = match telemetry.cpu_tier {
                1 => ("Tier 1: High-Performance Esports Battle (16ms)", colors.accent),
                2 => ("Tier 2: Background / Lobby Standby (60ms)", Color32::from_rgb(0, 220, 255)),
                _ => ("Tier 3: Zero-Power Hibernation (250ms, 0.00% CPU)", Color32::from_rgb(0, 240, 140)),
            };

            ui.label(
                egui::RichText::new(format!("Active Scheduling State: {}", tier_label))
                    .strong()
                    .size(13.0)
                    .color(tier_color),
            );
            ui.add_space(6.0);
            ui.label(egui::RichText::new(format!("Game Audio Peak Level: {:.2}", telemetry.game_peak_level)).size(12.5).color(colors.text_secondary));
            ui.label(egui::RichText::new(format!("Volume Ducking Multiplier: {:.2}x", telemetry.current_duck_multiplier)).size(12.5).color(colors.text_secondary));
            ui.label(egui::RichText::new(format!("Active Windows Audio Sessions: {}", telemetry.active_apps.len())).size(12.5).color(colors.text_secondary));
        });
}

// ----------------------------------------------------
// 8. INFORMATION TAB
// ----------------------------------------------------
fn render_information_view(
    ui: &mut egui::Ui,
    colors: &ThemeColors,
    updater: &crate::updater::UpdateManager,
    audio_engine: &AudioEngine,
) {
    // Card 1: App Info
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!("SoundLock Suite v{}", updater.current_version()))
                    .strong()
                    .size(18.0)
                    .color(colors.accent),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Esports-Grade Smart Game-Audio Ducking & Footstep Booster")
                    .size(13.0)
                    .color(colors.text_primary),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Engineered 100% in pure Rust with native WASAPI integration for sub-millisecond reaction and zero game FPS drop.")
                    .size(12.5)
                    .color(colors.text_secondary),
            );
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("GitHub Repository:").size(12.5).color(colors.text_muted));
                ui.hyperlink_to("github.com/Mahim0007/Soundlock---Esports-Audio-Optimizer", "https://github.com/Mahim0007/Soundlock---Esports-Audio-Optimizer");
            });
        });

    ui.add_space(14.0);

    // Card 2: Software Updates & GitHub Auto-Update
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Software Updates")
                    .strong()
                    .size(15.0)
                    .color(colors.text_primary),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("SoundLock checks GitHub Releases in the background. When you release a new version on GitHub, the app detects it and lets you update with 1 click.")
                    .size(12.5)
                    .color(colors.text_secondary),
            );
            ui.add_space(12.0);

            let status = updater.status.lock().map(|s| s.clone()).unwrap_or(crate::updater::UpdateStatus::Idle);
            match status {
                crate::updater::UpdateStatus::Idle => {
                    let btn = egui::Button::new(
                        egui::RichText::new("⟳ Check for Updates Now")
                            .size(13.0)
                            .color(colors.text_primary),
                    )
                    .fill(Color32::from_rgb(32, 38, 54))
                    .rounding(egui::Rounding::same(6.0));

                    if ui.add(btn).clicked() {
                        updater.check_for_updates();
                    }
                }
                crate::updater::UpdateStatus::Checking => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new("Checking GitHub for newer releases...")
                                .size(13.0)
                                .color(colors.text_primary),
                        );
                    });
                }
                crate::updater::UpdateStatus::UpToDate(ver) => {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("✓ SoundLock is up to date (v{})", ver))
                                .strong()
                                .size(13.0)
                                .color(Color32::from_rgb(0, 220, 120)),
                        );
                        ui.add_space(10.0);
                        if ui.button("⟳ Check Again").clicked() {
                            updater.check_for_updates();
                        }
                    });
                }
                crate::updater::UpdateStatus::Available { version, download_url, notes } => {
                    ui.label(
                        egui::RichText::new(format!("🚀 New Version Available: v{}", version))
                            .strong()
                            .size(14.0)
                            .color(Color32::from_rgb(0, 180, 255)),
                    );
                    if !notes.is_empty() {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!("Release Notes: {}", notes))
                                .small()
                                .color(colors.text_muted),
                        );
                    }
                    ui.add_space(10.0);

                    let dl_btn = egui::Button::new(
                        egui::RichText::new("⚡ Download & Install Update Now")
                            .strong()
                            .size(13.0)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 150, 240))
                    .rounding(egui::Rounding::same(6.0));

                    if ui.add(dl_btn).clicked() {
                        updater.download_and_install(download_url);
                    }
                }
                crate::updater::UpdateStatus::Downloading(_) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new("Downloading latest release package from GitHub...")
                                .size(13.0)
                                .color(Color32::from_rgb(0, 200, 255)),
                        );
                    });
                }
                crate::updater::UpdateStatus::ReadyToRestart { new_exe, current_exe } => {
                    ui.label(
                        egui::RichText::new("✓ Update downloaded and verified successfully!")
                            .strong()
                            .size(13.0)
                            .color(Color32::from_rgb(0, 220, 120)),
                    );
                    ui.add_space(10.0);

                    let restart_btn = egui::Button::new(
                        egui::RichText::new("🔄 Restart SoundLock to Apply Update")
                            .strong()
                            .size(13.0)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 180, 80))
                    .rounding(egui::Rounding::same(6.0));

                    if ui.add(restart_btn).clicked() {
                        let _ = updater.apply_and_restart(&new_exe, &current_exe);
                        audio_engine.stop();
                        std::thread::sleep(std::time::Duration::from_millis(80));
                        std::process::exit(0);
                    }
                }
                crate::updater::UpdateStatus::Failed(err) => {
                    ui.label(
                        egui::RichText::new(format!("⚠ Update check: {}", err))
                            .size(13.0)
                            .color(Color32::from_rgb(255, 120, 120)),
                    );
                    ui.add_space(8.0);
                    if ui.button("⟳ Retry Check").clicked() {
                        updater.check_for_updates();
                    }
                }
            }
        });
}

// ----------------------------------------------------
// KEY CAPTURE HELPER FOR SHORTCUT REBINDING
// ----------------------------------------------------
fn capture_key_press(
    ctx: &egui::Context,
    config: &mut AppConfig,
    ui_state: &mut UiState,
    audio_engine: &AudioEngine,
) {
    let mut detected: Option<String> = None;

    ctx.input(|i| {
        for event in &i.raw.events {
            if let egui::Event::Key { key, pressed: true, .. } = event {
                if *key == egui::Key::Escape {
                    ui_state.is_recording_clutch_key = false;
                    return;
                }
                let name = match key {
                    egui::Key::F1 => "F1",
                    egui::Key::F2 => "F2",
                    egui::Key::F3 => "F3",
                    egui::Key::F4 => "F4",
                    egui::Key::F5 => "F5",
                    egui::Key::F6 => "F6",
                    egui::Key::F7 => "F7",
                    egui::Key::F8 => "F8",
                    egui::Key::F9 => "F9",
                    egui::Key::F10 => "F10",
                    egui::Key::F11 => "F11",
                    egui::Key::F12 => "F12",
                    egui::Key::Space => "Space",
                    egui::Key::Tab => "Tab",
                    egui::Key::A => "A",
                    egui::Key::B => "B",
                    egui::Key::C => "C",
                    egui::Key::D => "D",
                    egui::Key::E => "E",
                    egui::Key::F => "F",
                    egui::Key::G => "G",
                    egui::Key::H => "H",
                    egui::Key::I => "I",
                    egui::Key::J => "J",
                    egui::Key::K => "K",
                    egui::Key::L => "L",
                    egui::Key::M => "M",
                    egui::Key::N => "N",
                    egui::Key::O => "O",
                    egui::Key::P => "P",
                    egui::Key::Q => "Q",
                    egui::Key::R => "R",
                    egui::Key::S => "S",
                    egui::Key::T => "T",
                    egui::Key::U => "U",
                    egui::Key::V => "V",
                    egui::Key::W => "W",
                    egui::Key::X => "X",
                    egui::Key::Y => "Y",
                    egui::Key::Z => "Z",
                    _ => "",
                };
                if !name.is_empty() {
                    detected = Some(name.to_string());
                    break;
                }
            }
        }
    });

    #[cfg(windows)]
    if detected.is_none() {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        let candidates = [
            (VK_CAPITAL.0 as i32, "Caps Lock"),
            (VK_XBUTTON1.0 as i32, "Mouse 4"),
            (VK_XBUTTON2.0 as i32, "Mouse 5"),
            (VK_MBUTTON.0 as i32, "Middle Click"),
            (VK_LMENU.0 as i32, "Left Alt"),
            (VK_LCONTROL.0 as i32, "Left Ctrl"),
            (VK_LSHIFT.0 as i32, "Left Shift"),
            (VK_SPACE.0 as i32, "Space"),
            (VK_TAB.0 as i32, "Tab"),
        ];

        for (vk, name) in candidates {
            unsafe {
                if (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(vk) as u16 & 0x8000) != 0 {
                    detected = Some(name.to_string());
                    break;
                }
            }
        }
    }

    if let Some(key_name) = detected {
        config.clutch_hotkey = key_name;
        audio_engine.update_config(config);
        let _ = config.save();
        ui_state.is_recording_clutch_key = false;
    }
}

// ----------------------------------------------------
// SIDEBAR ITEM HELPER (Active Pill matching screenshot!)
// ----------------------------------------------------
#[derive(Clone, Copy)]
pub enum SidebarIcon {
    Gear,
    Toolbar,
    Window,
    Dock,
    Target,
    Shortcuts,
    Developer,
    Info,
}

fn render_sidebar_tab(
    ui: &mut egui::Ui,
    active_tab: &mut Tab,
    target_tab: Tab,
    label: &str,
    icon: SidebarIcon,
    colors: &ThemeColors,
) {
    let is_selected = *active_tab == target_tab;
    let desired_size = Vec2::new(ui.available_width(), 34.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *active_tab = target_tab;
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let painter = ui.painter();

    if is_selected {
        painter.rect_filled(rect, egui::Rounding::same(8.0), colors.accent);
    } else if response.hovered() {
        painter.rect_filled(rect, egui::Rounding::same(8.0), Color32::from_rgba_unmultiplied(255, 255, 255, 12));
    }

    let text_color = if is_selected {
        Color32::WHITE
    } else {
        colors.text_secondary
    };

    // Draw vector icon
    let icon_center = Pos2::new(rect.min.x + 16.0, rect.center().y);
    draw_sidebar_icon(painter, icon_center, icon, text_color);

    // Draw label
    painter.text(
        Pos2::new(rect.min.x + 32.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.5),
        text_color,
    );

    ui.add_space(2.0);
}

fn draw_sidebar_icon(painter: &egui::Painter, center: Pos2, icon: SidebarIcon, color: Color32) {
    let stroke = Stroke::new(1.3_f32, color);
    match icon {
        SidebarIcon::Gear => {
            painter.circle_stroke(center, 4.5, stroke);
            painter.circle_filled(center, 1.5, color);
        }
        SidebarIcon::Toolbar => {
            painter.rect_stroke(egui::Rect::from_center_size(center, Vec2::new(10.0, 8.0)), egui::Rounding::same(1.5), stroke);
            painter.line_segment([Pos2::new(center.x - 5.0, center.y - 1.5), Pos2::new(center.x + 5.0, center.y - 1.5)], stroke);
        }
        SidebarIcon::Window => {
            painter.rect_stroke(egui::Rect::from_center_size(center, Vec2::new(11.0, 9.0)), egui::Rounding::same(2.0), stroke);
            painter.line_segment([Pos2::new(center.x, center.y - 4.5), Pos2::new(center.x, center.y + 4.5)], stroke);
        }
        SidebarIcon::Dock => {
            painter.rect_stroke(egui::Rect::from_center_size(center, Vec2::new(10.0, 7.0)), egui::Rounding::same(1.5), stroke);
        }
        SidebarIcon::Target => {
            painter.circle_stroke(center, 5.0, stroke);
            painter.circle_filled(center, 2.0, color);
        }
        SidebarIcon::Shortcuts => {
            painter.rect_stroke(egui::Rect::from_center_size(center, Vec2::new(10.0, 6.0)), egui::Rounding::same(1.0), stroke);
        }
        SidebarIcon::Developer => {
            painter.line_segment([Pos2::new(center.x - 3.5, center.y - 3.0), Pos2::new(center.x - 5.5, center.y)], stroke);
            painter.line_segment([Pos2::new(center.x - 5.5, center.y), Pos2::new(center.x - 3.5, center.y + 3.0)], stroke);
            painter.line_segment([Pos2::new(center.x + 3.5, center.y - 3.0), Pos2::new(center.x + 5.5, center.y)], stroke);
            painter.line_segment([Pos2::new(center.x + 5.5, center.y), Pos2::new(center.x + 3.5, center.y + 3.0)], stroke);
        }
        SidebarIcon::Info => {
            painter.circle_stroke(center, 5.0, stroke);
            painter.circle_filled(Pos2::new(center.x, center.y - 2.0), 0.8, color);
            painter.line_segment([Pos2::new(center.x, center.y - 0.5), Pos2::new(center.x, center.y + 2.5)], stroke);
        }
    }
}
