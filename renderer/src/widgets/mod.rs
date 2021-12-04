mod font;
mod solid_widget;
mod clock_widget;
mod calendar_widget;
mod matrix_rain_widget;
mod gif_widget;

use async_trait::async_trait;
use serde::Deserialize;
use crate::{Part, PartCache, RenderError};

pub use font::draw_text;
pub use solid_widget::SolidWidget;
pub use clock_widget::ClockWidget;
pub use calendar_widget::CalendarWidget;
pub use matrix_rain_widget::MatrixRainWidget;
pub use gif_widget::GifWidget;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Widget {
    Solid(SolidWidget),
    Clock(ClockWidget),
    Calendar(CalendarWidget),
    MatrixRain(MatrixRainWidget),
    Gif(GifWidget)
}

#[async_trait]
impl Part for Widget {
    async fn start(&mut self, cache: PartCache, id: usize) -> Result<(), RenderError> {
        match self {
            Self::Solid(s) => s.start(cache, id).await,
            Self::Clock(s) => s.start(cache, id).await,
            Self::Calendar(s) => s.start(cache, id).await,
            Self::MatrixRain(s) => s.start(cache, id).await,
            Self::Gif(s) => s.start(cache, id).await,
        }
    }
}