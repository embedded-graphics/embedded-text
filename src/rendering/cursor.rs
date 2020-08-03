//! Cursor to track rendering position.
use core::marker::PhantomData;
use embedded_graphics::{fonts::Font, geometry::Point, primitives::Rectangle};

/// Internal structure that keeps track of position information while rendering a [`TextBox`].
///
/// [`TextBox`]: ../../struct.TextBox.html
#[derive(Copy, Clone, Debug)]
pub struct Cursor<F: Font> {
    _marker: PhantomData<F>,

    /// Current cursor position
    pub position: Point,

    left: i32,
    right: i32,
    bottom: i32,
}

impl<F: Font> Cursor<F> {
    /// Creates a new `Cursor` object located at the top left of the given bounding [`Rectangle`].
    #[inline]
    #[must_use]
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            _marker: PhantomData,
            position: bounds.top_left,
            bottom: bounds.bottom_right.y + 1,
            left: bounds.top_left.x,
            right: bounds.bottom_right.x + 1,
        }
    }

    /// Returns the width of the textbox
    #[inline]
    #[must_use]
    pub fn line_width(&self) -> u32 {
        (self.right - self.left) as u32
    }

    /// Starts a new line.
    #[inline]
    pub fn new_line(&mut self) {
        self.position.y += F::CHARACTER_SIZE.height as i32;
    }

    /// Moves the cursor back to the start of the line.
    #[inline]
    pub fn carriage_return(&mut self) {
        self.position.x = self.left;
    }

    /// Returns whether the cursor is in the bounding box.
    ///
    /// *Note:* Only vertical overrun is checked.
    #[inline]
    #[must_use]
    pub fn in_display_area(&self) -> bool {
        (self.position.y + F::CHARACTER_SIZE.height as i32) <= self.bottom
    }

    /// Returns whether the current line has enough space to also include an object of given width.
    #[inline]
    #[must_use]
    pub fn fits_in_line(&self, width: u32) -> bool {
        width as i32 <= self.right - self.position.x
    }

    /// Advances the cursor by a given amount.
    #[inline]
    pub fn advance_unchecked(&mut self, by: u32) {
        self.position.x += by as i32;
    }

    /// Advances the cursor by a given amount.
    #[inline]
    pub fn advance(&mut self, by: u32) -> bool {
        if self.fits_in_line(by) {
            self.advance_unchecked(by);
            true
        } else {
            false
        }
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
        cursor.advance(6);
        assert!(!cursor.fits_in_line(1));
    }
}
