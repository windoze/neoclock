use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use image::Rgba;
use serde::Deserialize;

use crate::{deserialize_pixel, PartCache, PartPixel, RenderError};

use super::FontConfig;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct ClockWidget {
    pub width: u32,
    pub height: u32,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub text_color: PartPixel,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub background_color: PartPixel,
    #[serde(flatten)]
    pub font_config: FontConfig,
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
            font_config: Default::default(),
        }
    }
}

#[async_trait]
impl crate::Part for ClockWidget {
    async fn start(&mut self, cache: PartCache, id: usize) -> Result<(), RenderError> {
        let font = self.font_config.load()?;

        loop {
            let now = chrono::Local::now();
            let time_str = if now.timestamp() % 2 == 0 {
                now.format("%H %M")
            } else {
                now.format("%H:%M")
            }
            .to_string();

            let img = font.draw_text(&time_str, self.text_color);
            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = img;
            }
            self.sleep().await;
        }
    }
}
