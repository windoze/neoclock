use async_trait::async_trait;
use chrono::{Datelike, Duration, NaiveDate, TimeZone, Timelike, Utc};
use image::Rgba;
use log::{debug, info};
use serde::Deserialize;
use tokio::time::timeout;

use crate::{deserialize_pixel, Part, PartCache, PartChannel, PartPixel, RenderError};

use super::font::FontConfig;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct CalendarWidget {
    pub width: u32,
    pub height: u32,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub text_color: PartPixel,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub background_color: PartPixel,
    #[serde(flatten)]
    pub font_config: FontConfig,
}

impl CalendarWidget {
    async fn sleep(&self, channel: &mut PartChannel) -> Option<String> {
        // Sleep until the beginning of the next minute
        let now = Utc::now();
        let n = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
            .and_then(|nd| nd.and_hms_opt(now.hour(), now.minute(), 0))
            .unwrap();
        let next_min = Utc.from_utc_datetime(&(n + Duration::minutes(1)));
        let d = next_min - now;
        match timeout(d.to_std().unwrap_or_default(), channel.recv()).await {
            Ok(v) => v,
            Err(_) => None,
        }
    }
}

impl Default for CalendarWidget {
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
impl Part for CalendarWidget {
    async fn start(
        &mut self,
        cache: PartCache,
        id: usize,
        mut channel: PartChannel,
    ) -> Result<(), RenderError> {
        info!("CalendarWidget({}) started.", id);
        let font = self.font_config.load()?;
        loop {
            let now = chrono::Local::now();
            let date_str = now.format("%b %d %a").to_string();

            let img = font.draw_text(&date_str, self.text_color, self.background_color);

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
