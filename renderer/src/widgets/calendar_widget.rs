use async_trait::async_trait;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use image::Rgba;
use serde::Deserialize;

use crate::{deserialize_pixel, Part, PartCache, PartPixel, RenderError};

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
    async fn sleep(&self) {
        // Sleep until the beginning of the next day
        // TODO: Is there any better way to do that?
        let now = Utc::now();
        let n = NaiveDate::from_ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 1);
        let nt: DateTime<Utc> = DateTime::from_utc(n, Utc)
            .checked_add_signed(Duration::from_std(std::time::Duration::from_secs(86400)).unwrap())
            .unwrap();
        let d = nt - now;
        tokio::time::sleep(d.to_std().unwrap()).await;
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
    async fn start(&mut self, cache: PartCache, id: usize) -> Result<(), RenderError> {
        let font = self.font_config.load()?;
        loop {
            let now = chrono::Local::now();
            let date_str = now.format("%b %d %a").to_string();

            let img = font.draw_text(&date_str, self.text_color);

            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = img;
            }
            self.sleep().await;
        }
    }
}
