use std::time::SystemTime;

use crate::{deserialize_pixel, PartImage, PartPixel, PeriodicallyRefreshedPart, PartCache};
use async_trait::async_trait;
use chrono::{Utc, Timelike, NaiveDateTime, DateTime};
use image::Rgba;
use rusttype::{Font, Scale};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct ClockWidget {
    pub width: u32,
    pub height: u32,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub text_color: PartPixel,
    #[serde(deserialize_with = "deserialize_pixel")]
    pub background_color: PartPixel,
    pub font_path: String,

    #[serde(skip)]
    font_data: Vec<u8>,
}

impl ClockWidget {
    async fn sleep(&self) {
        // Sleep until the beginning of the next second
        // TODO: Is there any better way to do that?
        let now = Utc::now();
        let ts = now.timestamp() + 1 /*sec*/;
        let ns = NaiveDateTime::from_timestamp(ts, 0);
        let nt: DateTime<Utc> = DateTime::from_utc(ns, Utc);
        let d = nt - now;
        tokio::time::sleep(d.to_std().unwrap()).await;
    }
}

impl Default for ClockWidget {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            text_color: Rgba::<u8>([255, 255, 0, 255]),
            background_color: Rgba::<u8>([0; 4]),
            font_path: Default::default(),
            font_data: Vec::from(super::font::DEF_FONT),
        }
    }
}

#[async_trait]
impl crate::Part for ClockWidget {
    async fn start(&mut self, cache: PartCache, id: usize) {
        let font = Font::try_from_vec(self.font_data.clone()).unwrap();
        let height: f32 = 12.4; // to get 80 chars across (fits most terminals); adjust as desired
        let pixel_height = height.ceil() as usize;
        let scale = Scale {
            x: height * 1.5,
            y: height,
        };
        let v_metrics = font.v_metrics(scale);
        let offset = rusttype::point(0.0, v_metrics.ascent);

        loop {
            let now = chrono::Local::now();
            let time_str = if now.timestamp() % 2 == 0 {
                now.format("%H %M")
            } else {
                now.format("%H:%M")
            }
            .to_string();
    
            let glyphs: Vec<_> = font.layout(&time_str, scale, offset).collect();
            let width = glyphs
                .iter()
                .rev()
                .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
                .next()
                .unwrap_or(0.0)
                .ceil() as usize;
    
            // println!("str: {}, width: {}, height: {}", time_str, width, pixel_height);
    
            let mut img = PartImage::new(self.width, self.height);
            for g in glyphs {
                if let Some(bb) = g.pixel_bounding_box() {
                    g.draw(|x, y, v| {
                        // let i = (v * mapping_scale + 0.5) as usize;
                        // so something's wrong if you get $ in the output.
                        let x = x as i32 + bb.min.x;
                        let y = y as i32 + bb.min.y;
                        // There's still a possibility that the glyph clips the boundaries of the bitmap
                        if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                            let x = x as u32;
                            let y = y as u32;
                            // pixel_data[(x + y * width)] = c;
                            let px = img.get_pixel_mut(x, y);
                            let mut color = self.text_color;
                            color.0[3] = (v * 255f32) as u8;
                            (*px) = color;
                            }
        
                    })
                }
            }
            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = img;
            }
            self.sleep().await;
        }
    }
}

// #[async_trait]
// impl PeriodicallyRefreshedPart for ClockWidget {
//     async fn init(&mut self, id: usize) {
//         if !self.font_path.is_empty() {
//             self.font_data = tokio::fs::read(&self.font_path).await.unwrap();
//         }
//     }

//     fn draw(&self) -> PartImage {
//         let font = Font::try_from_vec(self.font_data.clone()).unwrap();
//         let height: f32 = 12.4; // to get 80 chars across (fits most terminals); adjust as desired
//         let pixel_height = height.ceil() as usize;
//         let scale = Scale {
//             x: height * 1.5,
//             y: height,
//         };
//         let v_metrics = font.v_metrics(scale);
//         let offset = rusttype::point(0.0, v_metrics.ascent);

//         let now = chrono::Local::now();
//         let time_str = if now.timestamp() % 2 == 0 {
//             now.format("%H %M")
//         } else {
//             now.format("%H:%M")
//         }
//         .to_string();

//         let glyphs: Vec<_> = font.layout(&time_str, scale, offset).collect();
//         let width = glyphs
//             .iter()
//             .rev()
//             .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
//             .next()
//             .unwrap_or(0.0)
//             .ceil() as usize;

//         println!("str: {}, width: {}, height: {}", time_str, width, pixel_height);

//         let mut img = PartImage::new(self.width, self.height);
//         for g in glyphs {
//             if let Some(bb) = g.pixel_bounding_box() {
//                 g.draw(|x, y, v| {
//                     // let i = (v * mapping_scale + 0.5) as usize;
//                     // so something's wrong if you get $ in the output.
//                     let x = x as i32 + bb.min.x;
//                     let y = y as i32 + bb.min.y;
//                     // There's still a possibility that the glyph clips the boundaries of the bitmap
//                     if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
//                         let x = x as u32;
//                         let y = y as u32;
//                         // pixel_data[(x + y * width)] = c;
//                         let px = img.get_pixel_mut(x, y);
//                         let mut color = self.text_color;
//                         color.0[3] = (v * 255f32) as u8;
//                         (*px) = color;
//                         }
    
//                 })
//             }
//         }

//         img
//     }
// }
