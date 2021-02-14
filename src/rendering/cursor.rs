//! Cursor to track rendering position.
use embedded_graphics::{geometry::Point, primitives::Rectangle};

/// Internal structure that keeps track of position information while rendering a [`TextBox`].
///
/// [`TextBox`]: ../../struct.TextBox.html
// FIXME: split up into two structs: an outer cursor that is responsible for vertical movement
// and an inner cursor responsible for in-line, horizontal movement
#[derive(Copy, Clone, Debug)]
pub struct Cursor {
    /// Current cursor position
    pub position: Point,

    /// TextBox bounding rectangle
    pub bounds: Rectangle,

    line_height: i32,
    line_spacing: i32,
    tab_width: u32,
}

impl Cursor {
    /// Creates a new `Cursor` object located at the top left of the given bounding [`Rectangle`].
    #[inline]
    #[must_use]
    pub fn new(bounds: Rectangle, line_height: u32, line_spacing: i32, tab_width: u32) -> Self {
        Self {
            position: bounds.top_left,
            line_spacing,
            line_height: line_height.min(i32::MAX as u32) as i32,
            bounds,
            tab_width,
        }
    }

    /// Returns the coordinates of the bottom right corner.
    #[inline]
    pub fn bottom_right(&self) -> Point {
        self.bounds.bottom_right().unwrap_or(self.bounds.top_left)
    }

    /// Returns the distance to the next tab position.
    #[inline]
    pub fn next_tab_width(&self) -> u32 {
        let pos = self.x_in_line() as u32;
        let next_tab_pos = (pos / self.tab_width + 1) * self.tab_width;
        next_tab_pos - pos
    }

    /// Returns the width of the textbox
    #[inline]
    #[must_use]
    pub fn line_width(&self) -> u32 {
        self.bounds.size.width
    }

    /// Returns the height of a line
    #[inline]
    #[must_use]
    pub fn line_height(&self) -> i32 {
        self.line_height
    }

    /// Starts a new line.
    #[inline]
    pub fn new_line(&mut self) {
        self.position.y += self.line_height + self.line_spacing;
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
        self.bounds.top_left.y <= self.position.y
            && self.position.y <= self.bottom_right().y - self.line_height + 1
    }

    /// Returns whether the current line has enough space to also include an object of given width.
    #[inline]
    #[must_use]
    pub fn fits_in_line(&self, width: u32) -> bool {
        let target = self.position.x + width as i32;
        target <= self.bottom_right().x + 1
    }

    /// Returns the amount of empty space in the line.
    #[inline]
    #[must_use]
    pub fn space(&self) -> u32 {
        (self.bottom_right().x - self.position.x + 1) as u32
    }

    /// Advances the cursor by a given amount.
    #[inline]
    pub fn advance_unchecked(&mut self, by: u32) {
        self.position.x += by as i32;
    }

    /// Returns the current horizontal offset relative to the left side.
    #[inline]
    pub fn x_in_line(&self) -> i32 {
        self.position.x - self.bounds.top_left.x
    }

    /// Advances the cursor by a given amount.
    #[inline]
    pub fn advance(&mut self, by: u32) -> Result<u32, u32> {
        if self.fits_in_line(by) {
            self.advance_unchecked(by);
            Ok(by)
        } else {
            Err(self.space())
        }
    }

    /// Rewinds the cursor by a given amount.
    #[inline]
    pub fn rewind(&mut self, by: u32) -> bool {
        let target = self.position.x - by as i32;
        if self.bounds.top_left.x <= target {
            self.position.x = target;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use embedded_graphics::{
        geometry::{Point, Size},
        primitives::Rectangle,
    };

    #[test]
    fn fits_in_line() {
        // 6px width
        let cursor: Cursor = Cursor::new(Rectangle::new(Point::zero(), Size::new(6, 8)), 8, 0, 4);

        assert!(cursor.fits_in_line(6));
        assert!(!cursor.fits_in_line(7));
    }

    #[test]
    fn advance_moves_position() {
        // 6px width
        let mut cursor: Cursor =
            Cursor::new(Rectangle::new(Point::zero(), Size::new(6, 8)), 8, 0, 4);

        assert!(cursor.fits_in_line(1));
        let _ = cursor.advance(6);
        assert!(!cursor.fits_in_line(1));
    }

    #[test]
    fn rewind_moves_position_back() {
        // 6px width
        let mut cursor: Cursor =
            Cursor::new(Rectangle::new(Point::zero(), Size::new(6, 8)), 8, 0, 4);

        let _ = cursor.advance(6);
        assert_eq!(6, cursor.position.x);
        assert!(cursor.rewind(3));
        assert_eq!(3, cursor.position.x);
        assert!(cursor.rewind(3));
        assert_eq!(0, cursor.position.x);
        assert!(!cursor.rewind(3));
        assert_eq!(0, cursor.position.x);
    }

    #[test]
    fn in_display_area() {
        // 6px width
        let mut cursor: Cursor =
            Cursor::new(Rectangle::new(Point::zero(), Size::new(6, 8)), 8, 0, 4);

        let data = [(0, true), (-8, false), (-1, false), (1, false)];
        for &(pos, inside) in data.iter() {
            cursor.position.y = pos;
            assert_eq!(
                inside,
                cursor.in_display_area(),
                "in_display_area(Point(0, {:?})) is expected to be {:?} but is {:?}",
                pos,
                inside,
                cursor.in_display_area(),
            );
        }
    }
}
