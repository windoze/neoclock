use anyhow::Result;
use clap::{App, Arg, value_t};
use log::info;
use rpi_led_matrix::{LedCanvas, LedColor, LedMatrix, LedMatrixOptions};
use std::{fs::File, io::BufReader, time::Duration};

use renderer::{Drawable, Screen, WidgetConf};

struct Canvas(LedCanvas);

impl Drawable for Canvas {
    fn set_pixel(&mut self, x: u32, y: u32, red: u8, green: u8, blue: u8) {
        self.0
            .set(x as i32, y as i32, &LedColor { red, green, blue });
    }
}

#[link(name = "c")]
extern "C" {
    fn geteuid() -> u32;
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let matches = App::new("neoclock")
        .version("1.0")
        .author("Chen Xu <windoze@0d0a.com>")
        .about("LED Clock")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("refresh-rate")
                .short("r")
                .long("refresh-rate")
                .value_name("FPS")
                .help("Sets refresh rate")
                .default_value("30")
                .takes_value(true),
        )
        .get_matches();
    let config = matches.value_of("config").unwrap_or("config.json");
    let fps = value_t!(matches.value_of("refresh-rate"), u64)?;
    info!("Using config file at '{}'.", config);
    let file = File::open(config)?;
    let reader = BufReader::new(file);
    let parts: Vec<WidgetConf> = serde_json::from_reader(reader)?;

    let mut options = LedMatrixOptions::new();
    options.set_hardware_mapping("adafruit-hat-pwm");
    options.set_hardware_pulsing(unsafe { geteuid() } == 0);
    // Why the default value is set to 1000?
    options.set_pwm_lsb_nanoseconds(130);
    options.set_pixel_mapper_config("Rotate:180");
    options.set_cols(64);
    options.set_rows(64);
    options.set_refresh_rate(false);
    let matrix = LedMatrix::new(Some(options), None).unwrap();
    
    let screen = Screen::new(64, 64, parts);
    let mut canvas = Canvas(matrix.offscreen_canvas());
    loop {
        screen.render_to(&mut canvas);
        canvas = Canvas(matrix.swap(canvas.0));
        tokio::time::sleep(Duration::from_millis(1000 / fps)).await;
    }
}
