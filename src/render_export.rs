use crate::fonts::LoadedFonts;
use crate::scene::{SceneAnchor, SceneColor, ScenePoint, SceneRect, Shape};
use crate::state::DecodedImage;
use std::path::Path;
use tiny_skia::{
    FillRule, FilterQuality, IntRect, LineCap, Paint, Path as SkiaPath, PathBuilder, Pixmap,
    PixmapPaint,
    Stroke, Transform,
};

pub fn render_and_save(
    scene: &[Shape],
    fonts: &LoadedFonts,
    scale: u32,
    path: &Path,
) -> Result<(), String> {
    let size = 500 * scale;
    let mut pixmap = Pixmap::new(size, size).ok_or_else(|| "failed to allocate pixmap".to_owned())?;
    let pixel_font = fontdue::Font::from_bytes(fonts.pixel.clone(), fontdue::FontSettings::default()).ok();
    for shape in scene {
        draw_shape(&mut pixmap, shape, scale as f32, pixel_font.as_ref());
    }
    let rgba = unpremultiply(&pixmap);
    image::save_buffer_with_format(
        path,
        &rgba,
        size,
        size,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .map_err(|error| error.to_string())
}

fn draw_shape(pixmap: &mut Pixmap, shape: &Shape, scale: f32, font: Option<&fontdue::Font>) {
    match shape {
        Shape::FilledRect {
            rect,
            corner_radius,
            color,
        } => {
            if let Some(path) = rounded_rect_path(*rect, *corner_radius, scale) {
                let mut paint = Paint::default();
                paint.anti_alias = true;
                paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            }
        }
        Shape::StrokedRect {
            rect,
            corner_radius,
            stroke_width,
            color,
        } => {
            if let Some(path) = rounded_rect_path(*rect, *corner_radius, scale) {
                let mut paint = Paint::default();
                paint.anti_alias = true;
                paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                let mut stroke = Stroke::default();
                stroke.width = *stroke_width * scale;
                stroke.line_cap = LineCap::Square;
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
        Shape::FilledCircle {
            center,
            radius,
            color,
        } => {
            if let Some(path) = PathBuilder::from_circle(center.x * scale, center.y * scale, *radius * scale) {
                let mut paint = Paint::default();
                paint.anti_alias = true;
                paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            }
        }
        Shape::StrokedCircle {
            center,
            radius,
            stroke_width,
            color,
        } => {
            if let Some(path) = PathBuilder::from_circle(center.x * scale, center.y * scale, *radius * scale) {
                let mut paint = Paint::default();
                paint.anti_alias = true;
                paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                let mut stroke = Stroke::default();
                stroke.width = *stroke_width * scale;
                stroke.line_cap = LineCap::Round;
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
        Shape::Line {
            start,
            end,
            stroke_width,
            color,
        } => {
            let mut builder = PathBuilder::new();
            builder.move_to(start.x * scale, start.y * scale);
            builder.line_to(end.x * scale, end.y * scale);
            if let Some(path) = builder.finish() {
                let mut paint = Paint::default();
                paint.anti_alias = true;
                paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                let mut stroke = Stroke::default();
                stroke.width = *stroke_width * scale;
                stroke.line_cap = LineCap::Square;
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
        Shape::Text {
            text,
            pos,
            size,
            color,
            anchor,
            ..
        } => {
            if let Some(font) = font {
                draw_text(pixmap, font, text, pos, *size * scale, *color, *anchor);
            }
        }
        Shape::Image {
            rect,
            image,
            opacity,
            cover,
            ..
        } => {
            draw_image(pixmap, image, *rect, scale, *opacity, *cover);
        }
    }
}

fn rounded_rect_path(rect: SceneRect, radius: f32, scale: f32) -> Option<SkiaPath> {
    let x = rect.x * scale;
    let y = rect.y * scale;
    let w = rect.w * scale;
    let h = rect.h * scale;
    let r = (radius * scale).clamp(0.0, (w.min(h)) / 2.0);
    let mut builder = PathBuilder::new();
    if r <= 0.0 {
        builder.move_to(x, y);
        builder.line_to(x + w, y);
        builder.line_to(x + w, y + h);
        builder.line_to(x, y + h);
        builder.close();
    } else {
        builder.move_to(x + r, y);
        builder.line_to(x + w - r, y);
        builder.quad_to(x + w, y, x + w, y + r);
        builder.line_to(x + w, y + h - r);
        builder.quad_to(x + w, y + h, x + w - r, y + h);
        builder.line_to(x + r, y + h);
        builder.quad_to(x, y + h, x, y + h - r);
        builder.line_to(x, y + r);
        builder.quad_to(x, y, x + r, y);
        builder.close();
    }
    builder.finish()
}

fn draw_text(
    pixmap: &mut Pixmap,
    font: &fontdue::Font,
    text: &str,
    pos: &ScenePoint,
    size: f32,
    color: SceneColor,
    anchor: SceneAnchor,
) {
    let line = match font.horizontal_line_metrics(size) {
        Some(line) => line,
        None => return,
    };
    let width = measure_text_width(font, text, size);
    let ascent = line.ascent;
    let descent = line.descent;
    let baseline_y = match anchor {
        SceneAnchor::TopCenter => pos.y + ascent,
        SceneAnchor::CenterLeft | SceneAnchor::CenterRight => pos.y + (ascent + descent) / 2.0,
    };
    let pen_x = match anchor {
        SceneAnchor::CenterLeft => pos.x,
        SceneAnchor::TopCenter => pos.x - width / 2.0,
        SceneAnchor::CenterRight => pos.x - width,
    };
    let mut cursor = pen_x;
    for character in text.chars() {
        let glyph_index = font.lookup_glyph_index(character);
        let (metrics, coverage) = font.rasterize_indexed(glyph_index, size);
        let draw_x = (cursor + metrics.xmin as f32).round() as i32;
        let draw_y = (baseline_y + metrics.ymin as f32).round() as i32;
        blend_coverage(
            pixmap,
            &coverage,
            metrics.width,
            metrics.height,
            draw_x,
            draw_y,
            color,
        );
        cursor += metrics.advance_width;
    }
}

fn measure_text_width(font: &fontdue::Font, text: &str, size: f32) -> f32 {
    let mut width = 0.0f32;
    for character in text.chars() {
        let glyph_index = font.lookup_glyph_index(character);
        width += font.metrics_indexed(glyph_index, size).advance_width;
    }
    width
}

fn blend_coverage(
    pixmap: &mut Pixmap,
    coverage: &[u8],
    width: usize,
    height: usize,
    draw_x: i32,
    draw_y: i32,
    color: SceneColor,
) {
    let pixmap_width = pixmap.width() as i32;
    let pixmap_height = pixmap.height() as i32;
    let data = pixmap.data_mut();
    let source_red = color.r as f32 / 255.0;
    let source_green = color.g as f32 / 255.0;
    let source_blue = color.b as f32 / 255.0;
    let color_alpha = color.a as f32 / 255.0;
    for y in 0..height {
        for x in 0..width {
            let coverage_value = coverage[y * width + x];
            if coverage_value == 0 {
                continue;
            }
            let pixel_x = draw_x + x as i32;
            let pixel_y = draw_y + y as i32;
            if pixel_x < 0 || pixel_y < 0 || pixel_x >= pixmap_width || pixel_y >= pixmap_height {
                continue;
            }
            let index = (pixel_y * pixmap_width + pixel_x) as usize * 4;
            let source_alpha = (coverage_value as f32 / 255.0) * color_alpha;
            let source_premul_red = source_red * source_alpha;
            let source_premul_green = source_green * source_alpha;
            let source_premul_blue = source_blue * source_alpha;
            let dest_premul_red = data[index] as f32 / 255.0;
            let dest_premul_green = data[index + 1] as f32 / 255.0;
            let dest_premul_blue = data[index + 2] as f32 / 255.0;
            let dest_alpha = data[index + 3] as f32 / 255.0;
            let out_alpha = source_alpha + dest_alpha * (1.0 - source_alpha);
            data[index] = ((source_premul_red + dest_premul_red * (1.0 - source_alpha)) * 255.0).round() as u8;
            data[index + 1] = ((source_premul_green + dest_premul_green * (1.0 - source_alpha)) * 255.0).round() as u8;
            data[index + 2] = ((source_premul_blue + dest_premul_blue * (1.0 - source_alpha)) * 255.0).round() as u8;
            data[index + 3] = (out_alpha * 255.0).round() as u8;
        }
    }
}

fn draw_image(
    pixmap: &mut Pixmap,
    image: &DecodedImage,
    rect: SceneRect,
    scale: f32,
    opacity: f32,
    cover: bool,
) {
    let source = match decoded_to_pixmap(image) {
        Some(source) => source,
        None => return,
    };
    let dest_x = (rect.x * scale).round() as i32;
    let dest_y = (rect.y * scale).round() as i32;
    let dest_w = rect.w * scale;
    let dest_h = rect.h * scale;
    let mut paint = PixmapPaint::default();
    paint.quality = FilterQuality::Bilinear;
    paint.opacity = opacity.clamp(0.0, 1.0);
    if cover {
        let cropped = match cover_crop(&source, dest_w, dest_h) {
            Some(cropped) => cropped,
            None => return,
        };
        let scale_x = dest_w / cropped.width() as f32;
        let scale_y = dest_h / cropped.height() as f32;
        pixmap.draw_pixmap(
            dest_x,
            dest_y,
            cropped.as_ref(),
            &paint,
            Transform::from_scale(scale_x, scale_y),
            None,
        );
    } else {
        let scale_x = dest_w / source.width() as f32;
        let scale_y = dest_h / source.height() as f32;
        pixmap.draw_pixmap(
            dest_x,
            dest_y,
            source.as_ref(),
            &paint,
            Transform::from_scale(scale_x, scale_y),
            None,
        );
    }
}

fn cover_crop(source: &Pixmap, dest_w: f32, dest_h: f32) -> Option<Pixmap> {
    let source_w = source.width() as f32;
    let source_h = source.height() as f32;
    if source_w <= 0.0 || source_h <= 0.0 {
        return None;
    }
    let scale = (dest_w / source_w).max(dest_h / source_h);
    let crop_w = ((dest_w / scale).round() as u32).min(source.width());
    let crop_h = ((dest_h / scale).round() as u32).min(source.height());
    let offset_x = ((source_w - crop_w as f32) / 2.0).round() as i32;
    let offset_y = ((source_h - crop_h as f32) / 2.0).round() as i32;
    let int_rect = IntRect::from_xywh(offset_x, offset_y, crop_w, crop_h)?;
    source.clone_rect(int_rect)
}

fn decoded_to_pixmap(image: &DecodedImage) -> Option<Pixmap> {
    let mut pixmap = Pixmap::new(image.width, image.height)?;
    let data = pixmap.data_mut();
    for (index, pixel) in image.rgba.chunks_exact(4).enumerate() {
        let alpha = pixel[3] as u32;
        let red = pixel[0] as u32;
        let green = pixel[1] as u32;
        let blue = pixel[2] as u32;
        data[index * 4] = ((red * alpha + 127) / 255) as u8;
        data[index * 4 + 1] = ((green * alpha + 127) / 255) as u8;
        data[index * 4 + 2] = ((blue * alpha + 127) / 255) as u8;
        data[index * 4 + 3] = alpha as u8;
    }
    Some(pixmap)
}

fn unpremultiply(pixmap: &Pixmap) -> Vec<u8> {
    let data = pixmap.data();
    let mut out = vec![0u8; data.len()];
    for (index, chunk) in data.chunks_exact(4).enumerate() {
        let alpha = chunk[3] as u32;
        if alpha == 0 {
            continue;
        }
        out[index * 4] = ((chunk[0] as u32 * 255 + alpha / 2) / alpha) as u8;
        out[index * 4 + 1] = ((chunk[1] as u32 * 255 + alpha / 2) / alpha) as u8;
        out[index * 4 + 2] = ((chunk[2] as u32 * 255 + alpha / 2) / alpha) as u8;
        out[index * 4 + 3] = alpha as u8;
    }
    out
}
