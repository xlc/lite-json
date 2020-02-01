#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::impls::SimpleError;
pub use crate::json_parser::{Json, JsonValue};
pub use crate::parser::{Parser, ParserContext, ParserOptions};

pub mod impls;
pub mod json;
pub mod json_parser;
pub mod parser;

pub fn parse_json(input: &str) -> Result<JsonValue, SimpleError> {
    parse_json_with_options(input, Default::default())
}

pub fn parse_json_with_options(
    input: &str,
    options: ParserOptions,
) -> Result<JsonValue, SimpleError> {
    Json::parse(&input, Default::default(), &ParserContext::new(options)).map(|(ret, _)| ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impls::SimplePosition;
    use crate::json_parser::NumberValue;

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

    #[test]
    fn it_should_consume_all() {
        assert_eq!(
            parse_json(&r#""1"a"#),
            Err(SimpleError {
                reasons: vec![(
                    SimplePosition {
                        index: 3,
                        line: 0,
                        column: 3
                    },
                    "Expect end of input"
                )]
            })
        )
    }

    #[test]
    fn it_accepts_nest_level() {
        assert_eq!(
            parse_json_with_options(
                &r#"{ "test": 1 }"#,
                ParserOptions {
                    max_nest_level: Some(1)
                }
            ),
            Ok(JsonValue::Object(vec![(
                vec!['t', 'e', 's', 't'],
                JsonValue::Number(NumberValue {
                    integer: 1,
                    fraction: 0,
                    fraction_length: 0,
                    exponent: 0
                })
            ),]))
        );
    }

    #[test]
    fn it_accepts_more_nest_level() {
        assert_eq!(
            parse_json_with_options(
                &r#"{ "test": { "a": [ {} ] } }"#,
                ParserOptions {
                    max_nest_level: Some(5)
                }
            ),
            Ok(JsonValue::Object(vec![(
                vec!['t', 'e', 's', 't'],
                JsonValue::Object(vec![(
                    vec!['a'],
                    JsonValue::Array(vec![JsonValue::Object(vec![])])
                )])
            )]))
        );
    }

    #[test]
    fn it_error_on_too_deep_nest() {
        assert_eq!(
            parse_json_with_options(
                &r#"{ "test": { "a": [ {} ] } }"#,
                ParserOptions {
                    max_nest_level: Some(3)
                }
            ),
            Err(SimpleError {
                reasons: vec![(
                    SimplePosition {
                        index: 0,
                        line: 0,
                        column: 0
                    },
                    "Value"
                )]
            })
        );
    }
}
