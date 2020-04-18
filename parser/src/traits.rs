pub trait Position: core::ops::Sub<Self, Output = i32> + Copy {
    fn index(&self) -> u32;
    fn line(&self) -> u32;
    fn column(&self) -> u32;
}

pub trait Error {
    type Position;
    fn reasons(&self) -> &[(Self::Position, &'static str)];
    fn add_reason(self, position: Self::Position, reason: &'static str) -> Self;
}

pub trait Input: Default {
    type Position: Position;
    type Error: Error<Position = Self::Position>;
    fn next(&self, pos: Self::Position) -> Result<(char, Self::Position), Self::Error>;
    fn next_range(
        &self,
        start: Self::Position,
        counts: u32,
    ) -> Result<(&str, Self::Position), Self::Error>;
    fn error_at(&self, pos: Self::Position, reason: &'static str) -> Self::Error;
    fn is_end(&self, pos: Self::Position) -> bool;
}

pub type ResultOf<I, O> = Result<(O, <I as Input>::Position), <I as Input>::Error>;
