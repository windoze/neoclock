use std::time::Duration;

use async_trait::async_trait;
use image::Rgba;
use log::info;
use rusttype::Font;
use serde::Deserialize;

use crate::{deserialize_pixel, Part, PartCache, RenderError, PartPixel, movers::Wigwagable};
use super::draw_text;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct WigwagWidget {
    pub width: u32,
    pub height: u32,
    // TODO:
    pub text: String,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub text_color: PartPixel,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub background_color: PartPixel,
    pub font_path: String,
    pub font_height: f32,
    pub font_scale_x: f32,
    pub font_scale_y: f32,
    pub speed: u32,
}

impl Default for WigwagWidget {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            text: Default::default(),
            text_color: Rgba::<u8>([255, 255, 0, 255]),
            background_color: Rgba::<u8>([0; 4]),
            font_path: Default::default(),
            font_height: 12.4,
            font_scale_x: 1.0,
            font_scale_y: 1.0,
            speed: 100,
        }
    }
}

#[async_trait]
impl Part for WigwagWidget {
    async fn start(&mut self, cache: PartCache, id: usize) -> Result<(), crate::RenderError> {
        let font_data = if self.font_path.is_empty() {
            info!("WigwagWidget({}) - Using default font.", id);
            Vec::from(super::font::DEF_FONT)
        } else {
            info!("WigwagWidget({}) - Using font at {}.", id, self.font_path);
            tokio::fs::read(&self.font_path).await?
        };
        let font =
            Font::try_from_vec(font_data).ok_or(RenderError::FontError(self.font_path.clone()))?;

        let text_img = draw_text(&self.text, self.text_color, &font, self.font_height, self.font_scale_x, self.font_scale_y);
        let mut f = text_img.wigwag(self.width, self.height);
        loop {
            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = f.next().unwrap();
            }
            tokio::time::sleep(Duration::from_millis((1000 / self.speed) as u64)).await;
        }
    }
}