//! Cursor to track rendering position
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
        self.position.y < self.bounds.bottom_right.y
    }

    /// Returns whether the current line has enough space to also include an object of given width.
    #[inline]
    pub fn fits_in_line(&self, width: u32) -> bool {
        width as i32 <= self.bounds.bottom_right.x - self.position.x + 1
    }

    /// Advances the cursor by a given character.
    #[inline]
    pub fn advance_char(&mut self, c: char) -> bool {
        self.advance(F::char_width(c))
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
}
