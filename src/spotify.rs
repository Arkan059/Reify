use crate::code_image;
use crate::state::{CodeStyle, DecodedImage};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::io::Read;
use std::sync::Arc;
use std::time::Duration;

const OEMBED_URL: &str = "https://open.spotify.com/oembed";
const SCANNABLES_URL: &str = "https://scannables.scdn.co/uri/plain/png";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub struct TrackInfo {
    pub title: String,
    pub thumbnail: Option<DecodedImage>,
}

#[derive(serde::Deserialize)]
struct OEmbed {
    title: Option<String>,
    thumbnail_url: Option<String>,
}

pub fn parse_track_id(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    if let Some(rest) = input.strip_prefix("spotify:track:") {
        let identifier: String = rest
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric())
            .collect();
        if identifier.is_empty() {
            return None;
        }
        return Some(identifier);
    }
    if input.starts_with("https://") || input.starts_with("http://") {
        if let Some(position) = input.find("/track/") {
            let rest = &input[position + "/track/".len()..];
            let identifier: String = rest
                .chars()
                .take_while(|character| character.is_ascii_alphanumeric())
                .collect();
            if !identifier.is_empty() {
                return Some(identifier);
            }
        }
    }
    None
}

pub fn fetch_track(original_url: &str) -> Result<TrackInfo, String> {
    let encoded = utf8_percent_encode(original_url, NON_ALPHANUMERIC).to_string();
    let url = format!("{OEMBED_URL}?url={encoded}");
    let agent = build_agent()?;
    let response = agent.get(&url).call().map_err(|error| error.to_string())?;
    if response.status() != 200 {
        return Err(format!("HTTP status {}", response.status()));
    }
    let body = response.into_string().map_err(|error| error.to_string())?;
    let parsed: OEmbed = serde_json::from_str(&body).map_err(|error| error.to_string())?;
    let title = parsed.title.unwrap_or_else(|| "UNKNOWN TRACK".to_owned());
    let thumbnail = match parsed.thumbnail_url {
        Some(thumbnail_url) => download_image(&agent, &thumbnail_url).ok(),
        None => None,
    };
    Ok(TrackInfo { title, thumbnail })
}

pub fn fetch_code_image(track_id: &str, style: CodeStyle) -> Result<DecodedImage, String> {
    let (background, bars) = match style {
        CodeStyle::Black => ("000000", "white"),
        CodeStyle::White => ("ffffff", "black"),
        CodeStyle::SpotifyGreen => ("1ed760", "black"),
        CodeStyle::TransparentWhite => ("000000", "white"),
        CodeStyle::TransparentBlack => ("ffffff", "black"),
    };
    let url = format!(
        "{SCANNABLES_URL}/{background}/{bars}/640/spotify:track:{track_id}"
    );
    let agent = build_agent()?;
    let image = download_image(&agent, &url)?;
    match style {
        CodeStyle::TransparentWhite => Ok(code_image::make_transparent(image, true)),
        CodeStyle::TransparentBlack => Ok(code_image::make_transparent(image, false)),
        _ => Ok(image),
    }
}

fn build_agent() -> Result<ureq::Agent, String> {
    let tls_connector = ureq::native_tls::TlsConnector::new()
        .map_err(|error| error.to_string())?;
    Ok(ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(8))
        .timeout_read(Duration::from_secs(8))
        .user_agent(USER_AGENT)
        .tls_connector(Arc::new(tls_connector))
        .build())
}

fn download_image(agent: &ureq::Agent, url: &str) -> Result<DecodedImage, String> {
    let response = agent.get(url).call().map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    let decoded = image::load_from_memory(&bytes).map_err(|error| error.to_string())?;
    let rgba = decoded.to_rgba8();
    Ok(DecodedImage {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    })
}
