#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

use crate::traits::Serialize;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, PartialEq)]
pub struct NumberValue {
    pub integer: i64,
    pub fraction: u64,
    pub fraction_length: u32,
    pub exponent: i32,
}

impl NumberValue {
    /// Losslessly convert the inner value to `f64`.
    #[cfg(any(feature = "std", feature = "float"))]
    pub fn to_f64(self) -> f64 {
        self.into()
    }
}

#[cfg(any(feature = "std", feature = "float"))]
impl Into<f64> for NumberValue {
    fn into(self) -> f64 {
        #[cfg(not(feature = "std"))]
        use num_traits::float::FloatCore as _;

        (self.integer as f64 + self.fraction as f64 / 10f64.powi(self.fraction_length as i32))
            * 10f64.powi(self.exponent)
    }
}

pub type JsonObject = Vec<(Vec<char>, JsonValue)>;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, PartialEq)]
pub enum JsonValue {
    Object(JsonObject),
    Array(Vec<JsonValue>),
    String(Vec<char>),
    Number(NumberValue),
    Boolean(bool),
    Null,
}

impl Serialize for NumberValue {
    fn serialize_to(&self, buffer: &mut Vec<u8>, _indent: u32, _level: u32) {
        buffer.extend_from_slice(self.integer.to_string().as_bytes());

        if self.fraction > 0 {
            buffer.push('.' as u8);

            let fraction_nums = self.fraction.to_string();
            let fraction_length = self.fraction_length as usize;
            for _ in 0..fraction_length - fraction_nums.len() {
                buffer.push('0' as u8);
            }
            buffer.extend_from_slice(fraction_nums.as_bytes())
        }
        if self.exponent != 0 {
            buffer.push('e' as u8);
            if self.exponent < 0 {
                buffer.push('-' as u8);
            }
            buffer.extend_from_slice(self.exponent.abs().to_string().as_bytes());
        }
    }
}

fn push_string(buffer: &mut Vec<u8>, chars: &Vec<char>) {
    buffer.push('"' as u8);
    for ch in chars {
        match ch {
            '\x08' => buffer.extend_from_slice(br#"\b"#),
            '\x0c' => buffer.extend_from_slice(br#"\f"#),
            '\n' => buffer.extend_from_slice(br#"\n"#),
            '\r' => buffer.extend_from_slice(br#"\r"#),
            '\t' => buffer.extend_from_slice(br#"\t"#),
            '\"' => buffer.extend_from_slice(br#"\""#),
            '\\' => buffer.extend_from_slice(br#"\\"#),
            _ => match ch.len_utf8() {
                1 => {
                    let mut buff = [0u8; 1];
                    ch.encode_utf8(&mut buff);
                    buffer.push(buff[0]);
                }
                2 => {
                    let mut buff = [0u8; 2];
                    ch.encode_utf8(&mut buff);
                    buffer.extend_from_slice(&buff);
                }
                3 => {
                    let mut buff = [0u8; 3];
                    ch.encode_utf8(&mut buff);
                    buffer.extend_from_slice(&buff);
                }
                4 => {
                    let mut buff = [0u8; 4];
                    ch.encode_utf8(&mut buff);
                    buffer.extend_from_slice(&buff);
                }
                _ => panic!("Invalid UTF8 character"),
            },
        }
    }
    buffer.push('"' as u8);
}

fn push_new_line_indent(buffer: &mut Vec<u8>, indent: u32, level: u32) {
    if indent > 0 {
        buffer.push('\n' as u8);
    }
    let count = (indent * level) as usize;
    buffer.reserve(count);
    for _ in 0..count {
        buffer.push(' ' as u8);
    }
}

impl Serialize for JsonValue {
    fn serialize_to(&self, buffer: &mut Vec<u8>, indent: u32, level: u32) {
        match self {
            JsonValue::Object(obj) => {
                buffer.push('{' as u8);
                if obj.len() > 0 {
                    push_new_line_indent(buffer, indent, level + 1);
                    push_string(buffer, &obj[0].0);
                    buffer.push(':' as u8);
                    if indent > 0 {
                        buffer.push(' ' as u8);
                    }
                    obj[0].1.serialize_to(buffer, indent, level + 1);
                    for (key, val) in obj.iter().skip(1) {
                        buffer.push(',' as u8);
                        push_new_line_indent(buffer, indent, level + 1);
                        push_string(buffer, key);
                        buffer.push(':' as u8);
                        if indent > 0 {
                            buffer.push(' ' as u8);
                        }
                        val.serialize_to(buffer, indent, level + 1);
                    }
                    push_new_line_indent(buffer, indent, level);
                    buffer.push('}' as u8);
                } else {
                    buffer.push('}' as u8);
                }
            }
            JsonValue::Array(arr) => {
                buffer.push('[' as u8);
                if arr.len() > 0 {
                    push_new_line_indent(buffer, indent, level + 1);
                    arr[0].serialize_to(buffer, indent, level + 1);
                    for val in arr.iter().skip(1) {
                        buffer.push(',' as u8);
                        push_new_line_indent(buffer, indent, level + 1);
                        val.serialize_to(buffer, indent, level);
                    }
                    push_new_line_indent(buffer, indent, level);
                    buffer.push(']' as u8);
                } else {
                    buffer.push(']' as u8);
                }
            }
            JsonValue::String(str) => push_string(buffer, str),
            JsonValue::Number(num) => num.serialize_to(buffer, indent, level),
            JsonValue::Boolean(true) => buffer.extend_from_slice(b"true"),
            JsonValue::Boolean(false) => buffer.extend_from_slice(b"false"),
            JsonValue::Null => buffer.extend_from_slice(b"null"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_number_value() {
        let val = NumberValue {
            integer: 1234,
            fraction: 0,
            fraction_length: 0,
            exponent: 0,
        };
        assert_eq!(val.serialize(), b"1234");

        let val = NumberValue {
            integer: -1234,
            fraction: 0,
            fraction_length: 0,
            exponent: 0,
        };
        assert_eq!(val.serialize(), b"-1234");

        let val = NumberValue {
            integer: -1234,
            fraction: 5678,
            fraction_length: 4,
            exponent: 0,
        };
        assert_eq!(val.serialize(), b"-1234.5678");

        let val = NumberValue {
            integer: 1234,
            fraction: 1,
            fraction_length: 3,
            exponent: 0,
        };
        assert_eq!(val.serialize(), b"1234.001");

        let val = NumberValue {
            integer: 1234,
            fraction: 0,
            fraction_length: 0,
            exponent: 3,
        };
        assert_eq!(val.serialize(), b"1234e3");

        let val = NumberValue {
            integer: 1234,
            fraction: 0,
            fraction_length: 0,
            exponent: -5,
        };
        assert_eq!(val.serialize(), b"1234e-5");

        let val = NumberValue {
            integer: 1234,
            fraction: 56,
            fraction_length: 4,
            exponent: -5,
        };
        assert_eq!(val.serialize(), b"1234.0056e-5");

        let val = NumberValue {
            integer: -1234,
            fraction: 5,
            fraction_length: 2,
            exponent: 5,
        };
        assert_eq!(val.serialize(), b"-1234.05e5");
    }

    #[test]
    fn serialize_works() {
        let obj = JsonValue::Object(vec![(
            "test\"123".chars().into_iter().collect(),
            JsonValue::Null,
        )]);
        assert_eq!(
            std::str::from_utf8(&obj.format(4)[..]).unwrap(),
            r#"{
    "test\"123": null
}"#
        );

        let obj = JsonValue::Object(vec![
            (
                vec!['t', 'e', 's', 't'],
                JsonValue::Number(NumberValue {
                    integer: 123,
                    fraction: 4,
                    fraction_length: 2,
                    exponent: 0,
                }),
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
                    JsonValue::String(vec!['\"', '1', 'n', '\"']),
                    JsonValue::Object(vec![]),
                    JsonValue::Array(vec![]),
                ]),
            ),
        ]);

        assert_eq!(
            std::str::from_utf8(&obj.format(4)[..]).unwrap(),
            r#"{
    "test": 123.04,
    "test2": [
        1e-4,
        2.041e2,
        true,
        false,
        null,
        "\"1n\"",
        {},
        []
    ]
}"#
        );

        assert_eq!(
            std::str::from_utf8(&obj.serialize()[..]).unwrap(),
            r#"{"test":123.04,"test2":[1e-4,2.041e2,true,false,null,"\"1n\"",{},[]]}"#
        );
    }
}
