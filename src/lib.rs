#![cfg_attr(not(feature = "std"), no_std)]

use crate::impls::SimpleError;
use crate::json::{Json, JsonValue};
use crate::parser::Parser;

pub mod impls;
pub mod json;
pub mod parser;

pub fn parse_json(input: &str) -> Result<JsonValue, SimpleError> {
    Json::parse(&input, Default::default()).map(|(ret, _)| ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::NumberValue;

    #[test]
    fn it_works() {
        assert_eq!(
            parse_json(&r#"{ "test": 1, "test2": [1e-4, 2.041e2, true, false, null, "\"1\n\""] }"#),
            Ok(JsonValue::Object(vec![
                (
                    vec!['t', 'e', 's', 't'],
                    JsonValue::Number(NumberValue {
                        integer: 1,
                        fraction: 0,
                        fraction_length: 0,
                        exponent: 0
                    })
                ),
                (
                    vec!['t', 'e', 's', 't', '2'],
                    JsonValue::Array(vec![
                        JsonValue::Number(NumberValue {
                            integer: 1,
                            fraction: 0,
                            fraction_length: 0,
                            exponent: -4,
                        }),
                        JsonValue::Number(NumberValue {
                            integer: 2,
                            fraction: 41,
                            fraction_length: 3,
                            exponent: 2,
                        }),
                        JsonValue::Boolean(true),
                        JsonValue::Boolean(false),
                        JsonValue::Null,
                        JsonValue::String(vec!['\"', '1', 'n', '\"'])
                    ])
                )
            ]))
        )
    }
}
