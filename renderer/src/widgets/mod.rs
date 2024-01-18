mod calendar_widget;
mod clock_widget;
mod flyer_widget;
mod font;
mod gif_widget;
mod matrix_rain_widget;
mod solid_widget;
mod wigwag_widget;
pub mod message;

use crate::{Part, PartCache, PartChannel, RenderError};
use async_trait::async_trait;
use serde::Deserialize;

pub use calendar_widget::CalendarWidget;
pub use clock_widget::ClockWidget;
pub use flyer_widget::FlyerWidget;
pub use font::FontConfig;
pub use gif_widget::GifWidget;
pub use matrix_rain_widget::MatrixRainWidget;
pub use solid_widget::SolidWidget;
pub use wigwag_widget::WigwagWidget;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Widget {
    Solid(SolidWidget),
    Clock(ClockWidget),
    Calendar(CalendarWidget),
    MatrixRain(MatrixRainWidget),
    Gif(GifWidget),
    Flyer(FlyerWidget),
    Wigwag(WigwagWidget),
}

#[async_trait]
impl Part for Widget {
    async fn start(
        &mut self,
        cache: PartCache,
        id: usize,
        channel: PartChannel,
    ) -> Result<(), RenderError> {
        match self {
            Self::Solid(s) => s.start(cache, id, channel).await,
            Self::Clock(s) => s.start(cache, id, channel).await,
            Self::Calendar(s) => s.start(cache, id, channel).await,
            Self::MatrixRain(s) => s.start(cache, id, channel).await,
            Self::Gif(s) => s.start(cache, id, channel).await,
            Self::Flyer(s) => s.start(cache, id, channel).await,
            Self::Wigwag(s) => s.start(cache, id, channel).await,
        }
    }
}
