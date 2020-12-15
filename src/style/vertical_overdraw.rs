//! Vertical overdraw options.
use crate::rendering::cursor::Cursor;
use core::ops::Range;
use embedded_graphics::fonts::MonoFont;

/// Implementors of this trait specify how drawing vertically outside the bounding box is handled.
pub trait VerticalOverdraw: Copy {
    /// Calculate the range of rows of the current line that can be drawn.
    fn calculate_displayed_row_range<F: MonoFont>(cursor: &Cursor<F>) -> Range<i32>;
}

/// Only render full rows of text.
#[derive(Copy, Clone, Debug)]
pub struct FullRowsOnly;
impl VerticalOverdraw for FullRowsOnly {
    #[inline]
    fn calculate_displayed_row_range<F: MonoFont>(cursor: &Cursor<F>) -> Range<i32> {
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
    fn calculate_displayed_row_range<F: MonoFont>(cursor: &Cursor<F>) -> Range<i32> {
        let offset_top = (cursor.bounds.top_left.y - cursor.position.y).max(0);
        // cursor bounds are one row shorter than real bounds for optimization
        // purposes so use the real height here
        let offset_bottom = (cursor.bottom_right().y + F::CHARACTER_SIZE.height as i32
            - cursor.position.y)
            .min(F::CHARACTER_SIZE.height as i32);

        offset_top..offset_bottom
    }
}

/// Display text even if it's outside the bounding box.
#[derive(Copy, Clone, Debug)]
pub struct Visible;
impl VerticalOverdraw for Visible {
    #[inline]
    fn calculate_displayed_row_range<F: MonoFont>(_: &Cursor<F>) -> Range<i32> {
        0..F::CHARACTER_SIZE.height as i32
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    };
    use embedded_graphics_core::primitives::Rectangle;

    use crate::{
        alignment::*,
        style::{height_mode::Exact, vertical_overdraw::*, TextBoxStyleBuilder},
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
            Rectangle::new(Point::new(0, 0), Size::new(55, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "                      #                       #",
            "                      #                       #",
            "#   #  ###  # ##   ## #        ###  # ##   ## #",
            "#   # #   # ##  # #  ##           # ##  # #  ##",
            "# # # #   # #     #   #        #### #   # #   #",
            "# # # #   # #     #   #       #   # #   # #   #",
            " # #   ###  #      ####        #### #   #  ####",
            "                                               ",
        ]);
    }

    #[test]
    fn visible_displays_regardless_of_bounds() {
        // This test verifies that FullRowsOnly does not draw partial rows

        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .vertical_alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .height_mode(Exact(Visible))
            .build();

        TextBox::new("word", Rectangle::new(Point::new(0, 2), Size::new(55, 3)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "                      #",
            "                      #",
            "#   #  ###  # ##   ## #",
            "#   # #   # ##  # #  ##",
            "# # # #   # #     #   #",
            "# # # #   # #     #   #",
            " # #   ###  #      ####",
            "                       ",
        ]);
    }

    #[test]
    fn hidden_only_displays_visible_rows() {
        // This test verifies that FullRowsOnly does not draw partial rows

        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .vertical_alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .height_mode(Exact(Hidden))
            .build();

        TextBox::new("word", Rectangle::new(Point::new(0, 2), Size::new(55, 4)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "                       ",
            "                       ",
            "#   #  ###  # ##   ## #",
            "#   # #   # ##  # #  ##",
            "# # # #   # #     #   #",
            "# # # #   # #     #   #",
            "                       ",
            "                       ",
        ]);
    }
}
