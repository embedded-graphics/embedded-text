//! Whitespace rendering

use core::{marker::PhantomData, mem::MaybeUninit};
use embedded_graphics::{prelude::*, style::TextStyle};

/// Pixel iterator to render font spacing
#[derive(Clone, Debug)]
pub struct EmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    _font: PhantomData<F>,
    color: MaybeUninit<C>,
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
    pub fn new(width: u32, position: Point, style: TextStyle<C, F>) -> Self {
        if width == 0 || style.background_color.is_none() {
            Self {
                _font: PhantomData,
                color: MaybeUninit::uninit(),
                pos: Point::zero(),
                char_walk: Point::zero(),
                walk_max_x: 0,
            }
        } else {
            let walk_max_x = position.x + width as i32 - 1;
            let walk_max_y = position.y + F::CHARACTER_SIZE.height as i32;

            Self {
                _font: PhantomData,
                color: MaybeUninit::new(style.background_color.unwrap()),
                pos: Point::new(position.x, walk_max_y),
                char_walk: position,
                walk_max_x,
            }
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
        let walk = self.char_walk;
        if walk.y < self.pos.y {
            if walk.x < self.walk_max_x {
                self.char_walk.x += 1;
            } else {
                self.char_walk.x = self.pos.x;
                self.char_walk.y += 1;
            }

            // Skip to next point if pixel is transparent
            Some(Pixel(walk, unsafe {
                // this is safe because if not initialized,
                // coordinates are set to never hit this line
                self.color.assume_init()
            }))
        } else {
            // Done with filling this space
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::EmptySpaceIterator;
    use embedded_graphics::{
        fonts::{Font6x6, Font6x8},
        pixelcolor::BinaryColor,
        prelude::*,
        style::TextStyleBuilder,
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

    #[test]
    fn first_point_in_position() {
        let style = TextStyleBuilder::new(Font6x8)
            .background_color(BinaryColor::On)
            .build();

        let pos = Point::new(8, 6);
        assert_eq!(
            pos,
            EmptySpaceIterator::new(10, pos, style).next().unwrap().0
        );
    }

    #[test]
    fn minimal_number_of_pixels_returned() {
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        assert_eq!(
            80,
            EmptySpaceIterator::new(10, Point::zero(), style).count()
        );

        let style = TextStyleBuilder::new(Font6x6)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        assert_eq!(
            60,
            EmptySpaceIterator::new(10, Point::zero(), style).count()
        );
    }
}
