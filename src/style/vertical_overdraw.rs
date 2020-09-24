//! Vertical overdraw options.
use crate::rendering::cursor::Cursor;
use core::ops::Range;
use embedded_graphics::fonts::Font;

/// Implementors of this trait specify how drawing vertically outside the bounding box is handled.
pub trait VerticalOverdraw: Copy {
    /// Calculate the range of rows of the current line that can be drawn.
    fn calculate_displayed_row_range<F: Font>(cursor: &Cursor<F>) -> Range<i32>;
}

/// Only render full rows of text.
#[derive(Copy, Clone, Debug)]
pub struct FullRowsOnly;
impl VerticalOverdraw for FullRowsOnly {
    #[inline]
    fn calculate_displayed_row_range<F: Font>(cursor: &Cursor<F>) -> Range<i32> {
        if cursor.in_display_area() {
            0..F::CHARACTER_SIZE.height as i32
        } else {
            0..0
        }
    }
}

/// Render partially visible rows, but only inside the bounding box.
#[derive(Copy, Clone, Debug)]
pub struct Hidden;
impl VerticalOverdraw for Hidden {
    #[inline]
    fn calculate_displayed_row_range<F: Font>(cursor: &Cursor<F>) -> Range<i32> {
        todo!()
    }
}

/// Display text even if it's outside the bounding box.
#[derive(Copy, Clone, Debug)]
pub struct Visible;
impl VerticalOverdraw for Visible {
    #[inline]
    fn calculate_displayed_row_range<F: Font>(_: &Cursor<F>) -> Range<i32> {
        0..F::CHARACTER_SIZE.height as i32
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        primitives::Rectangle,
    };

    use crate::{
        alignment::{BottomAligned, LeftAligned},
        style::TextBoxStyleBuilder,
        TextBox,
    };

    #[test]
    fn default_is_full_rows_only() {
        // This test verifies that FullRowsOnly does not draw partial rows

        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new(
            "word and other words",
            Rectangle::new(Point::new(0, 0), Point::new(54, 14)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                      #                       #",
                "                      #                       #",
                "#   #  ###  # ##   ## #        ###  # ##   ## #",
                "#   # #   # ##  # #  ##           # ##  # #  ##",
                "# # # #   # #     #   #        #### #   # #   #",
                "# # # #   # #     #   #       #   # #   # #   #",
                " # #   ###  #      ####        #### #   #  ####",
                "                                               ",
            ])
        );
    }
}
