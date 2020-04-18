#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::impls::SimpleError;
pub use crate::json::{JsonObject, JsonValue, NumberValue, Serialize};
pub use crate::json_parser::Json;
pub use crate::parser::{Parser, ParserContext, ParserOptions};
pub use crate::parse::{parse_json, parse_json_with_options};

pub mod impls;
pub mod json;
pub mod json_parser;
pub mod parser;

mod parse;
