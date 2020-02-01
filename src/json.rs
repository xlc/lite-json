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