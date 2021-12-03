mod widgets;

pub use widgets::Widget;
pub type PartPixel = image::Rgba<u8>;
pub type PartImage = ImageBuffer<PartPixel, Vec<u8>>;
pub type ScreenPixel = image::Rgb<u8>;
pub type ScreenImage = ImageBuffer<ScreenPixel, Vec<u8>>;

use std::{sync::{RwLock, Arc}, time::Duration};
use async_trait::async_trait;
use image::{ImageBuffer, buffer::ConvertBuffer, Pixel};
use serde::{Deserializer, Deserialize, de::Error};

pub trait Drawable {
    fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8);
}

impl Drawable for ScreenImage {
    fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        self.put_pixel(x, y, image::Rgb::<u8>([r, g, b]));
    }
}

type PartCache = Arc<RwLock<Vec<PartImage>>>;

#[async_trait]
trait Part {
    async fn start(&mut self, cache: PartCache, id: usize);
}

#[async_trait]
trait PeriodicallyRefreshedPart {
    fn get_interval(&self) -> Duration {
        Duration::from_millis(100) 
    }

    async fn init(&mut self, _id: usize) {}
    
    fn draw(&self) -> PartImage;
}

#[async_trait]
impl<T> Part for T
    where T: PeriodicallyRefreshedPart + Send + Sync
{
    async fn start(&mut self, cache: PartCache, id: usize) {
        self.init(id).await;
        loop {
            let image = self.draw();
            if let Ok(mut write_guard) = cache.write() {
                (*write_guard)[id] = image;
            }
            tokio::time::sleep(self.get_interval()).await;
        }
    }
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
}

impl Screen {
    pub fn new(width: u32, height: u32, parts: Vec<WidgetConf>) -> Self {
        let mut v = Vec::with_capacity(parts.len());
        v.resize(parts.len(), Default::default());
        let r = Self {
            width,
            height,
            positions: parts.iter().map(|p| { (p.x, p.y) }).collect(),
            part_contents: Arc::new(RwLock::new(v)),
        };

        for (idx, mut part) in parts.into_iter().enumerate() {
            let cache = r.part_contents.clone();
            tokio::spawn(async move {
                part.widget.start(cache, idx).await;
            });
        }

        r
    }

    pub fn render(&self) -> ScreenImage {
        let mut screen = PartImage::new(self.width, self.height);
        for px in screen.pixels_mut() {
            (*px) = image::Rgba::<u8>([0,0,0,255]);
        }
        // Blend every part image into `screen`
        if let Ok(read_guard) = self.part_contents.read() {
            for (idx, img) in (*read_guard).iter().enumerate() {
                let (x, y) = self.positions[idx];
                // Blend `img` into `screen` at position `(x, y)`
                for px in 0..img.width() {
                    for py in 0..img.height() {
                        if (px + x) < self.width && (py + y) < self.height {
                            screen.get_pixel_mut(px + x, py + y).blend(img.get_pixel(px, py))
                        }
                    }
                }
            }
        }
        screen.convert()
    }

    pub fn render_to<T>(&self, target: &mut T)
        where T: Drawable
    {
        let image = self.render();
        for x in 0..image.width() {
            for y in 0..image.height() {
                let pixel = image.get_pixel(x, y);
                target.set_pixel(x, y, pixel.0[0], pixel.0[1], pixel.0[2]);
            }
        }
    }
}

fn deserialize_pixel<'de, D>(deserializer: D) -> Result<PartPixel, D::Error>
    where D: Deserializer<'de>
{
    let buf = String::deserialize(deserializer)?;
    let color = buf.parse::<css_color_parser::Color>().map_err(|e| {
        D::Error::custom(e.to_string())
    })?;
    Ok(image::Rgba::<u8>([color.r as u8, color.g as u8, color.b as u8, (color.a * 255f32) as u8]))
}

#[cfg(test)]
mod tests {
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
            },
            _ => assert!(false),
        }
    }

    #[tokio::test]
    async fn test_image() {
        let mut p1 = Rgba::<u8>([255, 0,0,255]);
        let p2 = Rgba::<u8>([0,0,255, 1]);
        p1.blend(&p2);
        println!("{:#?}", p1);
        let parts: Vec<WidgetConf> = serde_json::from_str(r#"[
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
            },
            {
                "type": "MatrixRain",
                "x": 0, 
                "y": 0,
                "width": 64,
                "height": 64,
                "speed": 100,
                "color": "rgba(0,255,0, 0.5)"
            }
        ]"#).unwrap();
        let s = Screen::new(64, 64, parts);
        for n in 0..5 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let img = s.render();
            img.save_with_format(format!("/tmp/b-{}.png", n), image::ImageFormat::Png).unwrap();
        }
        let img = s.render();
        assert_eq!(img.get_pixel(0, 0), &Rgb::<u8>([255,0,0]));
        assert_eq!(img.get_pixel(31, 31), &Rgb::<u8>([255,0,0]));
        assert_eq!(img.get_pixel(32, 31), &Rgb::<u8>([0,127,0]));
        assert_eq!(img.get_pixel(63, 0), &Rgb::<u8>([0,127,0]));
        assert_eq!(img.get_pixel(0, 32), &Rgb::<u8>([0,0,127]));
        assert_eq!(img.get_pixel(63, 63), &Rgb::<u8>([0,0,127]));
    }
}