use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use image::Rgba;
use log::{debug, info};
use serde::Deserialize;
use tokio::time::timeout;

use crate::{deserialize_pixel, PartCache, PartChannel, PartPixel, RenderError};

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
    async fn sleep(&self, channel: &mut PartChannel) -> Option<String> {
        // Sleep until the beginning of the next second
        // TODO: Is there any better way to do that?
        let now = Utc::now();
        let ts = now.timestamp() + 1 /*sec*/;
        let ns = NaiveDateTime::from_timestamp(ts, 0);
        let nt: DateTime<Utc> = DateTime::from_utc(ns, Utc);
        let d = nt - now;
        match timeout(d.to_std().unwrap(), channel.recv()).await {
            Ok(v) => v,
            Err(_) => None,
        }
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
    async fn start(
        &mut self,
        cache: PartCache,
        id: usize,
        mut channel: PartChannel,
    ) -> Result<(), RenderError> {
        info!("ClockWidget({}) started.", id);
        let font = self.font_config.load()?;

        loop {
            let now = chrono::Local::now();
            let time_str = if now.timestamp() % 2 == 0 {
                now.format("%H %M")
            } else {
                now.format("%H:%M")
            }
            .to_string();

            let img = font.draw_text(&time_str, self.text_color, self.background_color);
            if let Ok(mut write_guard) = cache.write() {
                write_guard.image = Some(img);
            }
            if let Some(s) = self.sleep(&mut channel).await {
                // TODO: Received a message
                debug!("Got message '{}'", s);
            }
        }
    }
}
