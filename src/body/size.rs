#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodySize {
    None,
    Sized(u64),
}

impl BodySize {
    pub const ZERO: Self = Self::Sized(0);

    pub fn is_eof(&self) -> bool {
        matches!(self, BodySize::None | BodySize::Sized(0))
    }
}
