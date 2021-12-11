mod movers;
mod widgets;
mod screen;

pub use screen::Screen;
pub use widgets::message;
pub use widgets::Widget;
pub(crate) type PartPixel = image::Rgba<u8>;
pub(crate) type PartImage = ImageBuffer<PartPixel, Vec<u8>>;
pub(crate) use screen::{Part, PartCache, PartChannel, PartSender};

use image::{ImageBuffer, Pixel};
use serde::{    de::Error,Deserialize, Deserializer };
use thiserror::Error;

pub const TRANSPARENT: PartPixel = image::Rgba::<u8>([0, 0, 0, 0]);
pub const BLACK: PartPixel = image::Rgba::<u8>([0, 0, 0, 255]);
pub const WHITE: PartPixel = image::Rgba::<u8>([255, 255, 255, 255]);
pub const RED: PartPixel = image::Rgba::<u8>([255, 0, 0, 255]);
pub const GREEN: PartPixel = image::Rgba::<u8>([0, 255, 0, 255]);
pub const BLUE: PartPixel = image::Rgba::<u8>([0, 0, 255, 255]);
pub const HALF_WHITE: PartPixel = image::Rgba::<u8>([255, 255, 255, 128]);
pub const HALF_YELLOW: PartPixel = image::Rgba::<u8>([255, 255, 0, 128]);

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

#[derive(Clone, Debug, Deserialize)]
pub struct WidgetConf {
    pub x: u32,
    pub y: u32,
    pub visible: Option<bool>,
    #[serde(flatten)]
    pub widget: Widget,
}

const DEFAULT_WIDTH: u32 = 64;
const DEFAULT_HEIGHT: u32 = 64;

pub const DEFAULT_GIF1_ID: usize = 1;
pub const DEFAULT_GIF2_ID: usize = 2;
pub const DEFAULT_GIF3_ID: usize = 3;
pub const DEFAULT_GIF4_ID: usize = 4;
pub const DEFAULT_FLYER_ID: usize = 7;

pub(crate) fn deserialize_pixel<'de, D>(deserializer: D) -> Result<PartPixel, D::Error>
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

pub(crate) fn fill<P, Container>(image: &mut ImageBuffer<P, Container>, color: P)
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: std::ops::Deref<Target = [P::Subpixel]> + std::ops::DerefMut,
{
    for p in image.pixels_mut() {
        (*p) = color;
    }
}

#[cfg(test)]
mod tests {
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
}
