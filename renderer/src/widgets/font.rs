use rusttype::{Font, Scale};

use crate::{PartPixel, PartImage};

pub const DEF_FONT: &[u8] = include_bytes!("../../fonts/DejaVuSansMono.ttf");

pub fn draw_text(text: &str, color: PartPixel, font: &Font, height: f32, scale_x: f32, scale_y: f32) -> PartImage {
    let pixel_height = height.ceil() as usize;
    let scale = Scale {
        x: height * scale_x,
        y: height * scale_y,
    };
    let v_metrics = font.v_metrics(scale);
    let offset = rusttype::point(0.0, v_metrics.ascent);

    let glyphs: Vec<_> = font.layout(text, scale, offset).collect();
    let width = glyphs
        .iter()
        .rev()
        .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
        .next()
        .unwrap_or(0.0)
        .ceil() as usize;

    let mut img = PartImage::new(width as u32, pixel_height as u32);

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;
                // There's still a possibility that the glyph clips the boundaries of the bitmap
                if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                    let x = x as u32;
                    let y = y as u32;
                    let px = img.get_pixel_mut(x, y);
                    let mut color = color;
                    color.0[3] = (v * 255f32) as u8;
                    (*px) = color;
                }
            })
        }
    }

    img
}