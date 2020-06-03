#[cfg(not(feature = "std"))]
pub extern crate alloc;

use crate::impls::{SimpleError, SimplePosition};
use crate::traits::{Error, Input, Position, ResultOf};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::marker::PhantomData;

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct ParserOptions {
    pub max_nest_level: Option<u32>,
}

impl Default for ParserOptions {
    fn default() -> Self {
        ParserOptions {
            max_nest_level: Some(100),
        }
    }
}

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct ParserContext {
    nest_level: u32,
    options: ParserOptions,
}

impl ParserContext {
    pub fn new(options: ParserOptions) -> Self {
        Self {
            nest_level: 0,
            options,
        }
    }

    pub fn options(&self) -> &ParserOptions {
        &self.options
    }

    pub fn nest<I: Input>(&self, input: &I, pos: I::Position) -> Result<Self, I::Error> {
        if Some(self.nest_level) == self.options.max_nest_level {
            Err(input.error_at(pos, "Exceeded nest level"))
        } else {
            Ok(Self {
                nest_level: self.nest_level + 1,
                options: self.options.clone(),
            })
        }
    }
}

pub trait Parser<I: Input> {
    type Output;
    fn parse(input: &I, current: I::Position, context: &ParserContext)
        -> ResultOf<I, Self::Output>;
}

pub trait Predicate<T> {
    fn eval(t: &T) -> bool;
}

pub struct ExpectChar<P>(PhantomData<P>);

impl<P: Predicate<char>, I: Input> Parser<I> for ExpectChar<P> {
    type Output = char;
    fn parse(
        input: &I,
        current: I::Position,
        _context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let (c, next) = input
            .next(current)
            .map_err(|e| e.add_reason(current, "ExpectChar"))?;
        if P::eval(&c) {
            Ok((c, next))
        } else {
            Err(input.error_at(current, "ExpectChar"))
        }
    }
}

pub struct Null;

impl<I: Input> Parser<I> for Null {
    type Output = ();
    fn parse(
        _input: &I,
        current: I::Position,
        _context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        Ok(((), current))
    }
}

pub struct Concat<P, P2>(PhantomData<(P, P2)>);

impl<I: Input, P: Parser<I>, P2: Parser<I>> Parser<I> for Concat<P, P2> {
    type Output = (P::Output, P2::Output);
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let (output1, pos) =
            P::parse(input, current, context).map_err(|e| e.add_reason(current, "Concat1"))?;
        let (output2, pos) =
            P2::parse(input, pos, context).map_err(|e| e.add_reason(current, "Concat2"))?;
        Ok(((output1, output2), pos))
    }
}

pub type Concat3<P, P2, P3> = Concat<P, Concat<P2, P3>>;
pub type Concat4<P, P2, P3, P4> = Concat<P, Concat<P2, Concat<P3, P4>>>;
pub type Concat5<P, P2, P3, P4, P5> = Concat<P, Concat<P2, Concat<P3, Concat<P4, P5>>>>;

#[cfg_attr(feature = "std", derive(Debug))]
pub enum Either<A, B> {
    A(A),
    B(B),
}

pub struct OneOf<P, P2>(PhantomData<(P, P2)>);

impl<I: Input, P: Parser<I>, P2: Parser<I>> Parser<I> for OneOf<P, P2> {
    type Output = Either<P::Output, P2::Output>;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        P::parse(input, current, context)
            .map(|(output, pos)| (Either::A(output), pos))
            .or_else(|_| {
                P2::parse(input, current, context).map(|(output, pos)| (Either::B(output), pos))
            })
            .map_err(|e| e.add_reason(current, "OneOf"))
    }
}

pub type OneOf3<P, P2, P3> = OneOf<P, OneOf<P2, P3>>;
pub type OneOf4<P, P2, P3, P4> = OneOf<P, OneOf3<P2, P3, P4>>;
pub type OneOf5<P, P2, P3, P4, P5> = OneOf<P, OneOf4<P2, P3, P4, P5>>;
pub type OneOf6<P, P2, P3, P4, P5, P6> = OneOf<P, OneOf5<P2, P3, P4, P5, P6>>;
pub type OneOf7<P, P2, P3, P4, P5, P6, P7> = OneOf<P, OneOf6<P2, P3, P4, P5, P6, P7>>;
pub type OneOf8<P, P2, P3, P4, P5, P6, P7, P8> = OneOf<P, OneOf7<P2, P3, P4, P5, P6, P7, P8>>;
pub type OneOf9<P, P2, P3, P4, P5, P6, P7, P8, P9> =
    OneOf<P, OneOf8<P2, P3, P4, P5, P6, P7, P8, P9>>;

pub type ZeroOrOne<P> = OneOf<P, Null>;

pub type ZeroOrMore<P> = OneOf<OneOrMore<P>, Null>;

//pub type OneOrMore<P> = Concat<P, ZeroOrMore<P>>;
pub struct OneOrMore<P>(PhantomData<P>);

impl<I: Input, P: Parser<I>> Parser<I> for OneOrMore<P> {
    type Output = Vec<P::Output>;
    fn parse(
        input: &I,
        current: I::Position,
        context: &ParserContext,
    ) -> ResultOf<I, Self::Output> {
        let mut output_list = Vec::new();
        let (output, mut pos) =
            P::parse(input, current, context).map_err(|e| e.add_reason(current, "OneOrMore"))?;
        output_list.push(output);
        loop {
            if let Ok((output, next_pos)) = P::parse(input, pos, context) {
                pos = next_pos;
                output_list.push(output);
            } else {
                return Ok((output_list, pos));
            }
        }
    }
}

impl Input for &str {
    type Position = SimplePosition;
    type Error = SimpleError;

    fn next(&self, pos: Self::Position) -> Result<(char, Self::Position), Self::Error> {
        self.chars()
            .nth(pos.index() as usize)
            .ok_or_else(|| self.error_at(pos, "Out of bounds"))
            .map(|c| (c, pos.next(c)))
    }

    fn next_range(
        &self,
        start: Self::Position,
        counts: u32,
    ) -> Result<(&str, Self::Position), Self::Error> {
        let start_index = start.index() as usize;
        let range = start_index..start_index + counts as usize;
        self.get(range)
            .map(|s| {
                let mut pos = start;
                for c in s.chars() {
                    pos = pos.next(c);
                }
                (s, pos)
            })
            .ok_or_else(|| self.error_at(start, "Out of bounds"))
    }

    fn error_at(&self, pos: Self::Position, reason: &'static str) -> Self::Error {
        let mut reasons = Vec::new();
        reasons.push((pos, reason));
        SimpleError { reasons }
    }

    fn is_end(&self, pos: Self::Position) -> bool {
        pos.index() as usize >= self.len()
    }
}

#[macro_export]
macro_rules! literals {
    (
        $(
            $( #[ $attr:meta ] )*
            $vis:vis $name:ident => $($($value:literal)..=+)|+;
        )*
    ) => {
        $(
            $crate::literals!{
                IMPL
                $( #[ $attr ] )*
                $vis $name => $($($value)..=+)|+
            }
        )*
    };
    (
        IMPL
        $( #[ $attr:meta ] )*
        $vis:vis $name:ident => $($($value:literal)..=+)|+
    ) => (
        $crate::paste::item! {
            $vis struct [< $name Predicate >];
            impl $crate::parser::Predicate<char> for [< $name Predicate >] {
                fn eval(c: &char) -> bool {
                    match *c {
                        $($($value)..=+)|+ => true,
                        _ => false
                    }
                }
            }

            $( #[ $attr ] )*
            $vis type $name = $crate::parser::ExpectChar<[< $name Predicate >]>;
        }
    );
}

#[macro_export]
macro_rules! parsers {
    (
        $(
            $( #[ $attr:meta ] )*
            $vis:vis $name:ident = $type:ty, $output_type:ty, ($output:ident) => $body:block;
        )*
    ) => {
        $(
            $vis struct $name;
            impl<I: $crate::traits::Input> $crate::parser::Parser<I> for $name {
                type Output = $output_type;
                fn parse(input: &I, current: I::Position, context: &ParserContext) -> $crate::traits::ResultOf<I, Self::Output> {
                    let ($output, pos) = <$type as $crate::parser::Parser<I>>::parse(input, current, context)
                        .map_err(|e| <I::Error as $crate::traits::Error>::add_reason(e, current, stringify!($name)))?;
                    let res = $body;
                    Ok((res, pos))
                }
            }
        )*
    };
}
