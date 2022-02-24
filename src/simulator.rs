use embedded_graphics::{pixelcolor::Rgb888, prelude::*, Drawable};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, Window, SimulatorEvent,
};

use crate::{Display, StringError};

pub struct Canvas(SimulatorDisplay<Rgb888>);

impl renderer::Drawable for Canvas {
    fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        let p = Pixel(Point::new(x as i32, y as i32), Rgb888::new(r, g, b));
        p.draw(&mut self.0)
        .unwrap();
    }
}

pub struct Matrix(Window);

impl Display for Matrix {
    type Canvas = Canvas;

    fn init() -> anyhow::Result<Self> {
        let output_settings = OutputSettingsBuilder::new().scale(8).build();
        let window = Window::new("NeoClock Simulator", &output_settings);
        Ok(Matrix(window))
    }

    fn get_canvas(&self) -> Self::Canvas {
        Canvas(SimulatorDisplay::<Rgb888>::new(Size::new(64, 64)))
    }

    fn swap(&mut self, canvas: Self::Canvas) -> anyhow::Result<Self::Canvas> {
        self.0.update(&canvas.0);
        if self.0.events().any(|e| e == SimulatorEvent::Quit) {
            Err(StringError("Quit".to_string()).into())
        } else {
            Ok(canvas)
        }
    }
}
