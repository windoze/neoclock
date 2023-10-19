use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use image::{ImageBuffer, Pixel, buffer::ConvertBuffer};
use log::debug;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{task::JoinHandle, sync::mpsc::Sender};

use crate::{RenderError, message::{NeoClockMessage, msg_task}, WidgetConf, Widget, widgets::*, PartImage, DEFAULT_WIDTH, DEFAULT_HEIGHT, TRANSPARENT, HALF_WHITE, HALF_YELLOW, Drawable, BLACK, fill};

pub(crate) type ScreenPixel = image::Rgb<u8>;
pub(crate) type ScreenImage = ImageBuffer<ScreenPixel, Vec<u8>>;
pub(crate) type PartSender = tokio::sync::mpsc::Sender<String>;
pub(crate) type PartChannel = tokio::sync::mpsc::Receiver<String>;

#[async_trait]
pub(crate) trait Part {
    async fn start(
        &mut self,
        cache: PartCache,
        id: usize,
        mut channel: PartChannel,
    ) -> Result<(), RenderError>;

    async fn try_read<T>(&self, channel: &mut PartChannel, d: std::time::Duration) -> Option<T>
    where
        T: DeserializeOwned,
    {
        if let Ok(Some(s)) = tokio::time::timeout(d, channel.recv()).await {
            debug!("Message body: '{}'", s);
            serde_json::from_str(&s).ok()
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PartContent {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) visible: bool,
    pub(crate) image: Option<PartImage>,
}

pub(crate) type PartCache = Arc<RwLock<PartContent>>;

struct PartTask {
    content: PartCache,
    sender: PartSender,
    join_handler: JoinHandle<Result<(), RenderError>>,
}

pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub sender: Sender<NeoClockMessage>,
    parts: Vec<PartTask>,
}

impl Screen {
    pub fn new(width: u32, height: u32, widgets: Vec<WidgetConf>) -> Screen {
        debug!("Widget lists:");
        for (idx, w) in widgets.iter().enumerate() {
            match w.widget {
                Widget::Solid(_) => debug!("Widget {}: Solid", idx),
                Widget::Clock(_) => debug!("Widget {}: Clock", idx),
                Widget::Calendar(_) => debug!("Widget {}: Calendar", idx),
                Widget::MatrixRain(_) => debug!("Widget {}: MatrixRain", idx),
                Widget::Gif(_) => debug!("Widget {}: Gif", idx),
                Widget::Flyer(_) => debug!("Widget {}: Flyer", idx),
                Widget::Wigwag(_) => debug!("Widget {}: Wigwag", idx),
            }
        }

        let mut children: Vec<PartTask> = Vec::with_capacity(widgets.len());

        for (idx, mut w) in widgets.into_iter().enumerate() {
            let cache = Arc::new(RwLock::new(PartContent {
                x: w.x,
                y: w.y,
                visible: w.visible.unwrap_or(true),
                image: None,
            }));

            let (sender, receiver) = tokio::sync::mpsc::channel(100); // TODO:
            let mc = cache.clone();
            let join_handler = tokio::spawn(async move { w.widget.start(mc, idx, receiver).await });

            let part = PartTask {
                content: cache,
                sender,
                join_handler,
            };

            children.push(part);
        }

        let (sender, receiver) = tokio::sync::mpsc::channel(10);

        let part_senders: Vec<PartSender> = children.iter().map(|c| c.sender.clone()).collect();
        let part_contents: Vec<PartCache> = children.iter().map(|c| c.content.clone()).collect();
        tokio::spawn(async move {
            msg_task(receiver, part_senders, part_contents).await;
        });

        Self {
            width,
            height,
            sender,
            parts: children,
        }
    }

    pub async fn stop(&mut self) {
        for c in self.parts.iter_mut() {
            c.join_handler.abort();
            (&mut c.join_handler).await.unwrap().unwrap();
        }
    }

    fn render(&self) -> ScreenImage {
        let mut screen = PartImage::new(self.width, self.height);
        fill(&mut screen, BLACK);
        // Blend every visible part image into `screen`
        for part in self.parts.iter() {
            if let Ok(read_guard) = part.content.read() {
                if read_guard.visible {
                    if let Some(img) = &read_guard.image {
                        let x = read_guard.x;
                        let y = read_guard.y;
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
        self.parts[idx].sender.send(s).await?;
        Ok(())
    }

    pub async fn send<T>(&self, idx: usize, t: &T) -> Result<(), RenderError>
    where
        T: Serialize,
    {
        self.send_str(idx, serde_json::to_string(t)?).await
    }
}

impl Default for Screen {
    fn default() -> Screen {
        let parts = vec![
            WidgetConf {
                x: 0,
                y: 0,
                visible: Some(true),
                widget: Widget::Solid(SolidWidget {
                    width: DEFAULT_WIDTH,
                    height: DEFAULT_HEIGHT,
                    color: TRANSPARENT,
                }),
            },
            WidgetConf {
                x: 0,
                y: 0,
                visible: Some(true),
                widget: Widget::Gif(GifWidget {
                    location: "./robot.gif".to_string(),
                }),
            },
            WidgetConf {
                x: DEFAULT_WIDTH / 2,
                y: 0,
                visible: Some(true),
                widget: Widget::Gif(GifWidget {
                    location: Default::default(),
                }),
            },
            WidgetConf {
                x: 0,
                y: DEFAULT_HEIGHT / 2,
                visible: Some(true),
                widget: Widget::Gif(GifWidget {
                    location: Default::default(),
                }),
            },
            WidgetConf {
                x: DEFAULT_WIDTH / 2,
                y: DEFAULT_HEIGHT / 2,
                visible: Some(true),
                widget: Widget::Gif(GifWidget {
                    location: Default::default(),
                }),
            },
            WidgetConf {
                x: 0,
                y: 0,
                visible: Some(true),
                widget: Widget::Clock(ClockWidget {
                    width:DEFAULT_WIDTH,
                    height:DEFAULT_HEIGHT/2,
                    text_color: HALF_WHITE,
                    background_color: TRANSPARENT,
                    font_config: FontConfig {
                        font_path: Default::default(),
                        font_height: 20.5,
                        font_scale_x: 1.2,
                        font_scale_y: 1.0
                    }
                }),
            },
            WidgetConf {
                x: 0,
                y: DEFAULT_HEIGHT - 12,
                visible: Some(true),
                widget: Widget::Calendar(CalendarWidget {
                    width:DEFAULT_WIDTH,
                    height:DEFAULT_HEIGHT/2,
                    text_color: HALF_WHITE,
                    background_color: TRANSPARENT,
                    font_config: FontConfig {
                        font_path: Default::default(),
                        font_height: 12.4,
                        font_scale_x: 1.0,
                        font_scale_y: 1.0
                    }
                }),
            },
            WidgetConf {
                x: 0,
                y: 0,
                visible: Some(true),
                widget: Widget::Flyer(FlyerWidget {
                    width:DEFAULT_WIDTH,
                    height:DEFAULT_HEIGHT,
                    text_color: HALF_WHITE,
                    background_color: HALF_YELLOW,
                    speed: 100,
                    font_config: FontConfig {
                        font_path: Default::default(),
                        font_height: 12.4,
                        font_scale_x: 1.0,
                        font_scale_y: 1.0
                    }
                }),
            },
        ];
        Self::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, parts)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use image::{Rgba, Rgb};
    use crate::WidgetConf;
    use super::*;

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
