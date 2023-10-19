use std::{time::Duration, path::PathBuf};

use async_trait::async_trait;
use image::{AnimationDecoder, Frame};
use log::{debug, info};
use serde::Deserialize;

use crate::{Part, PartCache, PartChannel, RenderError, widgets::message::GifMessage};

#[derive(Clone, Debug, Deserialize)]
pub struct GifWidget {
    // TODO:
    pub location: String,
}

impl GifWidget {
    async fn load_gif(&self, url: &str) -> Result<Vec<Frame>, RenderError> {
        info!("GifWidget - Loading GIF from '{}'...", self.location);
        let bytes = if self.location.starts_with("http://") || self.location.starts_with("https://") {
            // Read from URL
            reqwest::Client::default()
            .get(url)
            .send()
            .await?
            .bytes()
            .await?
        } else {
            // Read from file
            tokio::fs::read(PathBuf::from(url)).await?.into()
        };

        let frames = image::codecs::gif::GifDecoder::new(bytes.as_ref())?
            .into_frames()
            .collect_frames()?;
        debug!("GifWidget - Frame count: {}", frames.len());

        Ok(frames)
    }
}

#[async_trait]
impl Part for GifWidget {
    async fn start(
        &mut self,
        cache: PartCache,
        id: usize,
        mut channel: PartChannel,
    ) -> Result<(), crate::RenderError> {
        info!("GifWidget({}) started.", id);
        let mut frames = self.load_gif(&self.location).await.unwrap_or_default();
        let mut i = 0;
        loop {
            if !frames.is_empty() {
                let img = frames[i].buffer().clone();

                if let Ok(mut write_guard) = cache.write() {
                    write_guard.image = Some(img);
                }
            }

            let d = if frames.is_empty() {
                Duration::from_secs(86400)
            } else {
                let (numer, denom) = frames[i].delay().numer_denom_ms();
                let delay = ((numer as f64 * 1000f64) / (denom as f64)) as u64;
                Duration::from_micros(delay)
            };
            if let Some(msg) = self.try_read::<GifMessage>(&mut channel, d).await {
                debug!("Gif widget {} got message '{:#?}'", id, msg);
                if let Ok(f) = self.load_gif(&msg.url).await {
                    frames = f;
                    i = 0;
                }
            }

            i = if frames.is_empty() {
                0
            } else {
                (i + 1) % frames.len()
            }
        }
    }
}
