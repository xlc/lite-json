#![cfg_attr(not(feature = "std"), no_std)]

pub mod json;
pub mod json_parser;
pub mod traits;

pub use crate::json::*;
pub use crate::json_parser::*;
pub use crate::traits::*;

#[cfg(feature = "lite-json-derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate lite_json_derive;

#[cfg(feature = "lite-json-derive")]
pub use lite_json_derive::*;
