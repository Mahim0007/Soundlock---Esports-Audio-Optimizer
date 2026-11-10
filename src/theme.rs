use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

pub struct ThemeColors {
    pub bg_dark: Color32,
    pub card_bg: Color32,
    pub card_stroke: Color32,
    pub sidebar_bg: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
}

impl ThemeColors {
    pub fn new(accent_hex: &str) -> Self {
        let accent = parse_hex_color(accent_hex).unwrap_or(Color32::from_rgb(227, 0, 140)); // #E3008C default
        let accent_hover = lighten_color(accent, 0.15);

        Self {
            bg_dark: Color32::from_rgb(18, 20, 28),
            card_bg: Color32::from_rgb(26, 30, 42),
            card_stroke: Color32::from_rgb(42, 48, 66),
            sidebar_bg: Color32::from_rgb(15, 17, 24),
            text_primary: Color32::from_rgb(245, 248, 255),
            text_secondary: Color32::from_rgb(170, 182, 205),
            text_muted: Color32::from_rgb(100, 112, 135),
            accent,
            accent_hover,
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, colors: &ThemeColors) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = colors.bg_dark;
    visuals.window_fill = colors.bg_dark;
    visuals.override_text_color = Some(colors.text_primary);

    visuals.widgets.inactive.bg_fill = colors.card_bg;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, colors.card_stroke);
    visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);

    visuals.widgets.hovered.bg_fill = lighten_color(colors.card_bg, 0.08);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.2_f32, colors.accent);
    visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);

    visuals.widgets.active.bg_fill = colors.accent;
    visuals.widgets.active.bg_stroke = Stroke::new(1.4_f32, colors.accent_hover);
    visuals.widgets.active.rounding = egui::Rounding::same(8.0);

    ctx.set_visuals(visuals);
}

/// Custom iOS / Fluent Style Animated Pill Toggle Switch matching screenshot!
pub fn toggle_switch(ui: &mut egui::Ui, value: &mut bool, accent: Color32) -> egui::Response {
    let desired_size = Vec2::new(44.0, 24.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let animation_t = ui.ctx().animate_bool(response.id, *value);

    // Track background
    let off_bg = Color32::from_rgb(45, 52, 70);
    let on_bg = accent;
    let bg = lerp_color(off_bg, on_bg, animation_t);

    ui.painter().rect_filled(rect, egui::Rounding::same(12.0), bg);
    ui.painter().rect_stroke(rect, egui::Rounding::same(12.0), Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 30)));

    // Circle knob position
    let knob_radius = 9.0;
    let left_x = rect.min.x + 3.0 + knob_radius;
    let right_x = rect.max.x - 3.0 - knob_radius;
    let current_x = left_x + (right_x - left_x) * animation_t;
    let knob_center = Pos2::new(current_x, rect.center().y);

    // Knob drop shadow
    ui.painter().circle_filled(Pos2::new(knob_center.x, knob_center.y + 1.0), knob_radius, Color32::from_rgba_unmultiplied(0, 0, 0, 60));
    // Knob fill
    ui.painter().circle_filled(knob_center, knob_radius, Color32::WHITE);

    response
}

/// Custom Slider with solid White Knob Background ("scrool button background shada")
pub fn white_knob_slider(
    ui: &mut egui::Ui,
    value: &mut f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    suffix: &str,
    decimals: usize,
    accent: Color32,
) -> egui::Response {
    let rail_width = 180.0_f32;
    let rail_height = 6.0_f32;
    let knob_radius = 8.5_f32;
    let value_pill_width = 54.0_f32;
    let total_width = rail_width + 14.0 + value_pill_width;
    let total_height = 24.0_f32;

    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(total_width, total_height), egui::Sense::click_and_drag());

    let rail_min_x = rect.min.x + knob_radius;
    let rail_max_x = rect.min.x + rail_width - knob_radius;
    let rail_len = (rail_max_x - rail_min_x).max(1.0);
    let center_y = rect.center().y;

    if response.hovered() || response.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    if response.clicked() || response.dragged() {
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            let t = ((mouse_pos.x - rail_min_x) / rail_len).clamp(0.0, 1.0);
            let mut new_val = min + t * (max - min);
            if let Some(s) = step {
                new_val = (new_val / s).round() * s;
            }
            new_val = new_val.clamp(min, max);
            if (*value - new_val).abs() > 1e-5 {
                *value = new_val;
                response.mark_changed();
            }
        }
    }

    let t = ((*value - min) / (max - min)).clamp(0.0, 1.0);
    let knob_x = rail_min_x + t * rail_len;
    let knob_center = Pos2::new(knob_x, center_y);

    let painter = ui.painter();

    // 1. Draw rail background track
    let rail_rect = egui::Rect::from_min_max(
        Pos2::new(rail_min_x, center_y - rail_height * 0.5),
        Pos2::new(rail_max_x, center_y + rail_height * 0.5),
    );
    painter.rect_filled(rail_rect, egui::Rounding::same(rail_height * 0.5), Color32::from_rgb(38, 44, 60));

    // 2. Draw active trailing fill
    if knob_x > rail_min_x {
        let active_rail = egui::Rect::from_min_max(
            Pos2::new(rail_min_x, center_y - rail_height * 0.5),
            Pos2::new(knob_x, center_y + rail_height * 0.5),
        );
        painter.rect_filled(active_rail, egui::Rounding::same(rail_height * 0.5), accent);
    }

    // 3. Draw WHITE KNOB (solid white background scroll button!)
    painter.circle_filled(
        Pos2::new(knob_center.x, knob_center.y + 1.2),
        knob_radius,
        Color32::from_rgba_unmultiplied(0, 0, 0, 80),
    );
    // PURE WHITE FILL
    painter.circle_filled(knob_center, knob_radius, Color32::WHITE);
    // Sleek border
    let border_color = if response.dragged() {
        accent
    } else if response.hovered() {
        Color32::from_rgb(255, 120, 200)
    } else {
        Color32::from_rgb(215, 222, 235)
    };
    painter.circle_stroke(knob_center, knob_radius, Stroke::new(1.8_f32, border_color));

    // 4. Value Pill on the right
    let pill_rect = egui::Rect::from_min_size(
        Pos2::new(rect.min.x + rail_width + 12.0, center_y - 11.0),
        Vec2::new(value_pill_width, 22.0),
    );
    painter.rect_filled(pill_rect, egui::Rounding::same(5.0), Color32::from_rgb(20, 24, 34));
    painter.rect_stroke(pill_rect, egui::Rounding::same(5.0), Stroke::new(1.0_f32, Color32::from_rgb(45, 52, 70)));

    let val_str = format!("{:.*}{}", decimals, *value, suffix);
    painter.text(
        pill_rect.center(),
        egui::Align2::CENTER_CENTER,
        val_str,
        egui::FontId::proportional(12.0),
        Color32::from_rgb(240, 244, 255),
    );

    response
}

/// Accent Color Palette Swatches matching screenshot
pub fn color_palette_strip(ui: &mut egui::Ui, current_hex: &mut String) -> bool {
    let swatches = [
        "#730046", "#A60064", "#D1007E", "#E3008C", "#F038A6", "#F566B9", "#FA9CD1", "#FCC2E3",
    ];

    let mut changed = false;
    let strip_rect = ui.available_rect_before_wrap();
    let swatch_width = (strip_rect.width() / swatches.len() as f32).clamp(18.0, 50.0);
    let swatch_height = 24.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(1.0, 0.0);
        for hex in swatches.iter() {
            let color = parse_hex_color(hex).unwrap_or(Color32::WHITE);
            let is_active = current_hex.eq_ignore_ascii_case(hex);

            let (rect, resp) = ui.allocate_exact_size(Vec2::new(swatch_width, swatch_height), egui::Sense::click());
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                *current_hex = hex.to_string();
                changed = true;
            }

            ui.painter().rect_filled(rect, egui::Rounding::ZERO, color);
            if is_active {
                ui.painter().rect_stroke(rect, egui::Rounding::ZERO, Stroke::new(2.0_f32, Color32::WHITE));
            }
        }
    });

    changed
}

// Helper color utilities
pub fn parse_hex_color(hex: &str) -> Option<Color32> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else {
        None
    }
}

pub fn lighten_color(c: Color32, factor: f32) -> Color32 {
    let r = (c.r() as f32 + (255.0 - c.r() as f32) * factor).clamp(0.0, 255.0) as u8;
    let g = (c.g() as f32 + (255.0 - c.g() as f32) * factor).clamp(0.0, 255.0) as u8;
    let b = (c.b() as f32 + (255.0 - c.b() as f32) * factor).clamp(0.0, 255.0) as u8;
    Color32::from_rgb(r, g, b)
}

pub fn lerp_color(c1: Color32, c2: Color32, t: f32) -> Color32 {
    let r = (c1.r() as f32 + (c2.r() as f32 - c1.r() as f32) * t) as u8;
    let g = (c1.g() as f32 + (c2.g() as f32 - c1.g() as f32) * t) as u8;
    let b = (c1.b() as f32 + (c2.b() as f32 - c1.b() as f32) * t) as u8;
    Color32::from_rgb(r, g, b)
}
