use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

use crate::{DEFAULT_FLYER_ID, PartCache, PartSender};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum NeoClockMessage {
    Show(usize),
    Hide(usize),
    Move(MoveMessage),
    Gif {
        id: usize,
        #[serde(flatten)]
        msg: GifMessage,
    },
    Flyer(FlyerMessage),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MoveMessage {
    pub id: usize,
    pub x: u32,
    pub y: u32,
}

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
            senders[id].send(serde_json::to_string(&m).unwrap()).await.unwrap_or_default();
        },
        NeoClockMessage::Flyer(m) => {
            senders[DEFAULT_FLYER_ID].send(serde_json::to_string(&m).unwrap()).await.unwrap_or_default();
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
        let s=r#"{"type":"Flyer","text":"Hahaha","ttl":10}"#;
        let msg = serde_json::from_str::<NeoClockMessage>(s).unwrap();
        match msg {
            NeoClockMessage::Flyer(m) => {
                assert_eq!(m.text, "Hahaha");
                assert_eq!(m.ttl, 10);
            },
            _ => {
                panic!();
            }
        }
    }
}