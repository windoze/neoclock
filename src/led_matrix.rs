use rpi_led_matrix::{LedCanvas, LedColor, LedMatrix, LedMatrixOptions};

use crate::StringError;

pub struct Canvas(LedCanvas);

impl renderer::Drawable for Canvas {
    fn set_pixel(&mut self, x: u32, y: u32, red: u8, green: u8, blue: u8) {
        self.0
            .set(x as i32, y as i32, &LedColor { red, green, blue });
    }
}

pub struct Matrix(LedMatrix);

impl crate::Display for Matrix {
    type Canvas = Canvas;

    fn init() -> anyhow::Result<Self> {
        let mut options = LedMatrixOptions::new();
        options.set_hardware_mapping("adafruit-hat-pwm");
        options.set_hardware_pulsing(unsafe { geteuid() } == 0);
        // Why the default value is set to 1000?
        options.set_pwm_lsb_nanoseconds(130);
        options.set_pixel_mapper_config("Rotate:180");
        options.set_cols(64);
        options.set_rows(64);
        options.set_refresh_rate(false);
        match LedMatrix::new(Some(options), None) {
            Ok(matrix) => Ok(Matrix(matrix)),
            Err(e) => {
                Err(StringError(format!("LED Matrix initialization failed, error is '{}'.", e)).into)
            }
        }
    }

    fn get_canvas(&self) -> Self::Canvas {
        Canvas(self.0.offscreen_canvas())
    }

    fn swap(&mut self, canvas: Self::Canvas) -> anyhow::Result<Self::Canvas> {
        Ok(Canvas(self.0.swap(canvas.0)))
    }
}

#[link(name = "c")]
extern "C" {
    fn geteuid() -> u32;
}
