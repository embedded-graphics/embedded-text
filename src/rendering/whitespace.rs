//! Whitespace rendering

use core::marker::PhantomData;
use embedded_graphics::{prelude::*, style::TextStyle};

/// Pixel iterator to render font spacing
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct EmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    _font: PhantomData<F>,
    color: Option<C>,
    pos: Point,
    char_walk: Point,
    walk_max_x: i32,
}

impl<C, F> EmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Creates a new pixel iterator to draw empty spaces.
    #[inline]
    #[must_use]
    pub fn new(pos: Point, width: u32, style: TextStyle<C, F>) -> Self {
        Self {
            _font: PhantomData,
            color: style.background_color,
            pos,
            char_walk: Point::zero(),
            walk_max_x: width as i32 - 1,
        }
    }
}

impl<C, F> Iterator for EmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(color) = self.color {
            if self.walk_max_x < 0 || self.char_walk.y >= F::CHARACTER_SIZE.height as i32 {
                // Done with filling this space
                None
            } else {
                let p = self.pos + self.char_walk;

                if self.char_walk.x < self.walk_max_x {
                    self.char_walk.x += 1;
                } else {
                    self.char_walk.x = 0;
                    self.char_walk.y += 1;
                }

                // Skip to next point if pixel is transparent
                Some(Pixel(p, color))
            }
        } else {
            None
        }
    }
}
