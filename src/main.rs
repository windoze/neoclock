mod config;

use anyhow::Result;
use log::{error, info, warn};
use rumqttc::{Event, Outgoing, Packet};
use std::{fs::File, io::BufReader, time::Duration};
use structopt::StructOpt;

use renderer::{Drawable, Screen, WidgetConf};

trait Display
where
    Self: Sized,
{
    type Canvas: Drawable;
    fn init() -> anyhow::Result<Self>;
    fn get_canvas(&self) -> Self::Canvas;
    fn swap(&mut self, canvas: Self::Canvas) -> anyhow::Result<Self::Canvas>;
}

#[cfg(feature = "rpi")]
mod led_matrix;
#[cfg(feature = "rpi")]
use led_matrix::Matrix;

#[cfg(all(feature = "simulator", not(feature = "rpi")))]
mod simulator;
#[cfg(all(feature = "simulator", not(feature = "rpi")))]
use simulator::Matrix;

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

    let mut matrix = Matrix::init()?;
    let mut canvas = matrix.get_canvas();

    let mut receiver = opt.get_receiver().await?;
    let sender = screen.sender.clone();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.spawn(async move {
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
        canvas = match matrix.swap(canvas) {
            Ok(c) => c,
            Err(_) => {
                rt.shutdown_background();
                break;
            }
        };
        tokio::time::sleep(Duration::from_millis(1000 / opt.fps)).await
    }
    Ok(())
}
