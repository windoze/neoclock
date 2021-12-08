use std::time::{Duration, Instant};

use async_trait::async_trait;
use image::{GenericImage, Rgba};
use log::debug;
use serde::Deserialize;

use super::FontConfig;
use crate::{
    deserialize_pixel, Part, PartCache, PartChannel, PartImage, PartPixel, RenderError,
    ScrollIterator, Scrollable,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct FlyerWidget {
    pub width: u32,
    pub height: u32,
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
            text_color: Rgba::<u8>([255, 255, 0, 255]),
            background_color: Rgba::<u8>([0; 4]),
            font_config: Default::default(),
            speed: 100,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FlyerMessage {
    text: String,
    ttl: u32,

    #[serde(skip)]
    expiration: Option<Instant>,
}

#[async_trait]
impl Part for FlyerWidget {
    async fn start(
        &mut self,
        cache: PartCache,
        id: usize,
        mut channel: PartChannel,
    ) -> Result<(), RenderError> {
        let font = self.font_config.load()?;

        let mut messages: Vec<(FlyerMessage, u32, ScrollIterator<PartPixel>)> = Default::default();
        loop {
            let n = Instant::now();
            let mut height: u32 = 0;
            messages = messages
                .into_iter()
                .filter(|(m, h, _)| {
                    if m.expiration.is_some() && m.expiration.unwrap() > n {
                        height += h;
                        true
                    } else {
                        debug!("Remove message '{}'.", m.text);
                        false
                    }
                })
                .collect();

            let img = if messages.is_empty() {
                None
            } else {
                let mut image = PartImage::new(self.width, height);
                let mut y = 0u32;
                for (_, h, i) in messages.iter_mut() {
                    image.copy_from(&(i.next().unwrap()), 0, y).unwrap();
                    y += *h;
                }
                Some(image)
            };
            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = img;
            }

            let d = Duration::from_millis((1000 / self.speed) as u64);
            if let Some(s) = match tokio::time::timeout(d, channel.recv()).await {
                Ok(s) => s,
                Err(_) => None,
            } {
                let mut msg: FlyerMessage = serde_json::from_str(&s).unwrap_or_default();
                if msg.ttl > 0 {
                    debug!("Got message '{:#?}'", msg.text);
                    msg.expiration = Some(Instant::now() + Duration::from_secs(msg.ttl as u64));
                    let img = font.draw_text(&msg.text, self.text_color);
                    messages.push((
                        msg,
                        img.height(),
                        img.scroll(self.width, img.height(), -1, 0),
                    ));
                }
            }
        }
    }
}
