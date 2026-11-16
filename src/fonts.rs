use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;

pub const FAMILY_HEADING: &str = "heading";
pub const FAMILY_BODY: &str = "body";
pub const FAMILY_BODY_BOLD: &str = "body_bold";
pub const FAMILY_PIXEL: &str = "pixel";

#[derive(Clone)]
pub struct LoadedFonts {
    pub heading: Vec<u8>,
    pub body: Vec<u8>,
    pub body_bold: Vec<u8>,
    pub pixel: Vec<u8>,
}

impl LoadedFonts {
    pub fn pixel_font(&self) -> Option<fontdue::Font> {
        fontdue::Font::from_bytes(self.pixel.clone(), fontdue::FontSettings::default()).ok()
    }
}

pub fn load_fonts() -> LoadedFonts {
    let fallback = egui::FontDefinitions::default();
    LoadedFonts {
        heading: load_family(&["ariblk.ttf", "impact.ttf", "bahnschrift.ttf"], &fallback, "Ubuntu-Light"),
        body: load_family(&["segoeui.ttf", "arial.ttf"], &fallback, "Ubuntu-Light"),
        body_bold: load_family(&["segoeuib.ttf", "arialbd.ttf"], &fallback, "Ubuntu-Light"),
        pixel: load_family(&["lucon.ttf", "consola.ttf", "cour.ttf"], &fallback, "Hack"),
    }
}

fn fonts_directory() -> PathBuf {
    match std::env::var("WINDIR") {
        Ok(windir) => PathBuf::from(windir).join("Fonts"),
        Err(_) => PathBuf::from(r"C:\Windows\Fonts"),
    }
}

fn load_family(candidates: &[&str], fallback: &egui::FontDefinitions, preferred: &str) -> Vec<u8> {
    let directory = fonts_directory();
    for candidate in candidates {
        let path = directory.join(candidate);
        if let Ok(bytes) = std::fs::read(&path) {
            if fontdue::Font::from_bytes(bytes.clone(), fontdue::FontSettings::default()).is_ok() {
                return bytes;
            }
        }
    }
    bundled_font_bytes(fallback, preferred)
}

fn bundled_font_bytes(definitions: &egui::FontDefinitions, preferred: &str) -> Vec<u8> {
    let entry = definitions
        .font_data
        .get(preferred)
        .or_else(|| definitions.font_data.values().next());
    match entry {
        Some(data) => {
            let font_data: &egui::FontData = data.as_ref();
            font_data.font.as_ref().to_vec()
        }
        None => Vec::new(),
    }
}

pub fn install_fonts(ctx: &egui::Context, fonts: &LoadedFonts) {
    let mut definitions = egui::FontDefinitions::default();

    definitions.font_data.insert(
        FAMILY_HEADING.to_owned(),
        Arc::new(egui::FontData::from_owned(fonts.heading.clone())),
    );
    definitions.font_data.insert(
        FAMILY_BODY.to_owned(),
        Arc::new(egui::FontData::from_owned(fonts.body.clone())),
    );
    definitions.font_data.insert(
        FAMILY_BODY_BOLD.to_owned(),
        Arc::new(egui::FontData::from_owned(fonts.body_bold.clone())),
    );
    definitions.font_data.insert(
        FAMILY_PIXEL.to_owned(),
        Arc::new(egui::FontData::from_owned(fonts.pixel.clone())),
    );

    definitions.families.insert(
        egui::FontFamily::Name(FAMILY_HEADING.into()),
        vec![FAMILY_HEADING.to_owned()],
    );
    definitions.families.insert(
        egui::FontFamily::Name(FAMILY_BODY.into()),
        vec![FAMILY_BODY.to_owned()],
    );
    definitions.families.insert(
        egui::FontFamily::Name(FAMILY_BODY_BOLD.into()),
        vec![FAMILY_BODY_BOLD.to_owned()],
    );
    definitions.families.insert(
        egui::FontFamily::Name(FAMILY_PIXEL.into()),
        vec![FAMILY_PIXEL.to_owned()],
    );

    ctx.set_fonts(definitions);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(26.0, egui::FontFamily::Name(FAMILY_HEADING.into())),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Name(FAMILY_BODY.into())),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, egui::FontFamily::Name(FAMILY_BODY_BOLD.into())),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(11.0, egui::FontFamily::Name(FAMILY_BODY.into())),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(14.0, egui::FontFamily::Name(FAMILY_PIXEL.into())),
    );
    ctx.set_style(style);
}
