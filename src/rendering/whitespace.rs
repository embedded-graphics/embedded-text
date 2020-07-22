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
    pub fn new(width: u32, pos: Point, style: TextStyle<C, F>) -> Self {
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

#[cfg(test)]
mod test {
    use super::EmptySpaceIterator;
    use embedded_graphics::{
        fonts::Font6x8, pixelcolor::BinaryColor, prelude::*, style::TextStyleBuilder,
    };

    #[test]
    fn zero_width_does_not_render_anything() {
        let style = TextStyleBuilder::new(Font6x8)
            .background_color(BinaryColor::On)
            .build();

        assert_eq!(0, EmptySpaceIterator::new(0, Point::zero(), style).count());
    }

    #[test]
    fn transparent_background_does_not_render_anything() {
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .build();

        assert_eq!(0, EmptySpaceIterator::new(10, Point::zero(), style).count());
    }
}
