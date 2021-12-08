mod movers;
mod widgets;

pub use movers::{ScrollIterator, Scrollable};
pub use widgets::Widget;
pub type PartPixel = image::Rgba<u8>;
pub type PartImage = ImageBuffer<PartPixel, Vec<u8>>;
pub type ScreenPixel = image::Rgb<u8>;
pub type ScreenImage = ImageBuffer<ScreenPixel, Vec<u8>>;
pub type PartSender = tokio::sync::mpsc::Sender<String>;
pub type PartChannel = tokio::sync::mpsc::Receiver<String>;

use async_trait::async_trait;
use image::{buffer::ConvertBuffer, ImageBuffer, Pixel};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::sync::{Arc, RwLock};
use thiserror::Error;
use tokio::task::JoinHandle;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Initialization Error {0}")]
    InitializationError(String),

    #[error("File at '{0}' is not a valid font.")]
    FontError(String),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    SendError(#[from] tokio::sync::mpsc::error::SendError<std::string::String>),

    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),
}

pub trait Drawable {
    fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8);
}

type PartCache = Arc<RwLock<Vec<PartImage>>>;

#[async_trait]
trait Part {
    async fn start(
        &mut self,
        cache: PartCache,
        id: usize,
        mut channel: PartChannel,
    ) -> Result<(), RenderError>;
}

#[derive(Clone, Debug, Deserialize)]
pub struct WidgetConf {
    x: u32,
    y: u32,
    #[serde(flatten)]
    widget: Widget,
}

pub struct Screen {
    width: u32,
    height: u32,
    positions: Vec<(u32, u32)>,
    part_contents: PartCache,
    children: Vec<JoinHandle<Result<(), RenderError>>>,
    senders: Vec<PartSender>,
}

impl Screen {
    pub fn new(width: u32, height: u32, parts: Vec<WidgetConf>) -> Self {
        let mut v = Vec::with_capacity(parts.len());
        v.resize(parts.len(), Default::default());
        let positions: Vec<(u32, u32)> = parts.iter().map(|p| (p.x, p.y)).collect();
        let part_contents: PartCache = Arc::new(RwLock::new(v));
        let mut children: Vec<JoinHandle<Result<(), RenderError>>> =
            Vec::with_capacity(parts.len());
        let mut senders: Vec<PartSender> = Vec::with_capacity(parts.len());
        for (idx, mut part) in parts.into_iter().enumerate() {
            let cache = part_contents.clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(100); // TODO:
            senders.push(sender);
            children.push(tokio::spawn(async move {
                part.widget.start(cache, idx, receiver).await
            }));
        }

        Self {
            width,
            height,
            positions,
            part_contents,
            children,
            senders,
        }
    }

    pub async fn stop(&mut self) {
        for c in self.children.iter_mut() {
            c.abort();
            c.await.unwrap().unwrap();
        }
    }

    fn render(&self) -> ScreenImage {
        let mut screen = PartImage::new(self.width, self.height);
        for px in screen.pixels_mut() {
            (*px) = image::Rgba::<u8>([0, 0, 0, 255]);
        }
        // Blend every part image into `screen`
        if let Ok(read_guard) = self.part_contents.read() {
            for (idx, img) in (*read_guard).iter().enumerate() {
                let (x, y) = self.positions[idx];
                // Blend `img` into `screen` at position `(x, y)`
                for px in 0..img.width() {
                    for py in 0..img.height() {
                        if (px + x) < self.width && (py + y) < self.height {
                            screen
                                .get_pixel_mut(px + x, py + y)
                                .blend(img.get_pixel(px, py))
                        }
                    }
                }
            }
        }
        screen.convert()
    }

    pub fn render_to<T>(&self, target: &mut T)
    where
        T: Drawable,
    {
        let image = self.render();
        for x in 0..image.width() {
            for y in 0..image.height() {
                let pixel = image.get_pixel(x, y);
                target.set_pixel(x, y, pixel.0[0], pixel.0[1], pixel.0[2]);
            }
        }
    }

    pub async fn send_str(&self, idx: usize, s: String) -> Result<(), RenderError> {
        self.senders[idx].send(s).await?;
        Ok(())
    }

    pub async fn send<T>(&self, idx: usize, t: &T) -> Result<(), RenderError>
    where
        T: Serialize,
    {
        self.send_str(idx, serde_json::to_string(t)?).await
    }
}

fn deserialize_pixel<'de, D>(deserializer: D) -> Result<PartPixel, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    let color = buf
        .parse::<css_color_parser::Color>()
        .map_err(|e| D::Error::custom(e.to_string()))?;
    Ok(image::Rgba::<u8>([
        color.r as u8,
        color.g as u8,
        color.b as u8,
        (color.a * 255f32) as u8,
    ]))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use image::{Rgb, Rgba};
    use serde::Deserialize;

    use super::*;

    #[test]
    fn test_de_color() {
        #[derive(Debug, Deserialize)]
        struct S {
            #[serde(deserialize_with = "deserialize_pixel")]
            p: PartPixel,
        }
        let j = r#"{"p": "rgb(10, 20, 30)"}"#;
        let s: S = serde_json::from_str(j).unwrap();
        assert_eq!(s.p.0[0], 10);
        assert_eq!(s.p.0[1], 20);
        assert_eq!(s.p.0[2], 30);
        assert_eq!(s.p.0[3], 255);

        let j = r#"{"p": "rgba(10, 20, 30, .5)"}"#;
        let s: S = serde_json::from_str(j).unwrap();
        assert_eq!(s.p.0[0], 10);
        assert_eq!(s.p.0[1], 20);
        assert_eq!(s.p.0[2], 30);
        assert_eq!(s.p.0[3], 127);

        let j = r#"{
            "type": "Clock",
            "x": 1,
            "y": 2,
            "width": 10,
            "height": 5,
            "text_color": "rgb(30, 40, 50)",
            "background_color": "rgb(60, 70, 80)"
        }"#;
        let w: WidgetConf = serde_json::from_str(j).unwrap();
        assert_eq!(w.x, 1);
        assert_eq!(w.y, 2);
        match w.widget {
            Widget::Clock(c) => {
                assert_eq!(c.width, 10);
                assert_eq!(c.height, 5);
                assert_eq!(c.text_color.0[0], 30);
                assert_eq!(c.text_color.0[1], 40);
                assert_eq!(c.text_color.0[2], 50);
                assert_eq!(c.text_color.0[3], 255);
                assert_eq!(c.background_color.0[0], 60);
                assert_eq!(c.background_color.0[1], 70);
                assert_eq!(c.background_color.0[2], 80);
                assert_eq!(c.background_color.0[3], 255);
            }
            _ => panic!(),
        }
    }

    #[tokio::test]
    async fn test_image() {
        let mut p1 = Rgba::<u8>([255, 0, 0, 255]);
        let p2 = Rgba::<u8>([0, 0, 255, 1]);
        p1.blend(&p2);
        println!("{:#?}", p1);
        let parts: Vec<WidgetConf> = serde_json::from_str(
            r#"[
            {
                "type": "Solid",
                "x": 0, 
                "y": 0,
                "width": 32,
                "height": 32,
                "color": "rgb(255,0,0)"
            },
            {
                "type": "Solid",
                "x": 32, 
                "y": 0,
                "width": 32,
                "height": 32,
                "color": "rgba(0,255,0,0.5)"
            },
            {
                "type": "Solid",
                "x": 0, 
                "y": 32,
                "width": 64,
                "height": 32,
                "color": "rgba(0,0,255, 0.5)"
            },
            {
                "type": "Clock",
                "x": 10, 
                "y": 0,
                "width": 64,
                "height": 32,
                "text_color": "rgba(0,0,255, 0.5)",
                "background_color": "rgba(0,0,0,0)"
            }
        ]"#,
        )
        .unwrap();
        let s = Screen::new(64, 64, parts);
        for n in 0..5 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let img = s.render();
            img.save_with_format(format!("/tmp/b-{}.png", n), image::ImageFormat::Png)
                .unwrap();
        }
        let img = s.render();
        assert_eq!(img.get_pixel(0, 0), &Rgb::<u8>([255, 0, 0]));
        assert_eq!(img.get_pixel(31, 31), &Rgb::<u8>([255, 0, 0]));
        assert_eq!(img.get_pixel(32, 31), &Rgb::<u8>([0, 127, 0]));
        assert_eq!(img.get_pixel(63, 0), &Rgb::<u8>([0, 127, 0]));
        assert_eq!(img.get_pixel(0, 32), &Rgb::<u8>([0, 0, 127]));
        assert_eq!(img.get_pixel(63, 63), &Rgb::<u8>([0, 0, 127]));
    }
}
