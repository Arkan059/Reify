use crate::state::DecodedImage;

pub fn make_transparent(image: DecodedImage, white_bars: bool) -> DecodedImage {
    let mut rgba = image.rgba;
    for pixel in rgba.chunks_exact_mut(4) {
        let red = pixel[0] as u32;
        let green = pixel[1] as u32;
        let blue = pixel[2] as u32;
        let luminance = ((red * 299 + green * 587 + blue * 114) / 1000) as u8;
        let (color, alpha) = if white_bars {
            (255u8, luminance)
        } else {
            (0u8, 255u8 - luminance)
        };
        pixel[0] = color;
        pixel[1] = color;
        pixel[2] = color;
        pixel[3] = alpha;
    }
    DecodedImage {
        width: image.width,
        height: image.height,
        rgba,
    }
}
