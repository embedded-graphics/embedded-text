//! Rectangle helper extensions.
//!
//! Extends the embedded-graphics [`Rectangle`] struct with some helper methods.
use embedded_graphics::{prelude::*, primitives::Rectangle};

/// [`Rectangle`] extensions
pub trait RectExt {
    /// Returns the (correct) size of a [`Rectangle`].
    fn size(self) -> Size;
}

impl RectExt for Rectangle {
    #[inline]
    #[must_use]
    fn size(self) -> Size {
        // TODO: remove if fixed in embedded-graphics
        let width = (self.bottom_right.x - self.top_left.x) as u32 + 1;
        let height = (self.bottom_right.y - self.top_left.y) as u32 + 1;

        Size::new(width, height)
    }
}
