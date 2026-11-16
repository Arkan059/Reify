use crate::fonts;
use crate::scene::{SceneAnchor, SceneColor, SceneFont, ScenePoint, SceneRect, Shape};
use crate::state::{DecodedImage, ImageSlot, TextureCacheEntry};
use eframe::egui;
use std::collections::HashMap;

pub fn render_scene(
    painter: &egui::Painter,
    scene: &[Shape],
    origin: egui::Pos2,
    scale: f32,
    textures: &HashMap<ImageSlot, TextureCacheEntry>,
) {
    for shape in scene {
        render_shape(painter, shape, origin, scale, textures);
    }
}

fn render_shape(
    painter: &egui::Painter,
    shape: &Shape,
    origin: egui::Pos2,
    scale: f32,
    textures: &HashMap<ImageSlot, TextureCacheEntry>,
) {
    match shape {
        Shape::FilledRect {
            rect,
            corner_radius,
            color,
        } => {
            painter.rect_filled(to_screen_rect(*rect, origin, scale), *corner_radius * scale, to_color(color));
        }
        Shape::StrokedRect {
            rect,
            corner_radius,
            stroke_width,
            color,
        } => {
            painter.rect_stroke(
                to_screen_rect(*rect, origin, scale),
                *corner_radius * scale,
                egui::Stroke::new(*stroke_width * scale, to_color(color)),
                egui::StrokeKind::Middle,
            );
        }
        Shape::FilledCircle {
            center,
            radius,
            color,
        } => {
            painter.circle_filled(to_screen_point(*center, origin, scale), *radius * scale, to_color(color));
        }
        Shape::StrokedCircle {
            center,
            radius,
            stroke_width,
            color,
        } => {
            painter.circle_stroke(
                to_screen_point(*center, origin, scale),
                *radius * scale,
                egui::Stroke::new(*stroke_width * scale, to_color(color)),
            );
        }
        Shape::Line {
            start,
            end,
            stroke_width,
            color,
        } => {
            painter.line_segment(
                [to_screen_point(*start, origin, scale), to_screen_point(*end, origin, scale)],
                egui::Stroke::new(*stroke_width * scale, to_color(color)),
            );
        }
        Shape::Text {
            text,
            pos,
            size,
            color,
            anchor,
            font,
        } => {
            let font_id = match font {
                SceneFont::Pixel => egui::FontId::new(
                    *size * scale,
                    egui::FontFamily::Name(fonts::FAMILY_PIXEL.into()),
                ),
            };
            painter.text(
                to_screen_point(*pos, origin, scale),
                to_align(*anchor),
                text,
                font_id,
                to_color(color),
            );
        }
        Shape::Image {
            rect,
            image,
            opacity,
            cover,
            slot,
        } => {
            if let Some(texture_id) = textures.get(slot).map(|entry| entry.texture.id()) {
                let screen_rect = to_screen_rect(*rect, origin, scale);
                let tint = egui::Color32::from_white_alpha(
                    ((*opacity * 255.0).round() as u8).clamp(0, 255),
                );
                let uv = if *cover {
                    cover_uv(image, screen_rect)
                } else {
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
                };
                painter.image(texture_id, screen_rect, uv, tint);
            }
        }
    }
}

fn to_screen_point(point: ScenePoint, origin: egui::Pos2, scale: f32) -> egui::Pos2 {
    origin + egui::vec2(point.x * scale, point.y * scale)
}

fn to_screen_rect(rect: SceneRect, origin: egui::Pos2, scale: f32) -> egui::Rect {
    egui::Rect::from_min_size(
        origin + egui::vec2(rect.x * scale, rect.y * scale),
        egui::vec2(rect.w * scale, rect.h * scale),
    )
}

fn to_color(color: &SceneColor) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

fn to_align(anchor: SceneAnchor) -> egui::Align2 {
    match anchor {
        SceneAnchor::TopCenter => egui::Align2::CENTER_TOP,
        SceneAnchor::CenterLeft => egui::Align2::LEFT_CENTER,
        SceneAnchor::CenterRight => egui::Align2::RIGHT_CENTER,
    }
}

fn cover_uv(image: &DecodedImage, screen_rect: egui::Rect) -> egui::Rect {
    let image_width = image.width as f32;
    let image_height = image.height as f32;
    let dest_width = screen_rect.width();
    let dest_height = screen_rect.height();
    if image_width <= 0.0 || image_height <= 0.0 || dest_width <= 0.0 || dest_height <= 0.0 {
        return egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    }
    let scale = (dest_width / image_width).max(dest_height / image_height);
    let visible_width = dest_width / scale;
    let visible_height = dest_height / scale;
    let offset_x = (image_width - visible_width) / 2.0;
    let offset_y = (image_height - visible_height) / 2.0;
    egui::Rect::from_min_max(
        egui::pos2(offset_x / image_width, offset_y / image_height),
        egui::pos2(
            (offset_x + visible_width) / image_width,
            (offset_y + visible_height) / image_height,
        ),
    )
}
