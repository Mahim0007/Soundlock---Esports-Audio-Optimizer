use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use crate::config::AppConfig;
use crate::audio_engine::AudioEngine;
use crate::theme::{self, ThemeColors};

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

pub fn render_ui(
    ctx: &egui::Context,
    config: &mut AppConfig,
    active_tab: &mut Tab,
    audio_engine: &AudioEngine,
    should_quit: &mut bool,
) {
    let colors = ThemeColors::new(&config.accent_hex);
    theme::apply_theme(ctx, &colors);

    let telemetry = audio_engine.telemetry.lock().unwrap().clone();

    // ----------------------------------------------------
    // LEFT SIDEBAR (Matching Screenshot: Sleek Dark Sidebar)
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
            // Sidebar Navigation Items
            render_sidebar_tab(ui, active_tab, Tab::General, "General", SidebarIcon::Gear, &colors);
            render_sidebar_tab(ui, active_tab, Tab::SmartDucking, "Smart Ducking", SidebarIcon::Toolbar, &colors);
            render_sidebar_tab(ui, active_tab, Tab::FootstepBooster, "Footstep Booster", SidebarIcon::Window, &colors);
            render_sidebar_tab(ui, active_tab, Tab::AppMixer, "App Mixer", SidebarIcon::Dock, &colors);
            render_sidebar_tab(ui, active_tab, Tab::SpecificApps, "Specific Apps", SidebarIcon::Target, &colors);
            render_sidebar_tab(ui, active_tab, Tab::Shortcuts, "Shortcuts", SidebarIcon::Shortcuts, &colors);
            render_sidebar_tab(ui, active_tab, Tab::Developer, "Developer", SidebarIcon::Developer, &colors);

            // Bottom Information Tab (matching screenshot)
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(4.0);
                render_sidebar_tab(ui, active_tab, Tab::Information, "Information", SidebarIcon::Info, &colors);
            });
        });

    // ----------------------------------------------------
    // MAIN CONTENT PANEL (Matching Screenshot Layout)
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
                    Tab::General => "General",
                    Tab::SmartDucking => "Smart Ducking",
                    Tab::FootstepBooster => "Footstep Booster",
                    Tab::AppMixer => "App Mixer",
                    Tab::SpecificApps => "Specific Apps",
                    Tab::Shortcuts => "Shortcuts",
                    Tab::Developer => "Developer & Telemetry",
                    Tab::Information => "Information",
                };

                ui.label(
                    egui::RichText::new(tab_title)
                        .strong()
                        .size(16.0)
                        .color(colors.text_primary),
                );

                // Right aligned action buttons matching screenshot
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // "Quit" button with accent color
                    let quit_btn = egui::Button::new(
                        egui::RichText::new("Quit")
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

                    // "Cancel" button
                    let cancel_btn = egui::Button::new(
                        egui::RichText::new("Cancel")
                            .size(13.0)
                            .color(colors.text_secondary),
                    )
                    .fill(Color32::from_rgb(32, 36, 50))
                    .rounding(egui::Rounding::same(6.0));

                    if ui.add(cancel_btn).clicked() {
                        // Reset or minimize
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });

            ui.add_space(14.0);

            // RENDER SELECTED TAB
            match *active_tab {
                Tab::General => render_general_view(ui, config, &colors, &telemetry),
                Tab::SmartDucking => render_ducking_view(ui, config, &colors, &telemetry),
                Tab::FootstepBooster => render_footstep_view(ui, config, &colors),
                Tab::AppMixer => render_mixer_view(ui, &telemetry, &colors),
                Tab::SpecificApps => render_specific_apps_view(ui, config, &colors),
                Tab::Shortcuts => render_shortcuts_view(ui, config, &colors),
                Tab::Developer => render_developer_view(ui, &telemetry, &colors),
                Tab::Information => render_information_view(ui, &colors),
            }
        });
}

// ----------------------------------------------------
// 1. GENERAL TAB (EXACTLY MATCHING USER'S SCREENSHOT!)
// ----------------------------------------------------
fn render_general_view(ui: &mut egui::Ui, config: &mut AppConfig, colors: &ThemeColors, telemetry: &crate::audio_engine::EngineTelemetry) {
    // CARD 1: Run on startup & Language
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Run on startup?").size(13.5).color(colors.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::toggle_switch(ui, &mut config.run_on_startup, colors.accent).changed() {
                        let _ = config.save();
                    }
                });
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Language:").size(13.5).color(colors.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_source("lang_combo")
                        .selected_text(&config.language)
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
                ui.label(egui::RichText::new("Accent Color").size(13.5).color(colors.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Small Pill showing active hex e.g. #E3008C
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

            // Horizontal swatches bar matching screenshot!
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
                        ((i as f32 * 0.8).sin() * 25.0).abs() + 10.0
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
                    ui.label(egui::RichText::new("Target Competitive Game").strong().size(14.0).color(colors.text_primary));
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(format!("Current: {}", telemetry.detected_game))
                            .size(12.0)
                            .color(if telemetry.is_game_running { Color32::from_rgb(0, 240, 140) } else { colors.text_muted }),
                    );

                    ui.add_space(10.0);

                    // Button styled like "Select Wallpaper" in screenshot
                    let action_btn = egui::Button::new(
                        egui::RichText::new("Select Game / Profile")
                            .strong()
                            .size(12.5)
                            .color(colors.text_primary),
                    )
                    .fill(Color32::from_rgb(32, 38, 52))
                    .rounding(egui::Rounding::same(6.0));

                    if ui.add(action_btn).clicked() {
                        // Switch or calibrate
                    }
                });
            });
        });
}

// ----------------------------------------------------
// 2. SMART DUCKING TAB
// ----------------------------------------------------
fn render_ducking_view(ui: &mut egui::Ui, config: &mut AppConfig, colors: &ThemeColors, telemetry: &crate::audio_engine::EngineTelemetry) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Automatic Audio Ducking Engine").strong().size(15.0).color(colors.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::toggle_switch(ui, &mut config.ducking_enabled, colors.accent).changed() {
                        let _ = config.save();
                    }
                });
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Automatically lowers Spotify, Discord, or Chrome when competitive footsteps and gunshots occur.")
                    .size(12.5)
                    .color(colors.text_secondary),
            );

            ui.add_space(16.0);

            // Duck Volume Level Slider
            ui.label(egui::RichText::new("Background App Volume when Ducked:").size(13.0).color(colors.text_primary));
            if ui.add(egui::Slider::new(&mut config.duck_level_percent, 0.0..=60.0).suffix("%").step_by(5.0)).changed() {
                let _ = config.save();
            }

            ui.add_space(12.0);

            // Trigger Sensitivity Slider
            ui.label(egui::RichText::new("Game Audio Trigger Sensitivity:").size(13.0).color(colors.text_primary));
            if ui.add(egui::Slider::new(&mut config.trigger_threshold, 0.05..=0.60).step_by(0.05)).changed() {
                let _ = config.save();
            }

            ui.add_space(12.0);

            // Live Ducking Status Badge
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Engine Status:").size(13.0).color(colors.text_secondary));
                if telemetry.is_ducking_active {
                    ui.label(egui::RichText::new("● DUCKING ACTIVE (Audio Lowered)").strong().size(13.0).color(colors.accent));
                } else {
                    ui.label(egui::RichText::new("○ Inactive (Normal Volume)").size(13.0).color(Color32::from_rgb(0, 240, 140)));
                }
            });
        });
}

// ----------------------------------------------------
// 3. FOOTSTEP BOOSTER TAB
// ----------------------------------------------------
fn render_footstep_view(ui: &mut egui::Ui, _config: &mut AppConfig, colors: &ThemeColors) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Footstep Audio Frequency Equalizer").strong().size(15.0).color(colors.text_primary));
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Amplifies high-mid frequencies (2.5 kHz – 6.0 kHz) where shoe scuffs, wood creaks, and bomb defuse clicks register.")
                    .size(12.5)
                    .color(colors.text_secondary),
            );

            ui.add_space(16.0);

            ui.columns(3, |cols| {
                cols[0].group(|ui| {
                    ui.label(egui::RichText::new("Low Rumble (Explosions)").strong().size(12.5).color(colors.text_primary));
                    ui.label(egui::RichText::new("Dampened (-4 dB)").small().color(colors.text_muted));
                });
                cols[1].group(|ui| {
                    ui.label(egui::RichText::new("Voice / Mid (Discord)").strong().size(12.5).color(colors.text_primary));
                    ui.label(egui::RichText::new("Neutral (0 dB)").small().color(colors.text_muted));
                });
                cols[2].group(|ui| {
                    ui.label(egui::RichText::new("Footsteps (2-6 kHz)").strong().size(12.5).color(colors.accent));
                    ui.label(egui::RichText::new("Boosted (+6 dB)").small().color(colors.accent));
                });
            });
        });
}

// ----------------------------------------------------
// 4. APP MIXER TAB (Live Active Audio Sessions)
// ----------------------------------------------------
fn render_mixer_view(ui: &mut egui::Ui, telemetry: &crate::audio_engine::EngineTelemetry, colors: &ThemeColors) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Active Windows Audio Sessions").strong().size(15.0).color(colors.text_primary));
            ui.add_space(10.0);

            if telemetry.active_apps.is_empty() {
                ui.label(egui::RichText::new("No active audio sessions playing sound.").size(12.5).color(colors.text_muted));
            } else {
                for app in &telemetry.active_apps {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&app.name).strong().size(13.0).color(colors.text_primary));
                        if app.is_target_duck {
                            ui.label(egui::RichText::new("[Ducking Target]").small().color(colors.accent));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("{:.0}%", app.volume * 100.0));
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
fn render_specific_apps_view(ui: &mut egui::Ui, config: &mut AppConfig, colors: &ThemeColors) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Applications to Automatically Duck").strong().size(15.0).color(colors.text_primary));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Check the applications you want SoundLock to lower during combat:").size(12.5).color(colors.text_secondary));

            ui.add_space(14.0);

            let presets = ["Spotify.exe", "Discord.exe", "chrome.exe", "brave.exe", "msedge.exe", "vlc.exe"];
            for app_name in presets.iter() {
                let mut is_included = config.target_duck_apps.iter().any(|a| a.eq_ignore_ascii_case(app_name));
                if ui.checkbox(&mut is_included, *app_name).changed() {
                    if is_included {
                        if !config.target_duck_apps.contains(&app_name.to_string()) {
                            config.target_duck_apps.push(app_name.to_string());
                        }
                    } else {
                        config.target_duck_apps.retain(|a| !a.eq_ignore_ascii_case(app_name));
                    }
                    let _ = config.save();
                }
            }
        });
}

// ----------------------------------------------------
// 6. SHORTCUTS & CLUTCH TAB
// ----------------------------------------------------
fn render_shortcuts_view(ui: &mut egui::Ui, config: &mut AppConfig, colors: &ThemeColors) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Clutch Mode Keybinds").strong().size(15.0).color(colors.text_primary));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Instantly silences or ducks all background chatter with a single keypress during 1v3 clutch rounds:").size(12.5).color(colors.text_secondary));

            ui.add_space(14.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Emergency Clutch Key:").size(13.0).color(colors.text_primary));
                ui.text_edit_singleline(&mut config.clutch_hotkey);
            });
        });
}

// ----------------------------------------------------
// 7. DEVELOPER & TELEMETRY TAB (0.00% CPU Benchmark)
// ----------------------------------------------------
fn render_developer_view(ui: &mut egui::Ui, telemetry: &crate::audio_engine::EngineTelemetry, colors: &ThemeColors) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Esports Architecture Telemetry").strong().size(15.0).color(colors.text_primary));
            ui.add_space(12.0);

            let (tier_label, tier_color) = match telemetry.cpu_tier {
                1 => ("Tier 1: High-Performance Esports Battle (16ms)", colors.accent),
                2 => ("Tier 2: Background / Lobby Standby (100ms)", Color32::from_rgb(0, 220, 255)),
                _ => ("Tier 3: Zero-Power Hibernation (500ms, 0.00% CPU)", Color32::from_rgb(0, 240, 140)),
            };

            ui.label(egui::RichText::new(format!("Active Scheduling State: {}", tier_label)).strong().size(13.0).color(tier_color));
            ui.add_space(6.0);
            ui.label(egui::RichText::new(format!("Game Audio Peak Level: {:.2}", telemetry.game_peak_level)).size(12.5).color(colors.text_secondary));
            ui.label(egui::RichText::new(format!("Volume Ducking Multiplier: {:.2}", telemetry.current_duck_multiplier)).size(12.5).color(colors.text_secondary));
        });
}

// ----------------------------------------------------
// 8. INFORMATION TAB (Matching Bottom Sidebar in Screenshot)
// ----------------------------------------------------
fn render_information_view(ui: &mut egui::Ui, colors: &ThemeColors) {
    egui::Frame::none()
        .fill(colors.card_bg)
        .stroke(Stroke::new(1.0_f32, colors.card_stroke))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("SoundLock Suite v0.1.0").strong().size(18.0).color(colors.accent));
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Esports-Grade Smart Game-Audio Ducking & Footstep Booster").size(13.0).color(colors.text_primary));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Engineered 100% in pure Rust with native WASAPI integration for sub-millisecond reaction and zero game FPS drop.").size(12.5).color(colors.text_secondary));
        });
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
        // ACTIVE PILL: Solid vibrant accent fill (like Hot Pink #E3008C in screenshot) with rounded corners!
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
            // </> code brackets
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
