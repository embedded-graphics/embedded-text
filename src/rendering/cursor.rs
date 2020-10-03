//! Cursor to track rendering position.
use core::marker::PhantomData;
use embedded_graphics::{fonts::Font, geometry::Point, primitives::Rectangle};

/// Internal structure that keeps track of position information while rendering a [`TextBox`].
///
/// [`TextBox`]: ../../struct.TextBox.html
#[derive(Copy, Clone, Debug)]
pub struct Cursor<F: Font> {
    /// Current cursor position
    pub position: Point,

    /// TextBox bounding rectangle
    pub bounds: Rectangle,

    line_spacing: i32,

    _marker: PhantomData<F>,
}

impl<F: Font> Cursor<F> {
    /// Creates a new `Cursor` object located at the top left of the given bounding [`Rectangle`].
    #[inline]
    #[must_use]
    pub fn new(bounds: Rectangle, line_spacing: i32) -> Self {
        Self {
            _marker: PhantomData,
            position: bounds.top_left,
            line_spacing,
            bounds: Rectangle::new(
                bounds.top_left,
                bounds.bottom_right + Point::new(1, 1 - F::CHARACTER_SIZE.height as i32),
            ),
        }
    }

    /// Returns the width of the textbox
    #[inline]
    #[must_use]
    pub fn line_width(&self) -> u32 {
        (self.bounds.bottom_right.x - self.bounds.top_left.x) as u32
    }

    /// Starts a new line.
    #[inline]
    pub fn new_line(&mut self) {
        self.position.y += F::CHARACTER_SIZE.height as i32 + self.line_spacing;
    }

    /// Moves the cursor back to the start of the line.
    #[inline]
    pub fn carriage_return(&mut self) {
        self.position.x = self.bounds.top_left.x;
    }

    /// Returns whether the cursor is completely in the bounding box.
    ///
    /// Completely means, that the line that is marked by the cursor can be drawn without any
    /// vertical clipping or drawing outside the bounds.
    ///
    /// *Note:* Only vertical overrun is checked.
    #[inline]
    #[must_use]
    pub fn in_display_area(&self) -> bool {
        self.bounds.top_left.y <= self.position.y && self.position.y <= self.bounds.bottom_right.y
    }

    /// Returns whether the current line has enough space to also include an object of given width.
    #[inline]
    #[must_use]
    pub fn fits_in_line(&self, width: u32) -> bool {
        width <= self.space()
    }

    /// Returns the amount of empty space in the line.
    #[inline]
    #[must_use]
    pub fn space(&self) -> u32 {
        (self.bounds.bottom_right.x - self.position.x) as u32
    }

    /// Advances the cursor by a given amount.
    #[inline]
    pub fn advance_unchecked(&mut self, by: u32) {
        self.position.x += by as i32;
    }

    /// Returns the x coordinate relative to the left side.
    #[inline]
    pub fn x_in_line(&self) -> i32 {
        self.position.x - self.bounds.top_left.x
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
        let cursor: Cursor<Font6x8> =
            Cursor::new(Rectangle::new(Point::zero(), Point::new(5, 7)), 0);

        assert!(cursor.fits_in_line(6));
        assert!(!cursor.fits_in_line(7));
    }

    #[test]
    fn advance_moves_position() {
        // 6px width
        let mut cursor: Cursor<Font6x8> =
            Cursor::new(Rectangle::new(Point::zero(), Point::new(5, 7)), 0);

        assert!(cursor.fits_in_line(1));
        cursor.advance(6);
        assert!(!cursor.fits_in_line(1));
    }

    #[test]
    fn in_display_area() {
        // 6px width
        let mut cursor: Cursor<Font6x8> =
            Cursor::new(Rectangle::new(Point::zero(), Point::new(5, 7)), 0);

        let data = [(0, true), (-8, false), (-1, false), (1, false)];
        for &(pos, inside) in data.iter() {
            cursor.position.y = pos;
            assert_eq!(inside, cursor.in_display_area());
        }
    }
}
