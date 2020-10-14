//! Space rendering config

use crate::utils::font_ext::FontExt;
use core::marker::PhantomData;
use embedded_graphics::fonts::Font;

/// Retrieves size of space characters.
pub trait SpaceConfig: Copy + Default {
    /// The font for which this space config belongs.
    type Font: Font;

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
    F: Font + Copy,
{
    /// Creates a default space configuration object based on the current font.
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self {
            _font: PhantomData,
            space_width: F::total_char_width(' '),
        }
    }
}

impl<F> SpaceConfig for UniformSpaceConfig<F>
where
    F: Font + Copy,
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
