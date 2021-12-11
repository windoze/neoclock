mod config;

use anyhow::Result;
use log::{error, info, warn};
use rpi_led_matrix::{LedCanvas, LedColor, LedMatrix, LedMatrixOptions};
use rumqttc::{Event, Outgoing, Packet};
use std::{fs::File, io::BufReader, time::Duration};
use structopt::StructOpt;

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

fn init_matrix() -> Result<LedMatrix> {
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
        Ok(matrix) => Ok(matrix),
        Err(e) => {
            panic!("LED Matrix initialization failed, error is '{}'.", e);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let opt = config::Config::from_args();
    let screen = match &opt.config {
        Some(s) => {
            info!("Using config file at '{}'.", s);
            let file = File::open(s)?;
            let reader = BufReader::new(file);
            let parts: Vec<WidgetConf> = serde_json::from_reader(reader)?;
            Screen::new(64, 64, parts)
        }
        None => Screen::default(),
    };

    let matrix = init_matrix()?;
    let mut canvas = Canvas(matrix.offscreen_canvas());

    let mut receiver = opt.get_receiver().await?;
    let sender = screen.sender.clone();

    tokio::spawn(async move {
        loop {
            // NOTE:
            // receiver.poll() blocks for few seconds every time, we need to move it to another seperated task (or thread?)
            match receiver.poll().await {
                // Ignore routine messages to avoid log flooding
                Ok(Event::Incoming(Packet::PingResp)) => {}
                Ok(Event::Outgoing(Outgoing::PingReq)) => {}
                Ok(Event::Incoming(Packet::Publish(msg))) => {
                    info!(
                        "Got message: '{}({})'",
                        &msg.topic,
                        std::str::from_utf8(&msg.payload).unwrap()
                    );
                    if let Ok(m) =
                        serde_json::from_slice::<renderer::message::NeoClockMessage>(&msg.payload)
                    {
                        sender.send(m).await.unwrap_or_default();
                    } else {
                        warn!("Received invalid message.");
                    }
                }
                Ok(x) => {
                    info!("Got unwanted message: '{:#?}'", x);
                }
                Err(e) => {
                    error!("Connection error: '{:#?}'", e)
                }
            }
        }
    });

    loop {
        screen.render_to(&mut canvas);
        canvas = Canvas(matrix.swap(canvas.0));
        tokio::time::sleep(Duration::from_millis(1000 / opt.fps)).await
    }
}
