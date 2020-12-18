//! Whitespace rendering modified for underlined/strikethrough text.

use crate::{utils::font_ext::FontExt, Rectangle};
use core::{marker::PhantomData, ops::Range};
use embedded_graphics::{prelude::*, primitives::rectangle};
use embedded_graphics_core::pixelcolor::BinaryColor;

/// Pixel iterator to render boxes using a single color, and horizontal lines with a different one.
///
/// This struct may be used to implement custom rendering algorithms. Internally, this pixel
/// iterator is used by [`StyledLinePixelIterator`] to render whitespace.
///
/// [`StyledLinePixelIterator`]: ../line/struct.StyledLinePixelIterator.html
#[derive(Clone, Debug)]
pub struct ModifiedEmptySpaceIterator<F>
where
    F: MonoFont,
{
    points: rectangle::Points,
    underline: bool,
    strikethrough: bool,
    _font: PhantomData<F>,
}

impl<F> ModifiedEmptySpaceIterator<F>
where
    F: MonoFont,
{
    /// Creates a new pixel iterator to draw empty spaces.
    #[inline]
    #[must_use]
    pub fn new(width: u32, rows: Range<i32>, underline: bool, strikethrough: bool) -> Self {
        let start = rows.start;
        let rows = rows.count() as u32;
        // adjust height if whole character is displayed for underline
        let rows = if underline { rows + 1 } else { rows };

        Self {
            points: Rectangle::new(Point::new(0, start), Size::new(width, rows)).points(),
            underline,
            strikethrough,
            _font: PhantomData,
        }
    }
}

impl<F> Iterator for ModifiedEmptySpaceIterator<F>
where
    F: MonoFont,
{
    type Item = Pixel<BinaryColor>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.points.next().map(|point| {
            let is_underline = self.underline && point.y as u32 == F::CHARACTER_SIZE.height;
            let is_strikethrough = self.strikethrough && point.y as u32 == F::strikethrough_pos();

            let color = BinaryColor::from(is_underline || is_strikethrough);
            Pixel(point, color)
        })
    }
}
