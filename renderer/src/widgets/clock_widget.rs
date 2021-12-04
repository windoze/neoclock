use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use image::Rgba;
use log::info;
use rusttype::Font;
use serde::Deserialize;

use crate::{deserialize_pixel, PartCache, PartPixel, RenderError};

use super::draw_text;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct ClockWidget {
    pub width: u32,
    pub height: u32,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub text_color: PartPixel,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub background_color: PartPixel,
    pub font_path: String,
    pub font_height: f32,
    pub font_scale_x: f32,
    pub font_scale_y: f32,
}

impl ClockWidget {
    async fn sleep(&self) {
        // Sleep until the beginning of the next second
        // TODO: Is there any better way to do that?
        let now = Utc::now();
        let ts = now.timestamp() + 1 /*sec*/;
        let ns = NaiveDateTime::from_timestamp(ts, 0);
        let nt: DateTime<Utc> = DateTime::from_utc(ns, Utc);
        let d = nt - now;
        tokio::time::sleep(d.to_std().unwrap()).await;
    }
}

impl Default for ClockWidget {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            text_color: Rgba::<u8>([255, 255, 0, 255]),
            background_color: Rgba::<u8>([0; 4]),
            font_path: Default::default(),
            font_height: 20.5,
            font_scale_x: 1.2,
            font_scale_y: 1.0,
        }
    }
}

#[async_trait]
impl crate::Part for ClockWidget {
    async fn start(&mut self, cache: PartCache, id: usize) -> Result<(), crate::RenderError> {
        let font_data = if self.font_path.is_empty() {
            info!("ClockWidget({}) - Using default font.", id);
            Vec::from(super::font::DEF_FONT)
        } else {
            info!("ClockWidget({}) - Using font at {}.", id, self.font_path);
            tokio::fs::read(&self.font_path).await?
        };
        let font =
            Font::try_from_vec(font_data).ok_or(RenderError::FontError(self.font_path.clone()))?;

        loop {
            let now = chrono::Local::now();
            let time_str = if now.timestamp() % 2 == 0 {
                now.format("%H %M")
            } else {
                now.format("%H:%M")
            }
            .to_string();

            let img = draw_text(&time_str, self.text_color, &font, self.font_height, self.font_scale_x, self.font_scale_y);
            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = img;
            }
            self.sleep().await;
        }
    }
}
