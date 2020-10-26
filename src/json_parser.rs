#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::json::{JsonObject, JsonValue, NumberValue};
use lite_parser::{
    impls::SimpleError,
    literals,
    parser::{
        Concat, Concat3, Either, OneOf, OneOrMore, Parser, ParserContext, ParserOptions,
        ZeroOrMore, ZeroOrOne,
    },
    parsers,
    traits::{Error, Input, ResultOf},
};

use core::convert::TryInto;

literals! {
    pub WhitespaceChar => '\u{0020}' | '\u{000D}' | '\u{000A}' | '\u{0009}';
    pub SignChar => '+' | '-';
    pub NegativeSignChar => '-';
    pub EChar => 'E' | 'e';
    pub OneToNineChar => '1' ..= '9';
    pub DigitChar => '0' ..= '9';
    pub DotChar => '.';
    pub HexChar => '0' ..= '9' | 'a' ..= 'f' | 'A' ..= 'F';
    pub DoubleQuoteChar => '"';
    pub OpenCurlyBracketChar => '{';
    pub CloseCurlyBracketChar => '}';
    pub CommaChar => ',';
    pub OpenSquareBracketChar => '[';
    pub CloseSquareBracketChar => ']';
}

pub type Whitespace = ZeroOrMore<WhitespaceChar>;

pub type Sign = ZeroOrOne<SignChar>;

pub type Digits = OneOrMore<DigitChar>;

parsers! {
    pub PositiveInteger = OneOf<Concat<OneToNineChar, Digits>, DigitChar>, u64, (output) => {
        match output {
            Either::A((c, cs)) => {
                let mut val = c.to_digit(10).unwrap() as u64;
                for c in cs {
                    val *= 10;
                    val += c.to_digit(10).unwrap() as u64;
                }
                val
            },
            Either::B(c) => c.to_digit(10).unwrap() as u64,
        }
    };

    pub NegativeInteger = Concat<NegativeSignChar, PositiveInteger>, i64, (output) => {
        let (_, output) = output;
        - (output as i64)
    };

    pub Integer = OneOf<PositiveInteger, NegativeInteger>, i64, (output) => {
        match output {
            Either::A(a) => a as i64,
            Either::B(b) => b,
        }
    };

    pub Fraction = ZeroOrOne<Concat<DotChar, Digits>>, (u64, u32), (output) => {
        match output {
            Either::A((_, cs)) => {
                let mut val = 0u64;
                let len = cs.len();
                for c in cs {
                    val *= 10u64;
                    val += c.to_digit(10).unwrap() as u64;
                }
                (val, len as u32)
            },
            Either::B(_) => (0u64, 0u32),
        }
    };

    pub Exponent = ZeroOrOne<Concat3<EChar, Sign, Digits>>, i32, (output) => {
        match output {
            Either::A((_, (s, cs))) => {
                let mul = if let Either::A('-') = s { -1 } else { 1 };
                let mut val = 0i32;
                for c in cs {
                    val *= 10;
                    val += c.to_digit(10).unwrap() as i32;
                }
                val * mul
            },
            Either::B(_) => 0,
        }
    };

    pub Number = Concat3<Integer, Fraction, Exponent>, NumberValue, (output) => {
        let (n, (f, e)) = output;
        NumberValue {
            integer: n,
            fraction: f.0,
            fraction_length: f.1,
            exponent: e,
        }
    };

    pub Hex = HexChar, u8, (output) => {
        output.to_digit(16).unwrap() as u8
    };

    pub String = Concat3<DoubleQuoteChar, Characters, DoubleQuoteChar>, Vec<char>, (output) => {
        match (output.1).0 {
            Either::A(bytes) => bytes,
            Either::B(_) => Vec::new(),
        }
    };
}

pub struct Escape;

impl<I: Input> Parser<I> for Escape {
    type Output = char;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let (c, next) = input
            .next(current)
            .map_err(|e| e.add_reason(current, "Escape"))?;
        match c {
            '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => Ok((c, next)),
            'u' => {
                let (b1, next) = <Hex as Parser<I>>::parse(input, next, context)?;
                let (b2, next) = <Hex as Parser<I>>::parse(input, next, context)?;
                let (b3, next) = <Hex as Parser<I>>::parse(input, next, context)?;
                let (b4, next) = <Hex as Parser<I>>::parse(input, next, context)?;
                let byte = (b1 as u32) << 24 | (b2 as u32) << 16 | (b3 as u32) << 8 | (b4 as u32);
                let c = byte
                    .try_into()
                    .map_err(|_| input.error_at(current, "Escape"))?;
                Ok((c, next))
            }
            _ => Err(input.error_at(current, "Escape")),
        }
    }
}

pub struct Character;

impl<I: Input> Parser<I> for Character {
    type Output = char;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let (c, next) = input
            .next(current)
            .map_err(|e| e.add_reason(current, "Character"))?;
        match c {
            '\\' => <Escape as Parser<I>>::parse(input, next, context),
            '"' => Err(input.error_at(current, "Character")),
            _ => Ok((c, next)),
        }
    }
}

pub type Characters = ZeroOrMore<Character>;

pub struct Member;

impl<I: Input> Parser<I> for Member {
    type Output = (Vec<char>, JsonValue);
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let (_, next) = <Whitespace as Parser<I>>::parse(input, current, context)?;
        let (key, next) = <String as Parser<I>>::parse(input, next, context)?;
        let (_, next) = <Whitespace as Parser<I>>::parse(input, next, context)?;
        let next = input
            .next(next)
            .and_then(|(c, next)| {
                if c == ':' {
                    Ok(next)
                } else {
                    Err(input.error_at(next, "Character"))
                }
            })
            .map_err(|e| e.add_reason(current, "Member"))?;
        let (value, next) = <Element as Parser<I>>::parse(input, next, context)?;
        Ok(((key, value), next))
    }
}

pub struct Element;

impl<I: Input> Parser<I> for Element {
    type Output = JsonValue;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let (_, next) = <Whitespace as Parser<I>>::parse(input, current, context)?;
        let (output, next) = <Value as Parser<I>>::parse(input, next, context)?;
        let (_, next) = <Whitespace as Parser<I>>::parse(input, next, context)?;
        Ok((output, next))
    }
}

pub struct Value;

impl<I: Input> Parser<I> for Value
where
    I::Position: Copy,
{
    type Output = JsonValue;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        if let Ok((output, next)) = <Object as Parser<I>>::parse(input, current, context) {
            return Ok((JsonValue::Object(output), next));
        }
        if let Ok((output, next)) = <Array as Parser<I>>::parse(input, current, context) {
            return Ok((JsonValue::Array(output), next));
        }
        if let Ok((output, next)) = <String as Parser<I>>::parse(input, current, context) {
            return Ok((JsonValue::String(output), next));
        }
        if let Ok((output, next)) = <Number as Parser<I>>::parse(input, current, context) {
            return Ok((JsonValue::Number(output), next));
        }
        let (value, next) = input.next_range(current, 4)?;
        if value == "null" {
            return Ok((JsonValue::Null, next));
        }
        if value == "true" {
            return Ok((JsonValue::Boolean(true), next));
        }
        let (value, next) = input.next_range(current, 5)?;
        if value == "false" {
            return Ok((JsonValue::Boolean(false), next));
        }
        Err(input.error_at(current, "Value"))
    }
}

pub struct Object;

impl<I: Input> Parser<I> for Object {
    type Output = JsonObject;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let context = &context.nest(input, current)?;
        let (_, next) = <OpenCurlyBracketChar as Parser<I>>::parse(input, current, context)?;
        let (output, next) =
            <OneOf<Members, Whitespace> as Parser<I>>::parse(input, next, context)?;
        let (_, next) = <CloseCurlyBracketChar as Parser<I>>::parse(input, next, context)?;
        let output = match output {
            Either::A(a) => a,
            Either::B(_) => Vec::new(),
        };
        Ok((output, next))
    }
}

pub struct Members;

impl<I: Input> Parser<I> for Members {
    type Output = Vec<(Vec<char>, JsonValue)>;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let (output, next) = <Member as Parser<I>>::parse(input, current, context)?;
        let (rest, next) =
            <ZeroOrMore<Concat<CommaChar, Member>> as Parser<I>>::parse(input, next, context)?;
        let mut result = Vec::new();
        result.push(output);
        if let Either::A(rest) = rest {
            result.extend(rest.into_iter().map(|(_, m)| m))
        }
        Ok((result, next))
    }
}

pub struct Elements;

impl<I: Input> Parser<I> for Elements {
    type Output = Vec<JsonValue>;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let (output, next) = <Element as Parser<I>>::parse(input, current, context)?;
        let (rest, next) =
            <ZeroOrMore<Concat<CommaChar, Element>> as Parser<I>>::parse(input, next, context)?;
        let mut result = Vec::new();
        result.push(output);
        if let Either::A(rest) = rest {
            result.extend(rest.into_iter().map(|(_, m)| m))
        }
        Ok((result, next))
    }
}

pub struct Array;

impl<I: Input> Parser<I> for Array {
    type Output = Vec<JsonValue>;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let context = &context.nest(input, current)?;
        let (_, next) = <OpenSquareBracketChar as Parser<I>>::parse(input, current, context)?;
        let (output, next) =
            <OneOf<Elements, Whitespace> as Parser<I>>::parse(input, next, context)?;
        let (_, next) = <CloseSquareBracketChar as Parser<I>>::parse(input, next, context)?;
        let output = match output {
            Either::A(a) => a,
            Either::B(_) => Vec::new(),
        };
        Ok((output, next))
    }
}

pub struct Json;

impl<I: Input> Parser<I> for Json {
    type Output = <Element as Parser<I>>::Output;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let (_, next) = <Whitespace as Parser<I>>::parse(input, current, context)?;
        let (res, next) = <Element as Parser<I>>::parse(input, next, context)?;
        let (_, next) = <Whitespace as Parser<I>>::parse(input, next, context)?;
        if input.is_end(next) {
            Ok((res, next))
        } else {
            Err(input.error_at(next, "Expect end of input"))
        }
    }
}

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
    use crate::NumberValue;
    use lite_parser::impls::SimplePosition;

    #[test]
    fn it_works() {
        assert_eq!(
            parse_json(
                &r#"{ "test": 1, "test2": [1e-4, 2.041e2, true, false, null, "\"1\n\""], "test3": [], "test4": {} }"#
            ),
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
                ),
                (vec!['t', 'e', 's', 't', '3'], JsonValue::Array(vec![])),
                (vec!['t', 'e', 's', 't', '4'], JsonValue::Object(vec![]))
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
