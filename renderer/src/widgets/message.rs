use std::time::Instant;

use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

use crate::{PartCache, PartSender, PartPixel, deserialize_pixel, serialize_pixel};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum NeoClockMessage {
    Show(usize),
    Hide(usize),
    Move(MoveMessage),
    Solid{
        id: usize,
        #[serde(flatten)]
        msg: SolidMessage,
    },
    Clock{
        id: usize,
        #[serde(flatten)]
        msg: ClockMessage,
    },
    Calendar{
        id: usize,
        #[serde(flatten)]
        msg: CalendarMessage,
    },
    Gif {
        id: usize,
        #[serde(flatten)]
        msg: GifMessage,
    },
    Flyer {
        id: usize,
        #[serde(flatten)]
        msg: FlyerMessage,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MoveMessage {
    pub id: usize,
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SolidMessage {
    #[serde(deserialize_with = "deserialize_pixel", serialize_with = "serialize_pixel")]
    color: PartPixel,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClockMessage;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CalendarMessage;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GifMessage {
    pub url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FlyerMessage {
    pub text: String,
    pub ttl: u32,

    #[serde(skip)]
    pub(crate) expiration: Option<Instant>,
}

pub async fn msg_task(mut receiver: Receiver<NeoClockMessage>, part_senders: Vec<PartSender>, parts: Vec<PartCache>) {
    loop {
        if let Some(msg)= receiver.recv().await {
            msg_handler(&part_senders, &parts, msg).await;
        }
    }
}

pub async fn msg_handler(senders: &[PartSender], parts: &[PartCache], msg: NeoClockMessage) {
    match msg {
        NeoClockMessage::Gif{id, msg: m} => {
            info!("Sending Gif message '{:#?}' to widget {}", m, id);
            senders[id].send(serde_json::to_string(&m).unwrap()).await.unwrap_or_default();
        },
        NeoClockMessage::Flyer{id, msg: m} => {
            info!("Sending Flyer message '{:#?}' to widget {}", m, id);
            senders[id].send(serde_json::to_string(&m).unwrap()).await.unwrap_or_default();
        }
        NeoClockMessage::Solid{id, msg: m} => {
            info!("Sending Solid message '{:#?}' to widget {}", m, id);
            senders[id].send(serde_json::to_string(&m).unwrap()).await.unwrap_or_default();
        }
        NeoClockMessage::Clock{id, msg: m} => {
            info!("Sending Clock message '{:#?}' to widget {}", m, id);
            senders[id].send(serde_json::to_string(&m).unwrap()).await.unwrap_or_default();
        }
        NeoClockMessage::Calendar{id, msg: m} => {
            info!("Sending Calendar message '{:#?}' to widget {}", m, id);
            senders[id].send(serde_json::to_string(&m).unwrap()).await.unwrap_or_default();
        }
        NeoClockMessage::Show(id) => {
            if id < parts.len() {
                if let Ok(mut write_guard) = parts[id].write() {
                    write_guard.visible = true;
                }
            }    
        },
        NeoClockMessage::Hide(id) => {
            if id < parts.len() {
                if let Ok(mut write_guard) = parts[id].write() {
                    write_guard.visible = false;
                }
            }    
        },
        NeoClockMessage::Move(MoveMessage { id, x, y }) => {
            if id < parts.len() {
                if let Ok(mut write_guard) = parts[id].write() {
                    write_guard.x = x;
                    write_guard.y = y;
                }
            }    
        },
    };
}

#[cfg(test)]
mod tests {
    use super::NeoClockMessage;

    #[test]
    fn test_msg() {
        let s=r#"{"type":"Flyer","id":10,"text":"Hahaha","ttl":10}"#;
        let msg = serde_json::from_str::<NeoClockMessage>(s).unwrap();
        match msg {
            NeoClockMessage::Flyer{id, msg} => {
                assert_eq!(id, 10);
                assert_eq!(msg.text, "Hahaha");
                assert_eq!(msg.ttl, 10);
            },
            _ => {
                panic!();
            }
        }
    }
}