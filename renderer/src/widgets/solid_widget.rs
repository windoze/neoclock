use crate::{deserialize_pixel, Part, PartCache, PartImage, PartPixel};
use async_trait::async_trait;
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
    async fn start(&mut self, cache: PartCache, id: usize) -> Result<(), crate::RenderError> {
        let mut img = PartImage::new(self.width, self.height);

        for p in img.pixels_mut() {
            *p = self.color;
        }
        if let Ok(mut write_guard) = cache.write() {
            (*write_guard)[id] = img;
        }
        Ok(())
    }
}
