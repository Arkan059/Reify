use eframe::egui;

pub const COLOR_BACKGROUND: egui::Color32 = egui::Color32::from_rgb(0x0a, 0x0a, 0x0a);
pub const COLOR_PANEL: egui::Color32 = egui::Color32::from_rgb(0x17, 0x17, 0x17);
pub const COLOR_BORDER: egui::Color32 = egui::Color32::from_rgb(0x26, 0x26, 0x26);
pub const COLOR_MAIN_TEXT: egui::Color32 = egui::Color32::from_rgb(0xf5, 0xf5, 0xf5);
pub const COLOR_MUTED_TEXT: egui::Color32 = egui::Color32::from_rgb(0xa3, 0xa3, 0xa3);
pub const COLOR_ACCENT: egui::Color32 = egui::Color32::from_rgb(0x1e, 0xd7, 0x60);
pub const COLOR_ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(0x1d, 0xb9, 0x54);
pub const COLOR_ACCENT_TEXT: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);
pub const COLOR_ERROR_TEXT: egui::Color32 = egui::Color32::from_rgb(0xff, 0x5d, 0x5d);

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    let corner_radius = egui::CornerRadius::same(2);
    let border_stroke = egui::Stroke::new(1.0_f32, COLOR_BORDER);

    visuals.dark_mode = true;
    visuals.override_text_color = Some(COLOR_MAIN_TEXT);
    visuals.panel_fill = COLOR_PANEL;
    visuals.window_fill = COLOR_PANEL;
    visuals.extreme_bg_color = COLOR_BACKGROUND;
    visuals.faint_bg_color = COLOR_BACKGROUND;
    visuals.error_fg_color = COLOR_ERROR_TEXT;
    visuals.window_corner_radius = corner_radius;
    visuals.window_stroke = border_stroke;
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    visuals.popup_shadow = egui::epaint::Shadow::NONE;
    visuals.selection = egui::style::Selection {
        bg_fill: COLOR_ACCENT,
        stroke: egui::Stroke::new(1.0_f32, COLOR_ACCENT),
    };

    configure_widget(
        &mut visuals.widgets.noninteractive,
        border_stroke,
        corner_radius,
        COLOR_MUTED_TEXT,
        COLOR_PANEL,
    );
    configure_widget(
        &mut visuals.widgets.inactive,
        border_stroke,
        corner_radius,
        COLOR_MAIN_TEXT,
        COLOR_PANEL,
    );
    configure_widget(
        &mut visuals.widgets.hovered,
        egui::Stroke::new(1.0_f32, COLOR_ACCENT),
        corner_radius,
        COLOR_MAIN_TEXT,
        COLOR_PANEL,
    );
    configure_widget(
        &mut visuals.widgets.active,
        egui::Stroke::new(1.0_f32, COLOR_ACCENT),
        corner_radius,
        COLOR_MAIN_TEXT,
        COLOR_PANEL,
    );
    configure_widget(
        &mut visuals.widgets.open,
        egui::Stroke::new(1.0_f32, COLOR_ACCENT),
        corner_radius,
        COLOR_MAIN_TEXT,
        COLOR_PANEL,
    );

    ctx.set_visuals(visuals);
}

fn configure_widget(
    widget: &mut egui::style::WidgetVisuals,
    stroke: egui::Stroke,
    corner_radius: egui::CornerRadius,
    text_color: egui::Color32,
    fill: egui::Color32,
) {
    widget.bg_fill = fill;
    widget.weak_bg_fill = fill;
    widget.bg_stroke = stroke;
    widget.corner_radius = corner_radius;
    widget.fg_stroke = egui::Stroke::new(1.0_f32, text_color);
}
