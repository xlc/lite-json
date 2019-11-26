#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::parser::{
    Concat, Concat3, Either, Error, Input, OneOf, OneOrMore, Parser, ResultOf, ZeroOrMore,
    ZeroOrOne,
};
use crate::{literals, parsers};
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
    fn parse(input: &I, current: I::Position) -> ResultOf<I, Self::Output> {
        let (c, next) = input
            .next(current)
            .map_err(|e| e.add_reason(current, "Escape"))?;
        match c {
            '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => Ok((c, next)),
            'u' => {
                let (b1, next) = <Hex as Parser<I>>::parse(input, next)?;
                let (b2, next) = <Hex as Parser<I>>::parse(input, next)?;
                let (b3, next) = <Hex as Parser<I>>::parse(input, next)?;
                let (b4, next) = <Hex as Parser<I>>::parse(input, next)?;
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
    fn parse(input: &I, current: I::Position) -> ResultOf<I, Self::Output> {
        let (c, next) = input
            .next(current)
            .map_err(|e| e.add_reason(current, "Character"))?;
        match c {
            '\\' => <Escape as Parser<I>>::parse(input, next),
            '"' => Err(input.error_at(current, "Character")),
            _ => Ok((c, next)),
        }
    }
}

pub type Characters = ZeroOrMore<Character>;

pub struct Member;

impl<I: Input> Parser<I> for Member {
    type Output = (Vec<char>, JsonValue);
    fn parse(input: &I, current: I::Position) -> ResultOf<I, Self::Output> {
        let (_, next) = <Whitespace as Parser<I>>::parse(input, current)?;
        let (key, next) = <String as Parser<I>>::parse(input, next)?;
        let (_, next) = <Whitespace as Parser<I>>::parse(input, next)?;
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
        let (value, next) = <Element as Parser<I>>::parse(input, next)?;
        Ok(((key, value), next))
    }
}

pub struct Element;

impl<I: Input> Parser<I> for Element {
    type Output = JsonValue;
    fn parse(input: &I, current: I::Position) -> ResultOf<I, Self::Output> {
        let (_, next) = <Whitespace as Parser<I>>::parse(input, current)?;
        let (output, next) = <Value as Parser<I>>::parse(input, next)?;
        let (_, next) = <Whitespace as Parser<I>>::parse(input, next)?;
        Ok((output, next))
    }
}

pub struct Value;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, PartialEq)]
pub struct NumberValue {
    pub integer: i64,
    pub fraction: u64,
    pub fraction_length: u32,
    pub exponent: i32,
}

#[cfg(feature = "std")]
impl Into<f64> for NumberValue {
    fn into(self) -> f64 {
        (self.integer as f64 + self.fraction as f64 / 10f64.powi(self.fraction_length as i32))
            * 10f64.powi(self.exponent)
    }
}

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

impl<I: Input> Parser<I> for Value
where
    I::Position: Copy,
{
    type Output = JsonValue;
    fn parse(input: &I, current: I::Position) -> ResultOf<I, Self::Output> {
        if let Ok((output, next)) = <Object as Parser<I>>::parse(input, current) {
            return Ok((JsonValue::Object(output), next));
        }
        if let Ok((output, next)) = <Array as Parser<I>>::parse(input, current) {
            return Ok((JsonValue::Array(output), next));
        }
        if let Ok((output, next)) = <String as Parser<I>>::parse(input, current) {
            return Ok((JsonValue::String(output), next));
        }
        if let Ok((output, next)) = <Number as Parser<I>>::parse(input, current) {
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

type JsonObject = Vec<(Vec<char>, JsonValue)>;

impl<I: Input> Parser<I> for Object {
    type Output = JsonObject;
    fn parse(input: &I, current: I::Position) -> ResultOf<I, Self::Output> {
        let (_, next) = <OpenCurlyBracketChar as Parser<I>>::parse(input, current)?;
        let (output, next) = <OneOf<Members, Whitespace> as Parser<I>>::parse(input, next)?;
        let (_, next) = <CloseCurlyBracketChar as Parser<I>>::parse(input, next)?;
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
    fn parse(input: &I, current: I::Position) -> ResultOf<I, Self::Output> {
        let (output, next) = <Member as Parser<I>>::parse(input, current)?;
        let (rest, next) =
            <ZeroOrMore<Concat<CommaChar, Member>> as Parser<I>>::parse(input, next)?;
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
    fn parse(input: &I, current: I::Position) -> ResultOf<I, Self::Output> {
        let (output, next) = <Element as Parser<I>>::parse(input, current)?;
        let (rest, next) =
            <ZeroOrMore<Concat<CommaChar, Element>> as Parser<I>>::parse(input, next)?;
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
    fn parse(input: &I, current: I::Position) -> ResultOf<I, Self::Output> {
        let (_, next) = <OpenSquareBracketChar as Parser<I>>::parse(input, current)?;
        let (res, next) = <Elements as Parser<I>>::parse(input, next)?;
        let (_, next) = <CloseSquareBracketChar as Parser<I>>::parse(input, next)?;
        Ok((res, next))
    }
}

pub type Json = Element;
