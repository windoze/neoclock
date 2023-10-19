use std::time::Duration;

use crate::{
    deserialize_pixel, fill, message::SolidMessage, Part, PartCache, PartChannel, PartImage,
    PartPixel,
};
use async_trait::async_trait;
use log::{debug, info};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct SolidWidget {
    pub width: u32,
    pub height: u32,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub color: PartPixel,
}

#[async_trait]
impl Part for SolidWidget {
    async fn start(
        &mut self,
        cache: PartCache,
        id: usize,
        mut channel: PartChannel,
    ) -> Result<(), crate::RenderError> {
        info!("SolidWidget({}) started.", id);
        loop {
            let mut img = PartImage::new(self.width, self.height);
            fill(&mut img, self.color);
            if let Ok(mut write_guard) = cache.write() {
                write_guard.image = Some(img);
            }

            if let Some(msg) = self
                .try_read::<SolidMessage>(&mut channel, Duration::from_secs(86400))
                .await
            {
                debug!("Solid widget {} got message '{:#?}'", id, msg);
            }
        }
    }
}
