use std::{time::Duration, fs::File, io::BufReader};
use anyhow::Result;
use clap::App;
use rpi_led_matrix::{LedMatrixOptions, LedMatrix, LedCanvas, LedColor};

use renderer::{Drawable, Screen, WidgetConf};

struct Canvas(LedCanvas);

impl Drawable for Canvas {
    fn set_pixel(&mut self, x: u32, y: u32, red: u8, green: u8, blue: u8) {
        self.0.set(x as i32, y as i32, &LedColor { red, green, blue });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("neoclock")
    .version("1.0")
    .author("Chen Xu <windoze@0d0a.com>")
    .about("LED Clock")
    .args_from_usage("-c, --config=[FILE] 'Sets a custom config file'")
    .get_matches();
    let config = matches.value_of("config").unwrap_or("default.conf");
    println!("Value for config: {}", config);
    let file = File::open(config)?;
    let reader = BufReader::new(file);
    let parts: Vec<WidgetConf> = serde_json::from_reader(reader).unwrap();

    let mut options = LedMatrixOptions::new();
    options.set_hardware_mapping("adafruit-hat-pwm");
    // Why the default value is set to 1000?
    options.set_pwm_lsb_nanoseconds(130);
    options.set_pixel_mapper_config("Rotate:180");
    options.set_cols(64);
    options.set_rows(64);
    // options.set_refresh_rate(false);
    let matrix = LedMatrix::new(Some(options), None).unwrap();
    let screen = Screen::new(64, 64, parts);
    let mut canvas = Canvas(matrix.offscreen_canvas());
    loop {
        screen.render_to(&mut canvas);
        canvas = Canvas(matrix.swap(canvas.0));
        tokio::time::sleep(Duration::from_millis(33)).await;
    }
}
