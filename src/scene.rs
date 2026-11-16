use crate::state::{AppState, CardFormat, CodePosition, DecodedImage, ImageSlot};
use eframe::egui;
use std::sync::Arc;

pub const CANVAS: f32 = 500.0;

#[derive(Clone, Copy)]
pub struct SceneRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy)]
pub struct ScenePoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy)]
pub struct SceneColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl SceneColor {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneAnchor {
    TopCenter,
    CenterLeft,
    CenterRight,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneFont {
    Pixel,
}

#[derive(Clone)]
pub enum Shape {
    FilledRect {
        rect: SceneRect,
        corner_radius: f32,
        color: SceneColor,
    },
    StrokedRect {
        rect: SceneRect,
        corner_radius: f32,
        stroke_width: f32,
        color: SceneColor,
    },
    FilledCircle {
        center: ScenePoint,
        radius: f32,
        color: SceneColor,
    },
    StrokedCircle {
        center: ScenePoint,
        radius: f32,
        stroke_width: f32,
        color: SceneColor,
    },
    Line {
        start: ScenePoint,
        end: ScenePoint,
        stroke_width: f32,
        color: SceneColor,
    },
    Text {
        text: String,
        pos: ScenePoint,
        size: f32,
        color: SceneColor,
        anchor: SceneAnchor,
        font: SceneFont,
    },
    Image {
        rect: SceneRect,
        image: Arc<DecodedImage>,
        opacity: f32,
        cover: bool,
        slot: ImageSlot,
    },
}

pub fn build_scene(state: &AppState) -> Vec<Shape> {
    match state.format {
        CardFormat::Cassette => build_cassette(state),
        CardFormat::CdCover => build_cd(state),
    }
}

pub fn code_dimensions(state: &AppState) -> Option<(f32, f32)> {
    let track_id = state.track_id.clone()?;
    let code = state.code_cache.get(&(track_id, state.code_style))?;
    let height = state.code_height;
    let width = height * (code.width as f32 / code.height as f32);
    Some((width, height))
}

pub fn code_rect_for(state: &AppState, position: CodePosition) -> Option<SceneRect> {
    let (width, height) = code_dimensions(state)?;
    Some(code_rect_with_position(
        state.format,
        position,
        state.code_offset,
        width,
        height,
    ))
}

fn code_rect_with_position(
    format: CardFormat,
    position: CodePosition,
    free_offset: egui::Vec2,
    width: f32,
    height: f32,
) -> SceneRect {
    let inset = if format == CardFormat::Cassette { 25.0 } else { 20.0 };
    match position {
        CodePosition::Free => SceneRect {
            x: free_offset.x,
            y: free_offset.y,
            w: width,
            h: height,
        },
        CodePosition::Center => SceneRect {
            x: (CANVAS - width) / 2.0,
            y: (CANVAS - height) / 2.0,
            w: width,
            h: height,
        },
        CodePosition::TopRight => SceneRect {
            x: CANVAS - inset - width,
            y: if format == CardFormat::Cassette { 111.0 } else { inset },
            w: width,
            h: height,
        },
        CodePosition::TopLeft => SceneRect {
            x: inset,
            y: if format == CardFormat::Cassette { 111.0 } else { inset },
            w: width,
            h: height,
        },
        CodePosition::BottomRight => SceneRect {
            x: CANVAS - inset - width,
            y: if format == CardFormat::Cassette {
                CANVAS - 111.0 - height
            } else {
                CANVAS - inset - height
            },
            w: width,
            h: height,
        },
        CodePosition::BottomLeft => SceneRect {
            x: inset,
            y: if format == CardFormat::Cassette {
                CANVAS - 111.0 - height
            } else {
                CANVAS - inset - height
            },
            w: width,
            h: height,
        },
        CodePosition::BottomCenter => SceneRect {
            x: (CANVAS - width) / 2.0,
            y: if format == CardFormat::Cassette {
                CANVAS * 0.68
            } else {
                CANVAS * 0.85
            },
            w: width,
            h: height,
        },
    }
}

fn build_cassette(state: &AppState) -> Vec<Shape> {
    let mut shapes = Vec::new();

    let body = SceneRect {
        x: 0.0,
        y: 91.0,
        w: 500.0,
        h: 318.0,
    };

    shapes.push(Shape::FilledRect {
        rect: SceneRect {
            x: body.x + 10.0,
            y: body.y + 10.0,
            w: body.w,
            h: body.h,
        },
        corner_radius: 0.0,
        color: SceneColor::rgba(0, 0, 0, 128),
    });

    shapes.push(Shape::FilledRect {
        rect: body,
        corner_radius: 0.0,
        color: SceneColor::rgb(0x1a, 0x1a, 0x1a),
    });
    shapes.push(Shape::StrokedRect {
        rect: body,
        corner_radius: 0.0,
        stroke_width: 2.0,
        color: SceneColor::rgb(0x33, 0x33, 0x33),
    });

    let screw_size = 10.0;
    let screw_positions = [
        ScenePoint {
            x: 8.0,
            y: body.y + 8.0,
        },
        ScenePoint {
            x: CANVAS - 8.0 - screw_size,
            y: body.y + 8.0,
        },
        ScenePoint {
            x: 8.0,
            y: body.y + body.h - 8.0 - screw_size,
        },
        ScenePoint {
            x: CANVAS - 8.0 - screw_size,
            y: body.y + body.h - 8.0 - screw_size,
        },
    ];
    for position in screw_positions {
        let rect = SceneRect {
            x: position.x,
            y: position.y,
            w: screw_size,
            h: screw_size,
        };
        shapes.push(Shape::FilledRect {
            rect,
            corner_radius: 0.0,
            color: SceneColor::rgb(0, 0, 0),
        });
        shapes.push(Shape::StrokedRect {
            rect,
            corner_radius: 0.0,
            stroke_width: 1.0,
            color: SceneColor::rgb(0x33, 0x33, 0x33),
        });
        shapes.push(Shape::Line {
            start: ScenePoint {
                x: position.x + 1.0,
                y: position.y + 1.0,
            },
            end: ScenePoint {
                x: position.x + screw_size - 1.0,
                y: position.y + screw_size - 1.0,
            },
            stroke_width: 1.0,
            color: SceneColor::rgb(0x44, 0x44, 0x44),
        });
    }

    let sticker = SceneRect {
        x: 25.0,
        y: 116.0,
        w: 450.0,
        h: 253.0,
    };
    let sticker_color = state.sticker_color;
    shapes.push(Shape::FilledRect {
        rect: sticker,
        corner_radius: 0.0,
        color: SceneColor::rgb(sticker_color[0], sticker_color[1], sticker_color[2]),
    });
    shapes.push(Shape::StrokedRect {
        rect: sticker,
        corner_radius: 0.0,
        stroke_width: 2.0,
        color: SceneColor::rgb(0, 0, 0),
    });

    let top_bar = SceneRect {
        x: sticker.x,
        y: sticker.y,
        w: sticker.w,
        h: 25.0,
    };
    shapes.push(Shape::FilledRect {
        rect: top_bar,
        corner_radius: 0.0,
        color: SceneColor::rgb(0, 0, 0),
    });

    let top_bar_center_y = top_bar.y + top_bar.h / 2.0;
    shapes.push(Shape::Text {
        text: "A-SIDE".to_owned(),
        pos: ScenePoint {
            x: sticker.x + 10.0,
            y: top_bar_center_y,
        },
        size: 19.0,
        color: SceneColor::rgb(0xff, 0xff, 0xff),
        anchor: SceneAnchor::CenterLeft,
        font: SceneFont::Pixel,
    });
    shapes.push(Shape::Text {
        text: "reify".to_owned(),
        pos: ScenePoint {
            x: sticker.x + sticker.w - 10.0,
            y: top_bar_center_y,
        },
        size: 19.0,
        color: SceneColor::rgb(0xff, 0xff, 0xff),
        anchor: SceneAnchor::CenterRight,
        font: SceneFont::Pixel,
    });

    let raw_title = state
        .track_title
        .clone()
        .unwrap_or_else(|| "UNKNOWN TRACK".to_owned())
        .to_uppercase();
    let title = truncate_title(state, &raw_title, sticker.w);
    let title_top = sticker.y + 25.0 + 9.0;
    shapes.push(Shape::Text {
        text: title,
        pos: ScenePoint {
            x: sticker.x + sticker.w / 2.0,
            y: title_top,
        },
        size: 35.0,
        color: SceneColor::rgb(0, 0, 0),
        anchor: SceneAnchor::TopCenter,
        font: SceneFont::Pixel,
    });

    let window_rect = SceneRect {
        x: 110.0,
        y: 195.0,
        w: 280.0,
        h: 80.0,
    };

    if state.use_album_art_on_cassette {
        if let Some(art) = state.album_art.as_ref() {
            shapes.push(Shape::Image {
                rect: window_rect,
                image: Arc::new(art.clone()),
                opacity: 0.6,
                cover: true,
                slot: ImageSlot::AlbumArt,
            });
        }
    }

    shapes.push(Shape::FilledRect {
        rect: window_rect,
        corner_radius: 0.0,
        color: SceneColor::rgb(0x0a, 0x0a, 0x0a),
    });
    shapes.push(Shape::StrokedRect {
        rect: window_rect,
        corner_radius: 0.0,
        stroke_width: 3.0,
        color: SceneColor::rgb(0, 0, 0),
    });

    if state.code_position != CodePosition::Center {
        let reel_radius = 27.5;
        let center_y = window_rect.y + window_rect.h / 2.0;
        let reel_centers = [
            ScenePoint {
                x: window_rect.x + 20.0 + reel_radius,
                y: center_y,
            },
            ScenePoint {
                x: window_rect.x + window_rect.w - 20.0 - reel_radius,
                y: center_y,
            },
        ];
        for center in reel_centers {
            shapes.push(Shape::FilledCircle {
                center,
                radius: reel_radius,
                color: SceneColor::rgb(0xff, 0xff, 0xff),
            });
            shapes.push(Shape::StrokedCircle {
                center,
                radius: reel_radius,
                stroke_width: 4.0,
                color: SceneColor::rgb(0, 0, 0),
            });
            shapes.push(Shape::FilledCircle {
                center,
                radius: 7.5,
                color: SceneColor::rgb(0, 0, 0),
            });
        }
    }

    let tape_width = body.w * 0.8;
    let tape = SceneRect {
        x: (CANVAS - tape_width) / 2.0,
        y: body.y + body.h - 40.0,
        w: tape_width,
        h: 40.0,
    };
    shapes.push(Shape::FilledRect {
        rect: tape,
        corner_radius: 0.0,
        color: SceneColor::rgb(0x11, 0x11, 0x11),
    });
    shapes.push(Shape::Line {
        start: ScenePoint {
            x: tape.x,
            y: tape.y,
        },
        end: ScenePoint {
            x: tape.x + tape.w,
            y: tape.y,
        },
        stroke_width: 2.0,
        color: SceneColor::rgb(0x33, 0x33, 0x33),
    });
    shapes.push(Shape::Line {
        start: ScenePoint {
            x: tape.x,
            y: tape.y,
        },
        end: ScenePoint {
            x: tape.x,
            y: tape.y + tape.h,
        },
        stroke_width: 2.0,
        color: SceneColor::rgb(0x33, 0x33, 0x33),
    });
    shapes.push(Shape::Line {
        start: ScenePoint {
            x: tape.x + tape.w,
            y: tape.y,
        },
        end: ScenePoint {
            x: tape.x + tape.w,
            y: tape.y + tape.h,
        },
        stroke_width: 2.0,
        color: SceneColor::rgb(0x33, 0x33, 0x33),
    });

    let square = 12.0;
    let gap = (tape.w - 4.0 * square) / 5.0;
    for index in 0..4 {
        let x = tape.x + gap + index as f32 * (square + gap);
        let y = tape.y + tape.h - 5.0 - square;
        shapes.push(Shape::FilledRect {
            rect: SceneRect {
                x,
                y,
                w: square,
                h: square,
            },
            corner_radius: 0.0,
            color: SceneColor::rgb(0, 0, 0),
        });
    }

    add_code_shape(state, &mut shapes);
    shapes
}

fn build_cd(state: &AppState) -> Vec<Shape> {
    let mut shapes = Vec::new();
    let rect = SceneRect {
        x: 0.0,
        y: 0.0,
        w: CANVAS,
        h: CANVAS,
    };
    if let Some(cover) = state.cover_image() {
        shapes.push(Shape::Image {
            rect,
            image: Arc::new(cover.clone()),
            opacity: 1.0,
            cover: true,
            slot: ImageSlot::Cover,
        });
    } else {
        shapes.push(Shape::FilledRect {
            rect,
            corner_radius: 0.0,
            color: SceneColor::rgb(0x11, 0x11, 0x11),
        });
    }
    shapes.push(Shape::StrokedRect {
        rect,
        corner_radius: 0.0,
        stroke_width: 1.0,
        color: SceneColor::rgb(0x33, 0x33, 0x33),
    });
    add_code_shape(state, &mut shapes);
    shapes
}

fn add_code_shape(state: &AppState, shapes: &mut Vec<Shape>) {
    let track_id = match state.track_id.clone() {
        Some(identifier) => identifier,
        None => return,
    };
    let code = match state.code_cache.get(&(track_id, state.code_style)) {
        Some(image) => image.clone(),
        None => return,
    };
    let height = state.code_height;
    let width = height * (code.width as f32 / code.height as f32);
    let rect = code_rect_with_position(
        state.format,
        state.code_position,
        state.code_offset,
        width,
        height,
    );
    shapes.push(Shape::Image {
        rect,
        image: code,
        opacity: 1.0,
        cover: false,
        slot: ImageSlot::Code,
    });
}

fn truncate_title(state: &AppState, title: &str, max_width: f32) -> String {
    match state.pixel_font.as_ref() {
        Some(font) => truncate_to_width(font, title, 35.0, max_width),
        None => title.to_owned(),
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

fn truncate_to_width(font: &fontdue::Font, text: &str, size: f32, max_width: f32) -> String {
    if measure_text_width(font, text, size) <= max_width {
        return text.to_owned();
    }
    let ellipsis = "...";
    let available = max_width - measure_text_width(font, ellipsis, size);
    if available <= 0.0 {
        return ellipsis.to_owned();
    }
    let mut result = String::new();
    let mut width = 0.0f32;
    for character in text.chars() {
        let glyph_index = font.lookup_glyph_index(character);
        let advance = font.metrics_indexed(glyph_index, size).advance_width;
        if width + advance > available {
            break;
        }
        result.push(character);
        width += advance;
    }
    result.push_str(ellipsis);
    result
}
