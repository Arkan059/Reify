use crate::fonts::LoadedFonts;
use eframe::egui;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CardFormat {
    Cassette,
    CdCover,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CodePosition {
    Free,
    Center,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    BottomCenter,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CodeStyle {
    Black,
    White,
    SpotifyGreen,
    TransparentWhite,
    TransparentBlack,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FetchStatus {
    Idle,
    Invalid,
    Fetching,
    Done,
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExportStatus {
    Idle,
    Exporting,
    Saved,
    Failed,
}

#[derive(Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ImageSlot {
    Cover,
    AlbumArt,
    Code,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TextureKey {
    Album(u64),
    Custom(u64),
    Code(String, CodeStyle),
}

pub struct TextureCacheEntry {
    pub key: TextureKey,
    pub texture: egui::TextureHandle,
}

pub struct PendingTrackFetch {
    pub request_id: u64,
    pub original_url: String,
    pub dispatch_at: Instant,
}

pub struct CodeFetchInFlight {
    pub request_id: u64,
    pub track_id: String,
    pub style: CodeStyle,
}

pub struct AppState {
    pub format: CardFormat,
    pub sticker_color: [u8; 3],
    pub spotify_input: String,
    pub track_id: Option<String>,
    pub track_title: Option<String>,
    pub album_art: Option<DecodedImage>,
    pub custom_cover: Option<DecodedImage>,
    pub custom_cover_name: Option<String>,
    pub use_album_art_on_cassette: bool,
    pub code_position: CodePosition,
    pub code_style: CodeStyle,
    pub code_height: f32,
    pub export_scale: u32,
    pub code_offset: egui::Vec2,
    pub fetch_status: FetchStatus,
    pub export_status: ExportStatus,
    pub loaded_fonts: LoadedFonts,
    pub track_request_id: u64,
    pub code_request_id: u64,
    pub pending_track_fetch: Option<PendingTrackFetch>,
    pub code_in_flight: Option<CodeFetchInFlight>,
    pub code_cache: HashMap<(String, CodeStyle), Arc<DecodedImage>>,
    pub album_art_revision: u64,
    pub custom_cover_revision: u64,
    pub textures: HashMap<ImageSlot, TextureCacheEntry>,
    pub pixel_font: Option<fontdue::Font>,
}

impl AppState {
    pub fn new(loaded_fonts: LoadedFonts) -> Self {
        let pixel_font = loaded_fonts.pixel_font();
        Self {
            format: CardFormat::Cassette,
            sticker_color: [0xd4, 0xd4, 0xd4],
            spotify_input: String::new(),
            track_id: None,
            track_title: None,
            album_art: None,
            custom_cover: None,
            custom_cover_name: None,
            use_album_art_on_cassette: false,
            code_position: CodePosition::BottomRight,
            code_style: CodeStyle::Black,
            code_height: 50.0,
            export_scale: 2,
            code_offset: egui::Vec2::ZERO,
            fetch_status: FetchStatus::Idle,
            export_status: ExportStatus::Idle,
            loaded_fonts,
            track_request_id: 0,
            code_request_id: 0,
            pending_track_fetch: None,
            code_in_flight: None,
            code_cache: HashMap::new(),
            album_art_revision: 0,
            custom_cover_revision: 0,
            textures: HashMap::new(),
            pixel_font,
        }
    }

    pub fn cover_image(&self) -> Option<&DecodedImage> {
        self.custom_cover.as_ref().or(self.album_art.as_ref())
    }

    pub fn update_textures(&mut self, ctx: &egui::Context) {
        let cover_key = self.cover_texture_key();
        let cover_image = self.cover_image().map(|image| Arc::new(image.clone()));
        let album_key = self.album_texture_key();
        let album_image = self.album_art.as_ref().map(|image| Arc::new(image.clone()));
        let code_key = self.code_texture_key();
        let code_image = self.code_image();

        self.refresh_texture(ctx, ImageSlot::Cover, cover_key, cover_image);
        self.refresh_texture(ctx, ImageSlot::AlbumArt, album_key, album_image);
        self.refresh_texture(ctx, ImageSlot::Code, code_key, code_image);
    }

    fn cover_texture_key(&self) -> Option<TextureKey> {
        if self.custom_cover.is_some() {
            Some(TextureKey::Custom(self.custom_cover_revision))
        } else if self.album_art.is_some() {
            Some(TextureKey::Album(self.album_art_revision))
        } else {
            None
        }
    }

    fn album_texture_key(&self) -> Option<TextureKey> {
        self.album_art
            .as_ref()
            .map(|_| TextureKey::Album(self.album_art_revision))
    }

    fn code_texture_key(&self) -> Option<TextureKey> {
        let track_id = self.track_id.clone()?;
        if self.code_cache.contains_key(&(track_id.clone(), self.code_style)) {
            Some(TextureKey::Code(track_id, self.code_style))
        } else {
            None
        }
    }

    fn code_image(&self) -> Option<Arc<DecodedImage>> {
        let track_id = self.track_id.clone()?;
        self.code_cache.get(&(track_id, self.code_style)).cloned()
    }

    fn refresh_texture(
        &mut self,
        ctx: &egui::Context,
        slot: ImageSlot,
        key: Option<TextureKey>,
        image: Option<Arc<DecodedImage>>,
    ) {
        match key {
            None => {
                self.textures.remove(&slot);
            }
            Some(key) => {
                let needs_upload = match self.textures.get(&slot) {
                    Some(entry) => entry.key != key,
                    None => true,
                };
                if needs_upload {
                    if let Some(image) = image {
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [image.width as usize, image.height as usize],
                            &image.rgba,
                        );
                        let texture = ctx.load_texture(
                            format!("reify-{slot:?}"),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        self.textures.insert(slot, TextureCacheEntry { key, texture });
                    }
                }
            }
        }
    }
}
