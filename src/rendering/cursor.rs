//! Cursor to track rendering position.
use embedded_graphics::{geometry::Point, primitives::Rectangle, text::LineHeight};

use az::SaturatingAs;

/// Tracks position within a line.
#[derive(Debug, Clone)]
pub struct LineCursor {
    start: Point,
    width: u32,
    position: u32,
    tab_width: u32,
}

impl LineCursor {
    /// Creates a new object whose position isn't important.
    pub const fn new(width: u32, tab_width: u32) -> Self {
        Self {
            start: Point::zero(),
            width,
            tab_width,
            position: 0,
        }
    }

    pub fn pos(&self) -> Point {
        self.start + Point::new(self.position.saturating_as(), 0)
    }

    /// Returns the distance to the next tab position.
    pub const fn next_tab_width(&self) -> u32 {
        let next_tab_pos = if self.tab_width == 0 {
            self.position
        } else {
            (self.position / self.tab_width + 1) * self.tab_width
        };
        next_tab_pos - self.position
    }

    /// Returns the width of the text box.
    pub const fn line_width(&self) -> u32 {
        self.width
    }

    /// Returns whether the current line has enough space to also include an object of given width.
    pub const fn fits_in_line(&self, width: u32) -> bool {
        width <= self.space()
    }

    /// Returns the amount of empty space in the line.
    pub const fn space(&self) -> u32 {
        self.width - self.position
    }

    /// Moves the cursor by a given amount.
    pub fn move_cursor(&mut self, by: i32) -> Result<i32, i32> {
        if by < 0 {
            let abs = by.unsigned_abs();
            if abs <= self.position {
                self.position -= abs;
                Ok(by)
            } else {
                Err(-(self.position as i32))
            }
        } else {
            let space = self.space() as i32;
            if by <= space {
                // Here we know by > 0, cast is safe
                self.position += by as u32;
                Ok(by)
            } else {
                Err(space)
            }
        }
    }

    /// Moves the cursor forward by a given amount.
    pub fn move_cursor_forward(&mut self, by: u32) -> Result<u32, u32> {
        let space = self.space();
        if by <= space {
            // Here we know by > 0, cast is safe
            self.position += by;
            Ok(by)
        } else {
            Err(space)
        }
    }
}

/// Internal structure that keeps track of position information while rendering a [`TextBox`].
///
/// [`TextBox`]: crate::TextBox
#[derive(Copy, Clone, Debug)]
pub struct Cursor {
    /// Current cursor position
    pub y: i32,

    top_left: Point,
    bottom: i32,

    line_width: u32,
    line_height: u32,
    line_spacing: i32,
    tab_width: u32,
}

impl Cursor {
    /// Creates a new `Cursor` object located at the top left of the given bounding [`Rectangle`].
    #[inline]
    #[must_use]
    pub fn new(
        bounds: Rectangle,
        base_line_height: u32,
        line_height: LineHeight,
        tab_width: u32,
    ) -> Self {
        Self {
            y: bounds.top_left.y,

            top_left: bounds.top_left,
            bottom: bounds.top_left.y + bounds.size.height.saturating_as::<i32>()
                - base_line_height.saturating_as::<i32>(),

            line_width: bounds.size.width,
            line_height: base_line_height,
            line_spacing: line_height.to_absolute(base_line_height).saturating_as(),
            tab_width,
        }
    }

    #[must_use]
    pub(crate) fn line(&self) -> LineCursor {
        LineCursor {
            start: self.line_start(),
            width: self.line_width,
            position: 0,
            tab_width: self.tab_width,
        }
    }

    /// Returns the coordinates of the start of the current line.
    #[inline]
    pub(crate) fn line_start(&self) -> Point {
        Point::new(self.top_left.x, self.y)
    }

    /// Returns the vertical offset of the bottom of the last visible line.
    #[inline]
    pub fn bottom(&self) -> i32 {
        self.bottom
    }

    /// Returns the coordinates of the top left corner.
    #[inline]
    pub fn top_left(&self) -> Point {
        self.top_left
    }

    /// Returns the width of the text box.
    #[inline]
    pub fn line_width(&self) -> u32 {
        self.line_width
    }

    /// Returns the height of a line.
    #[inline]
    pub fn line_height(&self) -> u32 {
        self.line_height
    }

    /// Starts a new line.
    #[inline]
    pub fn new_line(&mut self) {
        self.y += self.line_spacing;
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
        self.top_left.y <= self.y && self.y <= self.bottom
    }
}
