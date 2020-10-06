//! Whitespace rendering modified for underlined/strikethrough text.

use crate::utils::font_ext::FontExt;
use core::ops::Range;
use embedded_graphics::{prelude::*, style::TextStyle};

/// Pixel iterator to render boxes using a single color, and horizontal lines with a different one.
///
/// This struct may be used to implement custom rendering algorithms. Internally, this pixel
/// iterator is used by [`StyledLinePixelIterator`] to render whitespace.
///
/// [`StyledLinePixelIterator`]: ../line/struct.StyledLinePixelIterator.html
#[derive(Clone, Debug)]
pub struct ModifiedEmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    style: TextStyle<C, F>,
    pos: Point,
    char_walk: Point,
    max_coordinates: Point,
    underline_offset: Option<u32>,
    strikethrough_offset: Option<u32>,
}

impl<C, F> ModifiedEmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Creates a new pixel iterator to draw empty spaces.
    #[inline]
    #[must_use]
    pub fn new(
        width: u32,
        pos: Point,
        style: TextStyle<C, F>,
        rows: Range<i32>,
        underline: bool,
        strikethrough: bool,
    ) -> Self {
        let mut max_height = (F::CHARACTER_SIZE.height as i32).min(rows.end);
        let underline_offset = if underline {
            // adjust height if whole character is displayed for underline
            if rows.end == max_height {
                max_height += 1;
            }
            Some(F::CHARACTER_SIZE.height)
        } else {
            None
        };
        let strikethrough_offset = if strikethrough {
            Some(F::strikethrough_pos())
        } else {
            None
        };
        Self {
            style,
            pos,
            char_walk: Point::new(0, rows.start),
            max_coordinates: Point::new(width as i32 - 1, max_height),
            underline_offset,
            strikethrough_offset,
        }
    }
}

impl<C, F> Iterator for ModifiedEmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.char_walk.y >= self.max_coordinates.y {
                // Done with this char, move on to the next one
                break None;
            }
            let pos = self.char_walk;

            let is_extra_line = Some(self.char_walk.y as u32) == self.underline_offset
                || Some(self.char_walk.y as u32) == self.strikethrough_offset;

            if pos.x < self.max_coordinates.x {
                self.char_walk.x += 1;
            } else {
                self.char_walk.x = 0;
                self.char_walk.y += 1;
            }

            let color = if is_extra_line {
                self.style.text_color
            } else {
                self.style.background_color
            };

            // Skip to next point if pixel is transparent
            if let Some(color) = color {
                let p = self.pos + pos;
                break Some(Pixel(p, color));
            }
        }
    }
}
