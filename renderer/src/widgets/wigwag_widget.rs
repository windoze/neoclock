use std::time::Duration;

use async_trait::async_trait;
use image::Rgba;
use log::debug;
use serde::Deserialize;

use super::FontConfig;
use crate::{
    deserialize_pixel, movers::Wigwagable, Part, PartCache, PartChannel, PartPixel, RenderError,
};

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
    #[serde(flatten)]
    pub font_config: FontConfig,
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
            font_config: Default::default(),
            speed: 100,
        }
    }
}

#[async_trait]
impl Part for WigwagWidget {
    async fn start(
        &mut self,
        cache: PartCache,
        id: usize,
        mut channel: PartChannel,
    ) -> Result<(), RenderError> {
        let font = self.font_config.load()?;

        let text_img = font.draw_text(&self.text, self.text_color);
        let mut f = text_img.wigwag(self.width, self.height);
        loop {
            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = Some(f.next().unwrap());
            }
            let d = Duration::from_millis((1000 / self.speed) as u64);
            if let Some(s) = match tokio::time::timeout(d, channel.recv()).await {
                Ok(s) => s,
                Err(_) => None,
            } {
                // TODO: Received a message
                debug!("Got message '{}'", s);
            }
        }
    }
}
