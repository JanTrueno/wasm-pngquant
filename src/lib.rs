use wasm_bindgen::prelude::*;
use imagequant::new;
use rgb::FromSlice;

#[wasm_bindgen]
pub fn compress_png_data(image_data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut liq = new();
    liq.set_speed(3).unwrap();
    liq.set_quality(60, 80).unwrap();

    let rgba_pixels = image_data.as_rgba();
    let mut img = liq.new_image(rgba_pixels, width as usize, height as usize, 0.0).unwrap();

    let mut res = liq.quantize(&mut img).unwrap();
    let (palette, pixels) = res.remapped(&mut img).unwrap();

    // NEW: Encode the results into a proper, compressed PNG file!
    let mut final_png_file = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut final_png_file, width, height);
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        
        // Convert the palette into the format the PNG encoder expects
        let mut palette_bytes = Vec::with_capacity(palette.len() * 3);
        let mut trns_bytes = Vec::with_capacity(palette.len());
        for color in palette {
            palette_bytes.push(color.r);
            palette_bytes.push(color.g);
            palette_bytes.push(color.b);
            trns_bytes.push(color.a);
        }
        encoder.set_palette(palette_bytes);
        encoder.set_trns(trns_bytes);
        
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&pixels).unwrap();
    }
    
    // Return the actual PNG file bytes to JavaScript
    final_png_file
}
