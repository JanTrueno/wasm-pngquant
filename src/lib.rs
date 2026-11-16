use wasm_bindgen::prelude::*;
use imagequant::new;
use rgb::FromSlice;

/// The original entry point, kept so existing callers don't break:
/// pngquant's usual `--quality 60-80`, speed 3, full dithering.
#[wasm_bindgen]
pub fn compress_png_data(image_data: &[u8], width: u32, height: u32) -> Vec<u8> {
    quantize_to_png(image_data, width, height, 60, 80, 1.0, 3)
        .unwrap_or_else(|_| wasm_bindgen::throw_str("quantisation failed"))
}

/// Quantise RGBA pixels to a palette PNG with the knobs exposed.
///
/// - `quality_min` / `quality_max`: 0-100, same meaning as pngquant's
///   `--quality min-max`. If the result can't reach `quality_min` this
///   throws, exactly like pngquant exiting with status 99; pass 0 to never
///   fail.
/// - `dither`: 0.0 (off) to 1.0 (full Floyd-Steinberg).
/// - `speed`: 1 (slowest, best) to 10 (fastest).
#[wasm_bindgen]
pub fn compress_png(
    image_data: &[u8],
    width: u32,
    height: u32,
    quality_min: u8,
    quality_max: u8,
    dither: f32,
    speed: i32,
) -> Result<Vec<u8>, JsError> {
    quantize_to_png(image_data, width, height, quality_min, quality_max, dither, speed)
}

fn quantize_to_png(
    image_data: &[u8],
    width: u32,
    height: u32,
    quality_min: u8,
    quality_max: u8,
    dither: f32,
    speed: i32,
) -> Result<Vec<u8>, JsError> {
    let w = width as usize;
    let h = height as usize;
    if image_data.len() != w * h * 4 {
        return Err(JsError::new("image_data must be width * height * 4 RGBA bytes"));
    }

    let mut liq = new();
    liq.set_speed(speed.clamp(1, 10))
        .map_err(|e| JsError::new(&format!("speed: {e}")))?;
    let max = quality_max.min(100);
    liq.set_quality(quality_min.min(max), max)
        .map_err(|e| JsError::new(&format!("quality: {e}")))?;

    let rgba_pixels = image_data.as_rgba();
    let mut img = liq
        .new_image(rgba_pixels, w, h, 0.0)
        .map_err(|e| JsError::new(&format!("image: {e}")))?;

    let mut res = liq
        .quantize(&mut img)
        .map_err(|e| JsError::new(&format!("quantize: {e}")))?;
    res.set_dithering_level(dither.clamp(0.0, 1.0))
        .map_err(|e| JsError::new(&format!("dither: {e}")))?;
    let (palette, pixels) = res
        .remapped(&mut img)
        .map_err(|e| JsError::new(&format!("remap: {e}")))?;

    encode_indexed(w, h, &palette, &pixels)
}

/// Writes the smallest palette PNG it can: indices packed at 1/2/4 bits
/// when the palette is small enough, no row filter (palette indices aren't
/// numerically related to their neighbours, so filtering only hurts - libpng
/// does the same), and a tRNS chunk only when something is see-through.
fn encode_indexed(
    w: usize,
    h: usize,
    palette: &[imagequant::RGBA],
    pixels: &[u8],
) -> Result<Vec<u8>, JsError> {
    let (depth, bits) = match palette.len() {
        0..=2 => (png::BitDepth::One, 1),
        3..=4 => (png::BitDepth::Two, 2),
        5..=16 => (png::BitDepth::Four, 4),
        _ => (png::BitDepth::Eight, 8),
    };

    let data = if bits == 8 {
        pixels.to_vec()
    } else {
        let per_byte = 8 / bits;
        let row_bytes = (w + per_byte - 1) / per_byte;
        let mut packed = vec![0u8; row_bytes * h];
        for y in 0..h {
            let src = &pixels[y * w..(y + 1) * w];
            let dst = &mut packed[y * row_bytes..(y + 1) * row_bytes];
            for (x, &idx) in src.iter().enumerate() {
                let shift = 8 - bits - (x % per_byte) * bits;
                dst[x / per_byte] |= idx << shift;
            }
        }
        packed
    };

    let mut plte = Vec::with_capacity(palette.len() * 3);
    for c in palette {
        plte.extend_from_slice(&[c.r, c.g, c.b]);
    }
    // tRNS only has to run up to the last see-through entry.
    let trns_len = palette.iter().rposition(|c| c.a < 255).map_or(0, |i| i + 1);

    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, w as u32, h as u32);
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(depth);
        encoder.set_compression(png::Compression::Best);
        encoder.set_filter(png::FilterType::NoFilter);
        encoder.set_adaptive_filter(png::AdaptiveFilterType::NonAdaptive);
        encoder.set_palette(plte);
        if trns_len > 0 {
            encoder.set_trns(palette[..trns_len].iter().map(|c| c.a).collect::<Vec<u8>>());
        }
        let mut writer = encoder
            .write_header()
            .map_err(|e| JsError::new(&format!("png header: {e}")))?;
        writer
            .write_image_data(&data)
            .map_err(|e| JsError::new(&format!("png data: {e}")))?;
    }
    Ok(out)
}
