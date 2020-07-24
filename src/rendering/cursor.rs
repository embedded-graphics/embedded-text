//! Cursor to track rendering position
use crate::utils::font_ext::FontExt;
use core::marker::PhantomData;
use embedded_graphics::{prelude::*, primitives::Rectangle};

/// Internal structure that keeps track of rendering a [`TextBox`].
pub struct Cursor<F: Font> {
    _marker: PhantomData<F>,

    /// Bounding box of the [`TextBox`]
    pub bounds: Rectangle,

    /// Current cursor position
    pub position: Point,
}

impl<F: Font> Cursor<F> {
    /// Creates a new `Cursor` located at the top left of the bounding box.
    #[inline]
    #[must_use]
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            _marker: PhantomData,
            bounds,
            position: bounds.top_left,
        }
    }

    /// Starts a new line.
    #[inline]
    pub fn new_line(&mut self) {
        self.position.y += F::CHARACTER_SIZE.height as i32;
    }

    /// Moves the cursor back to the start of the line.
    #[inline]
    pub fn carriage_return(&mut self) {
        self.position.x = self.bounds.top_left.x;
    }

    /// Returns whether the cursor is in the bounding box.
    ///
    /// Note: Only vertical overrun is checked.
    #[inline]
    pub fn in_display_area(&self) -> bool {
        (self.position.y + F::CHARACTER_SIZE.height as i32 - 1) < self.bounds.bottom_right.y
    }

    /// Returns whether the current line has enough space to also include an object of given width.
    #[inline]
    pub fn fits_in_line(&self, width: u32) -> bool {
        width <= self.space_in_line()
    }

    /// Advances the cursor by a given character.
    #[inline]
    pub fn advance_char(&mut self, c: char) -> bool {
        self.advance(F::total_char_width(c))
    }

    /// Advances the cursor by a given amount.
    #[inline]
    pub fn advance(&mut self, by: u32) -> bool {
        if self.fits_in_line(by) {
            self.position.x += by as i32;
            true
        } else {
            false
        }
    }

    /// Returns the available space in the current line.
    #[inline]
    pub fn space_in_line(&self) -> u32 {
        (self.bounds.bottom_right.x + 1).saturating_sub(self.position.x) as u32
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use embedded_graphics::fonts::Font6x8;

    #[test]
    fn fits_in_line() {
        // 6px width
        let cursor: Cursor<Font6x8> = Cursor::new(Rectangle::new(Point::zero(), Point::new(5, 7)));

        assert!(cursor.fits_in_line(6));
        assert!(!cursor.fits_in_line(7));
    }

    #[test]
    fn advance_moves_position() {
        // 6px width
        let mut cursor: Cursor<Font6x8> =
            Cursor::new(Rectangle::new(Point::zero(), Point::new(5, 7)));

        assert!(cursor.fits_in_line(1));
        assert!(cursor.advance_char('a'));
        assert!(!cursor.fits_in_line(1));
    }
}
