//! Space rendering config

use core::marker::PhantomData;
use embedded_graphics::fonts::MonoFont;

/// Retrieves size of space characters.
pub trait SpaceConfig: Copy + Default {
    /// The font for which this space config belongs.
    type Font: MonoFont;

    /// Look at the size of next n spaces, without advancing.
    fn peek_next_width(&self, n: u32) -> u32;

    /// Advance the internal state
    fn consume(&mut self, n: u32) -> u32;
}

/// Contains the fixed width of a space character.
#[derive(Copy, Clone, Debug)]
pub struct UniformSpaceConfig<F> {
    _font: PhantomData<F>,

    /// Space width.
    pub space_width: u32,
}

impl<F> Default for UniformSpaceConfig<F>
where
    F: MonoFont,
{
    /// Creates a default space configuration object based on the current MonoFont.
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self {
            _font: PhantomData,
            space_width: F::CHARACTER_SIZE.width + F::CHARACTER_SPACING,
        }
    }
}

impl<F> SpaceConfig for UniformSpaceConfig<F>
where
    F: MonoFont,
{
    type Font = F;

    #[inline]
    fn peek_next_width(&self, n: u32) -> u32 {
        n * self.space_width
    }

    #[inline]
    fn consume(&mut self, n: u32) -> u32 {
        self.peek_next_width(n)
    }
}
