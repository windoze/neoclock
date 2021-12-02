use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{deserialize_pixel, Part, PartCache, PartImage, PartPixel};
use async_trait::async_trait;
use image::Rgba;
use rand::{Rng, SeedableRng};
use serde::Deserialize;

const COLOR_STEP: u32 = 20;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct MatrixRainWidget {
    pub width: u32,
    pub height: u32,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub color: PartPixel,
    pub speed: u32,
}

impl Default for MatrixRainWidget {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            color: Rgba::<u8>([0, 255, 0, 255]),
            speed: 100,
        }
    }
}

#[async_trait]
impl Part for MatrixRainWidget {
    async fn start(&mut self, cache: PartCache, id: usize) {
        let mut lines: Vec<(u32, u32)> = Vec::new();
        let mut last_in: u32 = 0;
        let mut rng = rand::rngs::StdRng::seed_from_u64(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        );
        loop {
            let mut img = PartImage::new(self.width, self.height);

            // Transparent background
            for p in img.pixels_mut() {
                (*p) = Rgba::<u8>([0, 0, 0, 0]);
            }

            if last_in == 0 {
                last_in = rng.gen_range(2..8);
                let x = rng.gen_range(0..64);
                lines.push((x, 0));
            }

            for n in (0..lines.len()).rev() {
                let l = lines.get_mut(n).unwrap();
                for i in 0..15 {
                    let y = if l.1 < i { 0 } else { l.1 - i };
                    let color =
                        Rgba::<u8>([0, 255, 0, (255 * (COLOR_STEP - i) / COLOR_STEP) as u8]);
                    if y < img.height() {
                        let px = img.get_pixel_mut(l.0, y);
                        (*px) = color;
                    }
                }
                l.1 = l.1 + 1;
                if l.1 > COLOR_STEP + self.height {
                    lines.remove(n);
                }
            }

            last_in = last_in - 1;
            tokio::time::sleep(Duration::from_millis((1000 / self.speed) as u64)).await;
            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = img;
            }
        }
    }
}
