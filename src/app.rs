use crate::fonts;
use crate::render_export;
use crate::render_screen;
use crate::scene;
use crate::spotify;
use crate::state::{
    AppState, CardFormat, CodeFetchInFlight, CodePosition, CodeStyle, DecodedImage, ExportStatus,
    FetchStatus, ImageSlot, PendingTrackFetch,
};
use crate::theme;
use crate::worker::{self, WorkerCommand, WorkerResult};
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

pub struct App {
    state: AppState,
    worker_tx: Sender<WorkerCommand>,
    worker_rx: Receiver<WorkerResult>,
    export_tx: Sender<Result<(), String>>,
    export_rx: Receiver<Result<(), String>>,
}

impl App {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        let loaded_fonts = fonts::load_fonts();
        theme::apply_theme(&creation_context.egui_ctx);
        fonts::install_fonts(&creation_context.egui_ctx, &loaded_fonts);
        let (worker_tx, worker_rx) = worker::spawn(creation_context.egui_ctx.clone());
        let (export_tx, export_rx) = std::sync::mpsc::channel();
        Self {
            state: AppState::new(loaded_fonts),
            worker_tx,
            worker_rx,
            export_tx,
            export_rx,
        }
    }

    fn poll_workers(&mut self) {
        while let Ok(result) = self.worker_rx.try_recv() {
            match result {
                WorkerResult::Track { request_id, result } => {
                    if request_id == self.state.track_request_id {
                        match result {
                            Ok(info) => {
                                self.state.track_title = Some(info.title);
                                self.state.album_art = info.thumbnail;
                                self.state.album_art_revision += 1;
                                self.state.fetch_status = FetchStatus::Done;
                                self.request_code();
                            }
                            Err(_) => {
                                self.state.fetch_status = FetchStatus::Error;
                            }
                        }
                    }
                }
                WorkerResult::Code {
                    request_id,
                    track_id,
                    style,
                    result,
                } => {
                    let matches = match &self.state.code_in_flight {
                        Some(in_flight) => {
                            in_flight.request_id == request_id
                                && in_flight.track_id == track_id
                                && in_flight.style == style
                        }
                        None => false,
                    };
                    if matches {
                        self.state.code_in_flight = None;
                        if let Ok(image) = result {
                            self.state
                                .code_cache
                                .insert((track_id, style), Arc::new(image));
                        }
                    }
                }
            }
        }
        while let Ok(result) = self.export_rx.try_recv() {
            self.state.export_status = match result {
                Ok(()) => ExportStatus::Saved,
                Err(_) => ExportStatus::Failed,
            };
        }
    }

    fn dispatch_debounced(&mut self) {
        let pending = match &self.state.pending_track_fetch {
            Some(entry) if std::time::Instant::now() >= entry.dispatch_at => {
                Some((entry.request_id, entry.original_url.clone()))
            }
            _ => None,
        };
        if let Some((request_id, original_url)) = pending {
            self.state.pending_track_fetch = None;
            let _ = self.worker_tx.send(WorkerCommand::FetchTrack {
                request_id,
                original_url,
            });
            self.request_code();
        }
    }

    fn request_code(&mut self) {
        let track_id = match self.state.track_id.clone() {
            Some(identifier) => identifier,
            None => return,
        };
        let style = self.state.code_style;
        if self.state.code_cache.contains_key(&(track_id.clone(), style)) {
            return;
        }
        if let Some(in_flight) = &self.state.code_in_flight {
            if in_flight.track_id == track_id && in_flight.style == style {
                return;
            }
        }
        self.state.code_request_id += 1;
        let request_id = self.state.code_request_id;
        self.state.code_in_flight = Some(CodeFetchInFlight {
            request_id,
            track_id: track_id.clone(),
            style,
        });
        let _ = self.worker_tx.send(WorkerCommand::FetchCode {
            request_id,
            track_id,
            style,
        });
    }

    fn on_input_changed(&mut self) {
        let input = self.state.spotify_input.trim().to_string();
        match spotify::parse_track_id(&input) {
            Some(track_id) => {
                self.state.track_id = Some(track_id.clone());
                self.state.fetch_status = FetchStatus::Fetching;
                self.state.track_request_id += 1;
                let request_id = self.state.track_request_id;
                self.state.pending_track_fetch = Some(PendingTrackFetch {
                    request_id,
                    original_url: input,
                    dispatch_at: std::time::Instant::now()
                        + std::time::Duration::from_millis(400),
                });
            }
            None => {
                self.state.fetch_status = if input.is_empty() {
                    FetchStatus::Idle
                } else {
                    FetchStatus::Invalid
                };
                self.state.track_request_id += 1;
                self.state.pending_track_fetch = None;
            }
        }
    }

    fn fetch_status_line(&self) -> (String, egui::Color32) {
        match self.state.fetch_status {
            FetchStatus::Idle => (String::new(), theme::COLOR_MUTED_TEXT),
            FetchStatus::Invalid => (
                "Not a valid Spotify track link".to_owned(),
                theme::COLOR_MUTED_TEXT,
            ),
            FetchStatus::Fetching => (
                "Fetching track info...".to_owned(),
                theme::COLOR_MUTED_TEXT,
            ),
            FetchStatus::Done => ("Done".to_owned(), theme::COLOR_ACCENT),
            FetchStatus::Error => (
                "Could not fetch track info".to_owned(),
                theme::COLOR_ERROR_TEXT,
            ),
        }
    }

    fn choose_cover_image(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file();
        let Some(path) = path else {
            return;
        };
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(_) => return,
        };
        let decoded = match image::load_from_memory(&bytes) {
            Ok(decoded) => decoded,
            Err(_) => return,
        };
        let rgba = decoded.to_rgba8();
        self.state.custom_cover = Some(DecodedImage {
            width: rgba.width(),
            height: rgba.height(),
            rgba: rgba.into_raw(),
        });
        self.state.custom_cover_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        self.state.custom_cover_revision += 1;
    }

    fn on_export_clicked(&mut self, ctx: &egui::Context) {
        let path = rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .set_file_name("reify-card.png")
            .save_file();
        if let Some(path) = path {
            self.start_export(ctx, path);
        }
    }

    fn start_export(&mut self, ctx: &egui::Context, path: PathBuf) {
        self.state.export_status = ExportStatus::Exporting;
        let scene = scene::build_scene(&self.state);
        let fonts = self.state.loaded_fonts.clone();
        let scale = self.state.export_scale;
        let sender = self.export_tx.clone();
        let repaint_ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = render_export::render_and_save(&scene, &fonts, scale, &path);
            let _ = sender.send(result);
            repaint_ctx.request_repaint();
        });
    }

    fn draw_header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header")
            .exact_height(56.0)
            .frame(
                egui::Frame::default()
                    .fill(theme::COLOR_PANEL)
                    .inner_margin(egui::Margin::symmetric(20, 0)),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("reify")
                            .font(egui::FontId::new(
                                26.0,
                                egui::FontFamily::Name(fonts::FAMILY_HEADING.into()),
                            ))
                            .color(theme::COLOR_MAIN_TEXT),
                    );
                });
                let rect = ui.max_rect();
                ui.painter().rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(rect.left(), rect.bottom() - 1.0),
                        egui::pos2(rect.right(), rect.bottom()),
                    ),
                    0.0,
                    theme::COLOR_BORDER,
                );
            });
    }

    fn draw_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("controls")
            .exact_width(340.0)
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .fill(theme::COLOR_PANEL)
                    .inner_margin(egui::Margin::same(20)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.controls_ui(ui, ctx);
                });
            });
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        section_label(ui, "FORMAT");
        let format_before = self.state.format;
        egui::ComboBox::from_id_salt("format")
            .selected_text(format_label(self.state.format))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.state.format, CardFormat::Cassette, "Cassette");
                ui.selectable_value(&mut self.state.format, CardFormat::CdCover, "CD cover");
            });
        if self.state.format != format_before && self.state.code_position == CodePosition::Free {
            self.state.code_offset = default_free_offset(&self.state);
        }
        ui.add_space(16.0);

        if self.state.format == CardFormat::Cassette {
            section_label(ui, "LABEL COLOR");
            let mut color = egui::Color32::from_rgb(
                self.state.sticker_color[0],
                self.state.sticker_color[1],
                self.state.sticker_color[2],
            );
            if ui.color_edit_button_srgba(&mut color).changed() {
                self.state.sticker_color = [color.r(), color.g(), color.b()];
            }
            ui.add_space(16.0);
        }

        section_label(ui, "SPOTIFY TRACK LINK");
        let input_response = ui.add(
            egui::TextEdit::singleline(&mut self.state.spotify_input)
                .hint_text("Paste a Spotify track link")
                .desired_width(f32::INFINITY),
        );
        if input_response.changed() {
            self.on_input_changed();
        }
        let (status_text, status_color) = self.fetch_status_line();
        if !status_text.is_empty() {
            ui.label(
                egui::RichText::new(status_text)
                    .font(egui::FontId::new(
                        12.0,
                        egui::FontFamily::Name(fonts::FAMILY_BODY.into()),
                    ))
                    .color(status_color),
            );
        }
        ui.add_space(16.0);

        if self.state.format == CardFormat::CdCover {
            section_label(ui, "COVER IMAGE");
            if ui.add(egui::Button::new("Choose image...")).clicked() {
                self.choose_cover_image();
            }
            if ui.add(egui::Button::new("Use album art")).clicked() {
                self.state.custom_cover = None;
                self.state.custom_cover_name = None;
                self.state.custom_cover_revision += 1;
            }
            if let Some(name) = &self.state.custom_cover_name {
                ui.label(
                    egui::RichText::new(name)
                        .font(egui::FontId::new(
                            12.0,
                            egui::FontFamily::Name(fonts::FAMILY_BODY.into()),
                        ))
                        .color(theme::COLOR_MUTED_TEXT),
                );
            }
            ui.add_space(16.0);
        }

        if self.state.format == CardFormat::Cassette {
            ui.checkbox(
                &mut self.state.use_album_art_on_cassette,
                egui::RichText::new("SHOW ALBUM ART ON CASSETTE")
                    .font(egui::FontId::new(
                        12.0,
                        egui::FontFamily::Name(fonts::FAMILY_BODY_BOLD.into()),
                    ))
                    .color(theme::COLOR_MAIN_TEXT),
            );
            ui.add_space(16.0);
        }

        section_label(ui, "CODE POSITION");
        let position_before = self.state.code_position;
        egui::ComboBox::from_id_salt("code-position")
            .selected_text(position_label(self.state.code_position))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.state.code_position, CodePosition::Free, "Free (drag)");
                ui.selectable_value(&mut self.state.code_position, CodePosition::Center, "Center");
                ui.selectable_value(&mut self.state.code_position, CodePosition::TopRight, "Top right");
                ui.selectable_value(&mut self.state.code_position, CodePosition::TopLeft, "Top left");
                ui.selectable_value(
                    &mut self.state.code_position,
                    CodePosition::BottomRight,
                    "Bottom right",
                );
                ui.selectable_value(
                    &mut self.state.code_position,
                    CodePosition::BottomLeft,
                    "Bottom left",
                );
                ui.selectable_value(
                    &mut self.state.code_position,
                    CodePosition::BottomCenter,
                    "Bottom center",
                );
            });
        if self.state.code_position != position_before
            && self.state.code_position == CodePosition::Free
        {
            if let Some(rect) = scene::code_rect_for(&self.state, position_before) {
                self.state.code_offset = egui::vec2(rect.x, rect.y);
            }
        }
        ui.add_space(16.0);

        section_label(ui, "CODE STYLE");
        let style_before = self.state.code_style;
        egui::ComboBox::from_id_salt("code-style")
            .selected_text(style_label(self.state.code_style))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.state.code_style, CodeStyle::Black, "Black");
                ui.selectable_value(&mut self.state.code_style, CodeStyle::White, "White");
                ui.selectable_value(
                    &mut self.state.code_style,
                    CodeStyle::SpotifyGreen,
                    "Spotify green",
                );
                ui.selectable_value(
                    &mut self.state.code_style,
                    CodeStyle::TransparentWhite,
                    "Transparent (white bars)",
                );
                ui.selectable_value(
                    &mut self.state.code_style,
                    CodeStyle::TransparentBlack,
                    "Transparent (black bars)",
                );
            });
        if self.state.code_style != style_before {
            self.request_code();
        }
        ui.add_space(16.0);

        section_label(ui, "CODE SIZE");
        ui.add(
            egui::Slider::new(&mut self.state.code_height, 30.0..=100.0)
                .show_value(true)
                .suffix(" px"),
        );
        ui.add_space(16.0);

        section_label(ui, "EXPORT SCALE");
        egui::ComboBox::from_id_salt("export-scale")
            .selected_text(format!("{}x", self.state.export_scale))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.state.export_scale, 1, "1x");
                ui.selectable_value(&mut self.state.export_scale, 2, "2x");
                ui.selectable_value(&mut self.state.export_scale, 3, "3x");
                ui.selectable_value(&mut self.state.export_scale, 4, "4x");
            });
        ui.add_space(16.0);

        if primary_button(ui, "Export PNG").clicked() {
            self.on_export_clicked(ctx);
        }
        let export_line = match self.state.export_status {
            ExportStatus::Saved => ("Saved", theme::COLOR_MUTED_TEXT),
            ExportStatus::Failed => ("Export failed", theme::COLOR_ERROR_TEXT),
            ExportStatus::Idle | ExportStatus::Exporting => ("", theme::COLOR_MUTED_TEXT),
        };
        if !export_line.0.is_empty() {
            ui.label(
                egui::RichText::new(export_line.0)
                    .font(egui::FontId::new(
                        12.0,
                        egui::FontFamily::Name(fonts::FAMILY_BODY.into()),
                    ))
                    .color(export_line.1),
            );
        }
    }

    fn draw_preview(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(theme::COLOR_PANEL)
                    .stroke(egui::Stroke::new(1.0_f32, theme::COLOR_BORDER)),
            )
            .show(ctx, |ui| {
                let available = ui.available_rect_before_wrap().shrink(24.0);
                let scale = (available.width() / scene::CANVAS)
                    .min(available.height() / scene::CANVAS)
                    .min(1.0);
                let card_size = egui::vec2(scene::CANVAS * scale, scene::CANVAS * scale);
                let origin = available.center() - card_size / 2.0;

                let card_rect = egui::Rect::from_min_size(origin, card_size);
                ui.painter().rect_stroke(
                    card_rect,
                    0.0,
                    egui::Stroke::new(1.0_f32, theme::COLOR_BORDER),
                    egui::StrokeKind::Inside,
                );

                let scene = scene::build_scene(&self.state);
                let code_card_rect = scene.iter().find_map(|shape| match shape {
                    scene::Shape::Image {
                        slot: ImageSlot::Code,
                        rect,
                        ..
                    } => Some(*rect),
                    _ => None,
                });
                render_screen::render_scene(ui.painter(), &scene, origin, scale, &self.state.textures);

                if let Some(code_rect) = code_card_rect {
                    let screen_rect = egui::Rect::from_min_size(
                        origin + egui::vec2(code_rect.x * scale, code_rect.y * scale),
                        egui::vec2(code_rect.w * scale, code_rect.h * scale),
                    );
                    let response =
                        ui.interact(screen_rect, ui.id().with("code-drag"), egui::Sense::drag());
                    if response.drag_started() && self.state.code_position != CodePosition::Free {
                        self.state.code_position = CodePosition::Free;
                        self.state.code_offset = egui::vec2(code_rect.x, code_rect.y);
                    }
                    if response.dragged() {
                        let delta = response.drag_delta() / scale;
                        let mut offset = self.state.code_offset + delta;
                        offset.x = offset.x.clamp(0.0, scene::CANVAS - code_rect.w);
                        offset.y = offset.y.clamp(0.0, scene::CANVAS - code_rect.h);
                        self.state.code_offset = offset;
                    }
                    if response.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    } else if response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_workers();
        self.dispatch_debounced();
        self.state.update_textures(ctx);
        self.draw_header(ctx);
        self.draw_side_panel(ctx);
        self.draw_preview(ctx);
    }
}

fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .font(egui::FontId::new(
                12.0,
                egui::FontFamily::Name(fonts::FAMILY_BODY_BOLD.into()),
            ))
            .color(theme::COLOR_MUTED_TEXT),
    );
}

fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.weak_bg_fill = theme::COLOR_ACCENT;
        ui.visuals_mut().widgets.inactive.bg_fill = theme::COLOR_ACCENT;
        ui.visuals_mut().widgets.hovered.weak_bg_fill = theme::COLOR_ACCENT_HOVER;
        ui.visuals_mut().widgets.hovered.bg_fill = theme::COLOR_ACCENT_HOVER;
        ui.visuals_mut().widgets.active.weak_bg_fill = theme::COLOR_ACCENT_HOVER;
        ui.visuals_mut().widgets.active.bg_fill = theme::COLOR_ACCENT_HOVER;
        ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.active.bg_stroke = egui::Stroke::NONE;
        ui.visuals_mut().widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0_f32, theme::COLOR_ACCENT_TEXT);
        ui.visuals_mut().widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0_f32, theme::COLOR_ACCENT_TEXT);
        ui.visuals_mut().widgets.active.fg_stroke =
            egui::Stroke::new(1.0_f32, theme::COLOR_ACCENT_TEXT);
        ui.visuals_mut().widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, theme::COLOR_ACCENT_TEXT);
        let button = egui::Button::new(
            egui::RichText::new(text)
                .color(theme::COLOR_ACCENT_TEXT)
                .font(egui::FontId::new(
                    14.0,
                    egui::FontFamily::Name(fonts::FAMILY_BODY_BOLD.into()),
                )),
        );
        ui.add_sized([ui.available_width(), 44.0], button)
    })
    .inner
}

fn default_free_offset(state: &AppState) -> egui::Vec2 {
    let (width, height) = scene::code_dimensions(state).unwrap_or((0.0, 0.0));
    match state.format {
        CardFormat::Cassette => egui::vec2(
            scene::CANVAS - 25.0 - width,
            scene::CANVAS - 111.0 - height,
        ),
        CardFormat::CdCover => egui::vec2(
            scene::CANVAS - 20.0 - width,
            scene::CANVAS - 20.0 - height,
        ),
    }
}

fn format_label(format: CardFormat) -> &'static str {
    match format {
        CardFormat::Cassette => "Cassette",
        CardFormat::CdCover => "CD cover",
    }
}

fn position_label(position: CodePosition) -> &'static str {
    match position {
        CodePosition::Free => "Free (drag)",
        CodePosition::Center => "Center",
        CodePosition::TopRight => "Top right",
        CodePosition::TopLeft => "Top left",
        CodePosition::BottomRight => "Bottom right",
        CodePosition::BottomLeft => "Bottom left",
        CodePosition::BottomCenter => "Bottom center",
    }
}

fn style_label(style: CodeStyle) -> &'static str {
    match style {
        CodeStyle::Black => "Black",
        CodeStyle::White => "White",
        CodeStyle::SpotifyGreen => "Spotify green",
        CodeStyle::TransparentWhite => "Transparent (white bars)",
        CodeStyle::TransparentBlack => "Transparent (black bars)",
    }
}
