use std::time::SystemTime;

use borsh::{from_slice, to_vec};
use borsh_derive::{BorshDeserialize, BorshSerialize};
use egui::Pos2;

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct Payload {
    pub x: f32,
    pub y: f32,
    pub time: u128,
}

impl Into<Vec<u8>> for Payload {
    fn into(self) -> Vec<u8> {
        to_vec(&self).unwrap()
    }
}

impl From<&Vec<u8>> for Payload {
    fn from(value: &Vec<u8>) -> Self {
        from_slice::<Payload>(value).unwrap()
    }
}

impl Into<Pos2> for Payload {
    fn into(self) -> Pos2 {
        Pos2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<&Pos2> for Payload {
    fn from(value: &Pos2) -> Self {
        Payload {
            x: value.x,
            y: value.y,
            time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }
}
