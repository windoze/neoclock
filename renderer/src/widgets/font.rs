use std::cmp::max;

use bdf::Glyph;
use image::ImageBuffer;
use rusttype::Scale;
use serde::Deserialize;

use crate::{PartImage, PartPixel, RenderError};

pub const DEF_FONT: &[u8] = include_bytes!("../../fonts/DejaVuSansMono.ttf");

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct FontConfig {
    pub font_path: String,
    pub font_height: f32,
    pub font_scale_x: f32,
    pub font_scale_y: f32,
}

impl FontConfig {
    pub fn load(&self) -> Result<Font, RenderError> {
        Font::load(
            &self.font_path,
            self.font_height,
            self.font_scale_x,
            self.font_scale_y,
        )
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            font_path: Default::default(),
            font_height: 12.4,
            font_scale_x: 1.0,
            font_scale_y: 1.0,
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum Font {
    Ttf {
        font: rusttype::Font<'static>,
        height: f32,
        scale_x: f32,
        scale_y: f32,
    },
    Bdf {
        font: bdf::Font,
        scale_x: u32,
        scale_y: u32,
    },
}

impl Font {
    fn load(path: &str, height: f32, scale_x: f32, scale_y: f32) -> Result<Self, RenderError> {
        Ok(if path.to_lowercase().ends_with(".bdf") {
            Self::Bdf {
                font: bdf::open(path).map_err(|_| RenderError::FontError(path.to_owned()))?,
                scale_x: scale_x as u32,
                scale_y: scale_y as u32,
            }
        } else {
            Self::Ttf {
                font: {
                    let font_data = if path.is_empty() {
                        Vec::from(super::font::DEF_FONT)
                    } else {
                        std::fs::read(path)?
                    };
                    rusttype::Font::try_from_vec(font_data)
                        .ok_or_else(|| RenderError::FontError(path.to_owned()))?
                },
                height,
                scale_x,
                scale_y,
            }
        })
    }

    pub fn draw_text(&self, text: &str, color: PartPixel) -> PartImage {
        match self {
            Font::Ttf {
                font,
                height,
                scale_x,
                scale_y,
            } => draw_ttf_text(
                text,
                color,
                font,
                height.to_owned(),
                scale_x.to_owned(),
                scale_y.to_owned(),
            ),
            Font::Bdf {
                font,
                scale_x,
                scale_y,
            } => draw_bdf_text(text, color, font, scale_x.to_owned(), scale_y.to_owned()),
        }
    }
}

fn draw_ttf_text(
    text: &str,
    color: PartPixel,
    font: &rusttype::Font,
    height: f32,
    scale_x: f32,
    scale_y: f32,
) -> PartImage {
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

fn draw_bdf_text(
    text: &str,
    color: PartPixel,
    font: &bdf::Font,
    scale_x: u32,
    scale_y: u32,
) -> PartImage {
    let mut width = 0u32;
    let mut height = 0u32;

    for c in text.chars() {
        let bounding_box = if font.glyphs().contains_key(&c) {
            font.glyphs()[&c].bounds()
        } else {
            font.glyphs()[&' '].bounds()
        };
        width += bounding_box.width;
        height = max(height, bounding_box.height);
    }
    let mut offset_x = 0u32;
    let mut image = ImageBuffer::<PartPixel, Vec<u8>>::new(width * scale_x, height * scale_y);
    for c in text.chars() {
        let glyph: &Glyph = if font.glyphs().contains_key(&c) {
            &font.glyphs()[&c]
        } else {
            // Use ' ' if the char is not included in the font
            &font.glyphs()[&' ']
        };
        for ((x, y), p) in glyph.pixels() {
            if p {
                // Scale pixel by scale_x and scale_y
                for s_x in 0..scale_x {
                    for s_y in 0..scale_y {
                        image.put_pixel((x + offset_x) * scale_x + s_x, y * scale_y + s_y, color);
                    }
                }
            }
        }
        offset_x += glyph.bounds().width;
    }

    image
}
