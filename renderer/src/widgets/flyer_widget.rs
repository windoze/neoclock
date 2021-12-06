use std::time::Duration;

use async_trait::async_trait;
use image::Rgba;
use serde::Deserialize;

use crate::{deserialize_pixel, Part, PartCache, RenderError, PartPixel, Scrollable};
use super::FontConfig;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct FlyerWidget {
    pub width: u32,
    pub height: u32,
    // TODO:
    pub text: String,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub text_color: PartPixel,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub background_color: PartPixel,
    #[serde(flatten)]
    pub font_config: FontConfig,
    pub speed: u32,
}

impl Default for FlyerWidget {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            text: Default::default(),
            text_color: Rgba::<u8>([255, 255, 0, 255]),
            background_color: Rgba::<u8>([0; 4]),
            font_config: Default::default(),
            speed: 100,
        }
    }
}

#[async_trait]
impl Part for FlyerWidget {
    async fn start(&mut self, cache: PartCache, id: usize) -> Result<(), RenderError> {
        let font = self.font_config.load()?;

        let text_img = font.draw_text(&self.text, self.text_color);
        let mut f = text_img.scroll(self.width, self.height, -1, 0);
        loop {
            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = f.next().unwrap();
            }
            tokio::time::sleep(Duration::from_millis((1000 / self.speed) as u64)).await;
        }
    }
}