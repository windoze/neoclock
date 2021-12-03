use std::time::Duration;

use async_trait::async_trait;
use image::AnimationDecoder;
use serde::Deserialize;

use crate::{Part, PartCache};

#[derive(Clone, Debug, Deserialize)]
pub struct GifWidget {
    // TODO:
    url: String,
}

#[async_trait]
impl Part for GifWidget {
    async fn start(&mut self, cache: PartCache, id: usize) {
        let bytes = reqwest::Client::default().get(&self.url)
            .send().await.unwrap().bytes().await.unwrap();

        let frames = image::codecs::gif::GifDecoder::new(bytes.as_ref()).unwrap().into_frames().collect_frames().unwrap();
        println!("Frame count: {}", frames.len());

        let mut i = 0;
        loop {
            let img = frames[i].buffer().clone();

            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = img;
            }

            let (numer, denom) = frames[i].delay().numer_denom_ms();
            let delay = ((numer as f64 * 1000f64) / (denom as f64)) as u64;
            let d = Duration::from_micros(delay);
            tokio::time::sleep(d).await;

            i = (i + 1) % frames.len();
        }
    }
}