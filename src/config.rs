use std::time::Duration;

use rumqttc::{MqttOptions, AsyncClient, QoS, EventLoop};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "neoclock", about = "LED Matrix Clock.")]
pub struct Config {
    #[structopt(short, long)]
    pub config: Option<String>,

    #[structopt(short = "r", long = "refresh-rate", default_value = "60")]
    pub fps: u64,

    #[structopt(short, long, help = "MQTT Broker Host Name")]
    host: Option<String>,

    #[structopt(short, long, help = "Device Id")]
    device_id: Option<String>,

    #[structopt(short, long, help = "Password")]
    password: Option<String>,

    #[structopt(short, long, help = "Use TLS")]
    use_tls: bool,

    #[structopt(short, long, default_value = "neoclock", help = "MQTT Topic")]
    topic: String,
}

impl Config {
    fn get_host(&self) -> String {
        match &self.host {
            Some(s) => s.to_owned(),
            None => std::env::var("NEOCLOCK_HOSTNAME").unwrap_or_default(),
        }
    }

    fn get_device_id(&self) -> String {
        match &self.device_id {
            Some(s) => s.to_owned(),
            None => std::env::var("NEOCLOCK_DEVICE_ID").unwrap_or_default(),
        }
    }

    fn get_password(&self) -> String {
        match &self.password {
            Some(s) => s.to_owned(),
            None => std::env::var("NEOCLOCK_PASSWORD").unwrap_or_default(),
        }
    }

    pub async fn get_receiver(&self) -> anyhow::Result<EventLoop> {
        let mut mqttoptions = MqttOptions::new(self.get_device_id(), self.get_host(), if self.use_tls { 8883 } else { 1883 });
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        if !self.get_password().is_empty() {
            mqttoptions.set_credentials(self.get_device_id(), self.get_password());
        }
        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
        client.subscribe(self.topic.clone(), QoS::AtMostOnce).await?;
        Ok(eventloop)
    }
}
