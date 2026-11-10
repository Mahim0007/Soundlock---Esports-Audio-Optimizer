use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use std::time::Instant;
use crate::config::AppConfig;
use crate::audio_engine::{AudioEngine, AudioEngineCommand, EngineTelemetry};
use crate::theme::{self, ThemeColors};
use crate::startup;

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
    ui_state: &mut UiState,
    should_quit: &mut bool,
) {
    let is_bn = config.language == "Bengali";
    let colors = ThemeColors::new(&config.accent_hex);
    theme::apply_theme(ctx, &colors);

    // Global Key Capture for Clutch Hotkey rebinding
    if ui_state.is_recording_clutch_key {
        capture_key_press(ctx, config, ui_state, audio_engine);
    }

    let telemetry = audio_engine.telemetry.lock().unwrap().clone();

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
            render_sidebar_tab(ui, active_tab, Tab::General, if is_bn { "সাধারণ" } else { "General" }, SidebarIcon::Gear, &colors);
            render_sidebar_tab(ui, active_tab, Tab::SmartDucking, if is_bn { "স্মার্ট ডাকিং" } else { "Smart Ducking" }, SidebarIcon::Toolbar, &colors);
            render_sidebar_tab(ui, active_tab, Tab::FootstepBooster, if is_bn { "ফুটস্টেপ বুস্টার" } else { "Footstep Booster" }, SidebarIcon::Window, &colors);
            render_sidebar_tab(ui, active_tab, Tab::AppMixer, if is_bn { "অ্যাপ মিক্সার" } else { "App Mixer" }, SidebarIcon::Dock, &colors);
            render_sidebar_tab(ui, active_tab, Tab::SpecificApps, if is_bn { "নির্দিষ্ট অ্যাপস" } else { "Specific Apps" }, SidebarIcon::Target, &colors);
            render_sidebar_tab(ui, active_tab, Tab::Shortcuts, if is_bn { "শর্টকাটস" } else { "Shortcuts" }, SidebarIcon::Shortcuts, &colors);
            render_sidebar_tab(ui, active_tab, Tab::Developer, if is_bn { "ডেভেলপার" } else { "Developer" }, SidebarIcon::Developer, &colors);

            // Bottom Information Tab (matching screenshot)
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(4.0);
                render_sidebar_tab(ui, active_tab, Tab::Information, if is_bn { "তথ্য" } else { "Information" }, SidebarIcon::Info, &colors);
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
                let tab_title = match (*active_tab, is_bn) {
                    (Tab::General, false) => "General",
                    (Tab::General, true) => "সাধারণ সেটিংস (General)",
                    (Tab::SmartDucking, false) => "Smart Ducking",
                    (Tab::SmartDucking, true) => "স্মার্ট অডিও ডাকিং (Smart Ducking)",
                    (Tab::FootstepBooster, false) => "Footstep Booster",
                    (Tab::FootstepBooster, true) => "ফুটস্টেপ ফ্রিকোয়েন্সি বুস্টার",
                    (Tab::AppMixer, false) => "App Mixer",
                    (Tab::AppMixer, true) => "লাইভ অ্যাপ মিক্সার (App Mixer)",
                    (Tab::SpecificApps, false) => "Specific Apps",
                    (Tab::SpecificApps, true) => "টার্গেট অ্যাপস (Specific Apps)",
                    (Tab::Shortcuts, false) => "Shortcuts",
                    (Tab::Shortcuts, true) => "ক্ল্যাচ শর্টকাট কিবাইন্ড (Shortcuts)",
                    (Tab::Developer, false) => "Developer & Telemetry",
                    (Tab::Developer, true) => "ডেভেলপার ও সিস্টেম মেট্রিক্স",
                    (Tab::Information, false) => "Information",
                    (Tab::Information, true) => "অ্যাপ পরিচিতি ও সহায়িকা",
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
                        egui::RichText::new(if is_bn { "বন্ধ করুন" } else { "Quit" })
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
                        egui::RichText::new(if is_bn { "মিনিমাইজ" } else { "Cancel" })
                            .size(13.0)
                            .color(colors.text_secondary),
                    )
                    .fill(Color32::from_rgb(32, 36, 50))
                    .rounding(egui::Rounding::same(6.0));

                    if ui.add(cancel_btn).clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });

            ui.add_space(14.0);

            // RENDER SELECTED TAB
            egui::ScrollArea::vertical().show(ui, |ui| {
                match *active_tab {
                    Tab::General => render_general_view(ui, config, &colors, &telemetry, audio_engine, is_bn),
                    Tab::SmartDucking => render_ducking_view(ui, config, &colors, &telemetry, audio_engine, is_bn),
                    Tab::FootstepBooster => render_footstep_view(ui, config, &colors, is_bn),
                    Tab::AppMixer => render_mixer_view(ui, &telemetry, audio_engine, config, &colors, is_bn),
                    Tab::SpecificApps => render_specific_apps_view(ui, config, ui_state, &telemetry, audio_engine, &colors, is_bn),
                    Tab::Shortcuts => render_shortcuts_view(ui, config, ui_state, &telemetry, audio_engine, &colors, is_bn),
                    Tab::Developer => render_developer_view(ui, &telemetry, &colors, is_bn),
                    Tab::Information => render_information_view(ui, &colors, is_bn),
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
    is_bn: bool,
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
                    egui::RichText::new(if is_bn { "উইন্ডোজ স্টার্টআপে চালু হবে?" } else { "Run on startup?" })
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
                    egui::RichText::new(if is_bn { "ভাষা:" } else { "Language:" })
                        .size(13.5)
                        .color(colors.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_source("lang_combo")
                        .selected_text(if config.language == "Bengali" { "বাংলা (Bengali)" } else { "English" })
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut config.language, "English".to_string(), "English").clicked() {
                                let _ = config.save();
                            }
                            if ui.selectable_value(&mut config.language, "Bengali".to_string(), "বাংলা (Bengali)").clicked() {
                                let _ = config.save();
                            }
                        });
                });
            });
        });

    ui.add_space(14.0);

    // CARD 2: Accent Color Selector with Hex Pill & Swatches Strip (Matching Screenshot!)
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(if is_bn { "থিম কালার (Accent Color)" } else { "Accent Color" })
                        .size(13.5)
                        .color(colors.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let hex_frame = egui::Frame::none()
                        .fill(Color32::from_rgb(18, 22, 32))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 3.0));
                    hex_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                            ui.painter().rect_filled(dot_rect, egui::Rounding::ZERO, colors.accent);
                            ui.label(
                                egui::RichText::new(&config.accent_hex)
                                    .monospace()
                                    .size(11.0)
                                    .color(colors.text_primary),
                            );
                        });
                    });
                });
            });

            ui.add_space(10.0);

            if theme::color_palette_strip(ui, &mut config.accent_hex) {
                let _ = config.save();
            }
        });

    ui.add_space(14.0);

    // CARD 3: Game Wallpaper / Target Status Card (Matching Screenshot Preview Area!)
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
                        egui::RichText::new(if is_bn { "টার্গেট কম্পিটিটিভ গেম" } else { "Target Competitive Game" })
                            .strong()
                            .size(14.0)
                            .color(colors.text_primary),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(format!("{}: {}", if is_bn { "বর্তমান গেম" } else { "Active" }, telemetry.detected_game))
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
    is_bn: bool,
) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(if is_bn { "অটোমেটিক অডিও ডাকিং ইঞ্জিন" } else { "Automatic Audio Ducking Engine" })
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
                egui::RichText::new(
                    if is_bn {
                        "গেমে গুরুত্বপূর্ণ সাউন্ড (ফুটস্টেপ/গোলাগুলি) আসলে স্বয়ংক্রিয়ভাবে Spotify, Discord বা Chrome এর সাউন্ড কমিয়ে দেবে।"
                    } else {
                        "Automatically lowers Spotify, Discord, or Chrome when competitive footsteps and gunshots occur."
                    },
                )
                .size(12.5)
                .color(colors.text_secondary),
            );

            ui.add_space(16.0);

            // Duck Volume Level Slider with White Knob
            ui.label(
                egui::RichText::new(
                    if is_bn {
                        "ডাকিং চলার সময় ব্যাকগ্রাউন্ড অ্যাপের ভলিউম:"
                    } else {
                        "Background App Volume when Ducked:"
                    },
                )
                .size(13.0)
                .color(colors.text_primary),
            );
            if theme::white_knob_slider(
                ui,
                &mut config.duck_level_percent,
                0.0,
                60.0,
                Some(5.0),
                "%",
                1,
                colors.accent,
            ).changed() {
                audio_engine.update_config(config);
                let _ = config.save();
            }

            ui.add_space(12.0);

            // Trigger Sensitivity Slider with White Knob
            ui.label(
                egui::RichText::new(
                    if is_bn {
                        "গেম সাউন্ড ডিটেকশন সংবেদনশীলতা (Trigger Sensitivity):"
                    } else {
                        "Game Audio Trigger Sensitivity:"
                    },
                )
                .size(13.0)
                .color(colors.text_primary),
            );
            if theme::white_knob_slider(
                ui,
                &mut config.trigger_threshold,
                0.05,
                0.60,
                Some(0.01),
                "",
                3,
                colors.accent,
            ).changed() {
                audio_engine.update_config(config);
                let _ = config.save();
            }

            ui.add_space(16.0);

            // Live Ducking Status Badge + Test Button
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(if is_bn { "ইঞ্জিন স্ট্যাটাস:" } else { "Engine Status:" }).size(13.0).color(colors.text_secondary));
                if telemetry.is_ducking_active {
                    ui.label(
                        egui::RichText::new(if is_bn { "● ডাকিং অ্যাক্টিভ (ভলিউম কমানো হয়েছে)" } else { "● DUCKING ACTIVE (Audio Lowered)" })
                            .strong()
                            .size(13.0)
                            .color(colors.accent),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(if is_bn { "○ স্বাভাবিক (নরমাল ভলিউম)" } else { "○ Normal Volume (Idle)" })
                            .size(13.0)
                            .color(Color32::from_rgb(0, 240, 140)),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let test_btn = egui::Button::new(
                        egui::RichText::new(if is_bn { "⚡ ৩-সেকেন্ড ডাকিং টেস্ট করুন" } else { "⚡ Test Ducking (3 sec)" })
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
}

// ----------------------------------------------------
// 3. FOOTSTEP BOOSTER TAB
// ----------------------------------------------------
fn render_footstep_view(ui: &mut egui::Ui, config: &mut AppConfig, colors: &ThemeColors, is_bn: bool) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(if is_bn { "ফুটস্টেপ অডিও ফ্রিকোয়েন্সি অপটিমাইজার" } else { "Footstep Audio Frequency Equalizer" })
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
                egui::RichText::new(
                    if is_bn {
                        "জুতার ঘষা, কাঠের ক্রিক এবং বোমা ডিফ্যুজের ২.৫ kHz – ৬.০ kHz ফ্রিকোয়েন্সি স্পষ্ট করে যাতে শত্রু সহজে ধরা পড়ে।"
                    } else {
                        "Amplifies high-mid frequencies (2.5 kHz – 6.0 kHz) where shoe scuffs, wood creaks, and bomb defuse clicks register."
                    },
                )
                .size(12.5)
                .color(colors.text_secondary),
            );

            ui.add_space(14.0);

            // Equalizer Gain Slider
            ui.label(
                egui::RichText::new(if is_bn { "বুস্ট গেইন (Boost Gain):" } else { "Footstep Boost Gain:" })
                    .size(13.0)
                    .color(colors.text_primary),
            );
            if theme::white_knob_slider(
                ui,
                &mut config.footstep_boost_db,
                0.0,
                12.0,
                Some(1.0),
                " dB",
                0,
                colors.accent,
            ).changed() {
                let _ = config.save();
            }

            ui.add_space(14.0);

            // 3-Band Equalizer Cards
            ui.columns(3, |cols| {
                cols[0].group(|ui| {
                    ui.label(egui::RichText::new(if is_bn { "লো রাম্বল (বিস্ফোরণ)" } else { "Low Rumble (Explosions)" }).strong().size(12.0).color(colors.text_primary));
                    ui.label(egui::RichText::new(if is_bn { "কমানো (-4 dB)" } else { "Dampened (-4 dB)" }).small().color(colors.text_muted));
                });
                cols[1].group(|ui| {
                    ui.label(egui::RichText::new(if is_bn { "ভয়েস / চ্যাট (Discord)" } else { "Voice / Mid (Discord)" }).strong().size(12.0).color(colors.text_primary));
                    ui.label(egui::RichText::new(if is_bn { "স্বাভাবিক (0 dB)" } else { "Neutral (0 dB)" }).small().color(colors.text_muted));
                });
                cols[2].group(|ui| {
                    ui.label(egui::RichText::new(if is_bn { "ফুটস্টেপ (২.৫-৬ kHz)" } else { "Footsteps (2-6 kHz)" }).strong().size(12.0).color(colors.accent));
                    ui.label(egui::RichText::new(format!("+{} dB Boost", config.footstep_boost_db)).small().color(colors.accent));
                });
            });

            ui.add_space(14.0);

            // Windows Sound Enhancements Launcher Button
            let win_eq_btn = egui::Button::new(
                egui::RichText::new(if is_bn { "🎛 উইন্ডোজ সাউন্ড প্যানেল ও লাউডনেস ইকুয়ালাইজার খুলুন" } else { "🎛 Open Windows Loudness Equalization Settings" })
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
    is_bn: bool,
) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(if is_bn { "উইন্ডোজ রিয়েল-টাইম অডিও মিক্সার" } else { "Active Windows Audio Sessions & Mixer" })
                    .strong()
                    .size(15.0)
                    .color(colors.text_primary),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(if is_bn { "সরাসরি এখান থেকেই যেকোনো অ্যাপের ভলিউম ও মিউট নিয়ন্ত্রণ করুন:" } else { "Adjust individual application volumes and mute states in real-time:" })
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(12.0);

            if telemetry.active_apps.is_empty() {
                ui.label(
                    egui::RichText::new(if is_bn { "বর্তমানে সাউন্ড বাজাচ্ছে এমন কোনো অ্যাপ পাওয়া যায়নি।" } else { "No active audio sessions currently playing sound." })
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
                                ui.label(egui::RichText::new(if is_bn { "[ডাকিং টার্গেট]" } else { "[Ducking Target]" }).small().color(colors.accent));
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Mute / Unmute Button
                                let mute_text = if app.is_muted {
                                    if is_bn { "🔇 আনমিউট" } else { "🔇 Unmute" }
                                } else {
                                    if is_bn { "🔊 মিউট" } else { "🔊 Mute" }
                                };
                                if ui.button(mute_text).clicked() {
                                    audio_engine.send_command(AudioEngineCommand::SetAppMute {
                                        pid: app.pid,
                                        mute: !app.is_muted,
                                    });
                                }

                                ui.add_space(8.0);

                                // Toggle Ducking Target button
                                let duck_action_text = if app.is_target_duck {
                                    if is_bn { "ডাকিং বাদ দিন" } else { "Remove Duck" }
                                } else {
                                    if is_bn { "+ ডাকিং করুন" } else { "+ Auto Duck" }
                                };
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
                            ui.label(egui::RichText::new(if is_bn { "ভলিউম:" } else { "Vol:" }).size(12.0).color(colors.text_secondary));
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
    is_bn: bool,
) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(if is_bn { "অটোমেটিক ডাকিংয়ের টার্গেট অ্যাপসমূহ" } else { "Applications to Automatically Duck" })
                    .strong()
                    .size(15.0)
                    .color(colors.text_primary),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(
                    if is_bn {
                        "গেমে সাউন্ড আসলে যেসব অ্যাপের ভলিউম অটো কমে যাবে তা সিলেক্ট করুন:"
                    } else {
                        "Select or add the applications SoundLock should silence/lower during gameplay:"
                    },
                )
                .size(12.5)
                .color(colors.text_secondary),
            );

            ui.add_space(14.0);

            // Preset Presets
            let presets = [
                "Spotify.exe",
                "Discord.exe",
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
                egui::RichText::new(if is_bn { "কাস্টম অ্যাপ যোগ করুন (.exe):" } else { "Add Custom Executable:" })
                    .strong()
                    .size(13.0)
                    .color(colors.text_primary),
            );

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut ui_state.custom_app_input);
                let add_btn = egui::Button::new(if is_bn { "+ অ্যাপ যোগ করুন" } else { "+ Add App" })
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
                    egui::RichText::new(if is_bn { "চলমান ব্যাকগ্রাউন্ড অ্যাপস থেকে এক ক্লিকে যোগ করুন:" } else { "Detected Active Apps (Click to add):" })
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
    is_bn: bool,
) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(if is_bn { "ক্ল্যাচ মোড কিবাইন্ডস (Clutch Mode Keybinds)" } else { "Clutch Mode Keybinds" })
                    .strong()
                    .size(15.0)
                    .color(colors.text_primary),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(
                    if is_bn {
                        "১v৩ বা হাড্ডাহাড্ডি ক্লাচ রাউন্ডের সময় একটি বোতাম চাপলেই ব্যাকগ্রাউন্ডের সব চ্যাট বা মিউজিক তাৎক্ষণিক বন্ধ হয়ে যাবে:"
                    } else {
                        "Instantly silences or ducks all background chatter with a single keypress during 1v3 clutch rounds:"
                    },
                )
                .size(12.5)
                .color(colors.text_secondary),
            );

            ui.add_space(16.0);

            // Clickable Hotkey Binder Box (Matching Screenshot Style)
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(if is_bn { "ইমার্জেন্সি ক্লাচ কি:" } else { "Emergency Clutch Key:" })
                        .size(13.0)
                        .color(colors.text_primary),
                );

                let key_text = if ui_state.is_recording_clutch_key {
                    if is_bn { "🔴 কিবোর্ড বা মাউসের যেকোনো কি চাপুন..." } else { "🔴 Press any key or mouse button..." }
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
                ui.label(egui::RichText::new(if is_bn { "ট্রিগার মোড:" } else { "Trigger Mode:" }).size(12.5).color(colors.text_secondary));

                if ui.radio_value(&mut config.clutch_hold_mode, true, if is_bn { "কি চেপে ধরলে (Hold)" } else { "Hold Key" }).clicked() {
                    audio_engine.update_config(config);
                    let _ = config.save();
                }

                ui.add_space(10.0);

                if ui.radio_value(&mut config.clutch_hold_mode, false, if is_bn { "একবার চাপলে অন/অফ (Toggle)" } else { "Toggle On/Off" }).clicked() {
                    audio_engine.update_config(config);
                    let _ = config.save();
                }
            });

            ui.add_space(16.0);

            // Live Clutch Status & Manual Toggle Test
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(if is_bn { "ক্লাচ স্ট্যাটাস:" } else { "Clutch Status:" }).size(13.0).color(colors.text_secondary));

                if telemetry.is_clutch_active {
                    ui.label(
                        egui::RichText::new(if is_bn { "● ক্লাচ অ্যাক্টিভ (সব চ্যাট সাইলেন্ট)" } else { "● CLUTCH ACTIVE (Chatter Silenced)" })
                            .strong()
                            .size(13.0)
                            .color(colors.accent),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(if is_bn { "○ রেডি (স্বাভাবিক)" } else { "○ Standing by (Normal)" })
                            .size(13.0)
                            .color(Color32::from_rgb(0, 240, 140)),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let clutch_toggle_btn = egui::Button::new(
                        egui::RichText::new(if is_bn { "ক্ল্যাচ মোড টেস্ট করুন" } else { "Test Clutch Mode" })
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
fn render_developer_view(ui: &mut egui::Ui, telemetry: &EngineTelemetry, colors: &ThemeColors, is_bn: bool) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(if is_bn { "ইস্পোর্টস আর্কিটেকচার ও রিয়েল-টাইম মেট্রিক্স" } else { "Esports Architecture Telemetry" })
                    .strong()
                    .size(15.0)
                    .color(colors.text_primary),
            );
            ui.add_space(12.0);

            let (tier_label, tier_color) = match telemetry.cpu_tier {
                1 => (if is_bn { "টায়ার ১: হাই-পারফরম্যান্স গেম ব্যাটল (16ms)" } else { "Tier 1: High-Performance Esports Battle (16ms)" }, colors.accent),
                2 => (if is_bn { "টায়ার ২: ব্যাকগ্রাউন্ড / লবি স্ট্যান্ডবাই (60ms)" } else { "Tier 2: Background / Lobby Standby (60ms)" }, Color32::from_rgb(0, 220, 255)),
                _ => (if is_bn { "টায়ার ৩: জিরো-পাওয়ার হাইবারনেশন (250ms, 0.00% CPU)" } else { "Tier 3: Zero-Power Hibernation (250ms, 0.00% CPU)" }, Color32::from_rgb(0, 240, 140)),
            };

            ui.label(
                egui::RichText::new(format!("{}: {}", if is_bn { "সিপিইউ শিডিউলিং মোড" } else { "Active Scheduling State" }, tier_label))
                    .strong()
                    .size(13.0)
                    .color(tier_color),
            );
            ui.add_space(6.0);
            ui.label(egui::RichText::new(format!("{}: {:.2}", if is_bn { "গেম অডিও পিক লেভেল" } else { "Game Audio Peak Level" }, telemetry.game_peak_level)).size(12.5).color(colors.text_secondary));
            ui.label(egui::RichText::new(format!("{}: {:.2}x", if is_bn { "ভলিউম মাল্টিপ্লায়ার" } else { "Volume Ducking Multiplier" }, telemetry.current_duck_multiplier)).size(12.5).color(colors.text_secondary));
            ui.label(egui::RichText::new(format!("{}: {}", if is_bn { "অ্যাক্টিভ অডিও সেশন সংখ্যা" } else { "Active Windows Audio Sessions" }, telemetry.active_apps.len())).size(12.5).color(colors.text_secondary));
        });
}

// ----------------------------------------------------
// 8. INFORMATION TAB
// ----------------------------------------------------
fn render_information_view(ui: &mut egui::Ui, colors: &ThemeColors, is_bn: bool) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("SoundLock Suite v0.1.0").strong().size(18.0).color(colors.accent));
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(if is_bn { "স্মার্ট গেম-অডিও অপটিমাইজার ও ব্যাকগ্রাউন্ড ডাক ডাকার" } else { "Esports-Grade Smart Game-Audio Ducking & Footstep Booster" })
                    .size(13.0)
                    .color(colors.text_primary),
            );
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(
                    if is_bn {
                        "১০০% পিওর রাস্টে (Rust) তৈরি এবং সরাসরি উইন্ডোজ WASAPI এর সাথে সংযুক্ত। গেমের FPS একটুও না কমিয়ে মিলি-সেকেন্ডে ব্যাকগ্রাউন্ড নয়েজ কমায়।"
                    } else {
                        "Engineered 100% in pure Rust with native WASAPI integration for sub-millisecond reaction and zero game FPS drop."
                    },
                )
                .size(12.5)
                .color(colors.text_secondary),
            );
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
    // 1. Check egui keyboard events
    let mut detected: Option<String> = None;

    ctx.input(|i| {
        for event in &i.raw.events {
            if let egui::Event::Key { key, pressed: true, .. } = event {
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

    // 2. Check native Windows special keys (Caps Lock, Mouse 4, Mouse 5, Alt, Shift, Ctrl)
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
