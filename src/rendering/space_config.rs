//! Space rendering config

/// Retrieves size of space characters.
pub trait SpaceConfig: Copy {
    /// Look at the size of next n spaces, without advancing.
    fn peek_next_width(&self, n: u32) -> u32;

    /// Advance the internal state
    fn consume(&mut self, n: u32) -> u32;
}

/// Contains the fixed width of a space character.
#[derive(Copy, Clone, Debug)]
pub struct UniformSpaceConfig {
    /// Space width.
    pub space_width: u32,
}

impl SpaceConfig for UniformSpaceConfig {
    #[inline]
    fn peek_next_width(&self, n: u32) -> u32 {
        n * self.space_width
    }

    #[inline]
    fn consume(&mut self, n: u32) -> u32 {
        self.peek_next_width(n)
    }
}
