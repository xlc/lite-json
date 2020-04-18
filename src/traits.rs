#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::json::JsonValue;

pub trait Serialize {
    fn serialize(&self) -> Vec<u8> {
        let mut res = Vec::new();
        self.serialize_to(&mut res, 0, 0);
        res
    }
    fn format(&self, indent: u32) -> Vec<u8> {
        let mut res = Vec::new();
        self.serialize_to(&mut res, indent, 0);
        res
    }
    fn serialize_to(&self, buffer: &mut Vec<u8>, indent: u32, level: u32);
}

pub trait IntoJson {
    fn into_json(self) -> JsonValue;
}

pub trait FromJson: Sized {
    fn from_json(value: JsonValue) -> Option<Self>;
}

// TODO: implement IntoJson & FromJson for common types such as &str, integers, Vec, Box, Option, Result, etc
