#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

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

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

impl Serialize for NumberValue {
    fn serialize(&self) -> Vec<u8> {
        let mut integer_part = self.integer.to_string().into_bytes();
        let mut fraction_part = if self.fraction > 0 {
            let mut fraction_part = vec!['.' as u8];
            let mut fraction_nums = self.fraction.to_string().into_bytes();
            let fraction_length = self.fraction_length as usize;
            if fraction_nums.len() < fraction_length {
                let mut zeros = vec!['0' as u8; fraction_length - fraction_nums.len()];
                fraction_part.append(&mut zeros);
                fraction_part.append(&mut fraction_nums);
            } else {
                fraction_part.append(&mut fraction_nums);
            }
            fraction_part
        } else {
            Vec::<u8>::new()
        };
        let mut exponent_part = if self.exponent == 0 {
            Vec::<u8>::new()
        } else {
            let mut exponent_part = vec!['e' as u8];
            if self.exponent < 0 {
                exponent_part.push('-' as u8);
            }
            let mut exp_str = self.exponent.abs().to_string().into_bytes();
            exponent_part.append(&mut exp_str);
            exponent_part
        };
        integer_part.append(&mut fraction_part);
        integer_part.append(&mut exponent_part);
        integer_part
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
}