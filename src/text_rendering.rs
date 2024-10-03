/// Functions related to text-rendering
/// Fonts are rendered to an atlas with glyph location metadata using the font-kit library.
/// This code should be multi-platform thanks to font-kit!

use std::cmp::max;

use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use font_kit::canvas::{Canvas, RasterizationOptions, Format};
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{Vector2F, Vector2I};


struct PrintableChars {
    current_char: char,
}

impl PrintableChars {
    fn new() -> PrintableChars {
        PrintableChars{current_char: char::from_u32(32).unwrap()}
    }
}


impl Iterator for PrintableChars {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let code = self.current_char as u32;
        match code {
            32..=126 => {
                self.current_char = char::from_u32(code + 1).unwrap();
                Some(char::from_u32(code).unwrap())
            },
            _ => None,
        }
    }
}


pub fn load_default_fonts() -> Font {
    let src = SystemSource::new();
    let font = match src.select_by_postscript_name("DejaVuSans") {
        Ok(font) => font,
        Err(_) => src.select_best_match(&[FamilyName::SansSerif], &Properties::new()).unwrap()
    };
    font.load().unwrap()
}


fn estimate_atlas_size(font: &Font, size: f32) -> (i32, i32){
    let mut total_length = 0;
    let mut max_height = 0;
    for c in PrintableChars::new() {
        let glyph_id = font.glyph_for_char(c).expect(&format!("Could not lookup glyph for '{}' ({})", c, c as u32));
        let raster_rect = font.raster_bounds(
            glyph_id, size,
            Transform2F::default(),
            HintingOptions::Vertical(size),
            RasterizationOptions::SubpixelAa
        ).unwrap();
        total_length += raster_rect.width();
        max_height = max(max_height, raster_rect.height());
    }
    let total_area = total_length * max_height;
    // Add three extra rows: One for characters that dont fit at the end of the line,
    // and another for a possibly cut of row at the bottom, and one for margin :)
    let side_length_guess = (total_area as f32).sqrt();
    let side_length = (total_area as f32 + side_length_guess * (max_height as f32) * 3.0).sqrt() as i32;
    return (side_length, side_length)
}


pub fn generate_atlas(font: &Font, size: f32) -> Canvas {
    let (canvas_w, canvas_h) = estimate_atlas_size(&font, size);
    log::debug!("Creating font atlas with dimensions {}x{} for \"{}\", {:.1}px", canvas_w, canvas_h, font.full_name(), size);
    let mut canvas = Canvas::new(Vector2I::new(canvas_w, canvas_h), Format::Rgb24);
    let mut transform_vec = Vector2F::new(0.0, -size);

    for c in PrintableChars::new() {
        
        let glyph_id = font.glyph_for_char(c).unwrap();
        let raster_rect = font.raster_bounds(
            glyph_id, size,
            Transform2F::from_translation(transform_vec),
            HintingOptions::Vertical(size),
            RasterizationOptions::SubpixelAa
        ).unwrap();
        // Skip drawing this glyph if it is zero-width.
        if raster_rect.width() == 0 { continue; }
        
        let new_x = transform_vec.x() + raster_rect.width() as f32;
        if new_x > canvas_w as f32 {
            transform_vec.set_x(0.0);
            transform_vec.set_y(transform_vec.y() - size);
        }
        font.rasterize_glyph(
            &mut canvas,
            glyph_id,
            size,
            Transform2F::from_translation(transform_vec),
            HintingOptions::Vertical(size),
            RasterizationOptions::SubpixelAa,
        ).unwrap();

        transform_vec.set_x(new_x);
    }
    canvas
}
