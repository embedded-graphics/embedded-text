//! Vertical overdraw options.
use crate::rendering::cursor::Cursor;
use core::ops::Range;

/// Implementors of this trait specify how drawing vertically outside the bounding box is handled.
pub trait VerticalOverdraw: Copy {
    /// Calculate the range of rows of the current line that can be drawn.
    fn calculate_displayed_row_range(cursor: &Cursor) -> Range<i32>;
}

/// Only render full rows of text.
#[derive(Copy, Clone, Debug)]
pub struct FullRowsOnly;
impl VerticalOverdraw for FullRowsOnly {
    #[inline]
    fn calculate_displayed_row_range(cursor: &Cursor) -> Range<i32> {
        if cursor.in_display_area() {
            0..cursor.line_height()
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
    fn calculate_displayed_row_range(cursor: &Cursor) -> Range<i32> {
        let offset_top = (cursor.top_left().y - cursor.y).max(0);
        let offset_bottom = (cursor.bottom_right().y - cursor.y + 1).min(cursor.line_height());

        offset_top..offset_bottom
    }
}

/// Display text even if it's outside the bounding box.
#[derive(Copy, Clone, Debug)]
pub struct Visible;
impl VerticalOverdraw for Visible {
    #[inline]
    fn calculate_displayed_row_range(cursor: &Cursor) -> Range<i32> {
        0..cursor.line_height()
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        geometry::{Point, Size},
        mock_display::MockDisplay,
        mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        primitives::Rectangle,
        Drawable,
    };

    use crate::{
        alignment::*,
        style::{height_mode::Exact, vertical_overdraw::*, TextBoxStyleBuilder},
        TextBox,
    };

    #[test]
    fn default_is_full_rows_only() {
        // This test verifies that FullRowsOnly does not draw partial rows
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(LeftAligned)
            .build();

        TextBox::with_textbox_style(
            "word and other words",
            Rectangle::new(Point::zero(), Size::new(55, 15)),
            style,
        )
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "................................................",
            "......................#.......................#.",
            "......................#.......................#.",
            "#...#...##...#.#....###.........###..###....###.",
            "#.#.#..#..#..##.#..#..#........#..#..#..#..#..#.",
            "#.#.#..#..#..#.....#..#........#..#..#..#..#..#.",
            ".#.#....##...#......###.........###..#..#...###.",
            "................................................",
            "................................................",
        ]);
    }

    #[test]
    fn visible_displays_regardless_of_bounds() {
        // This test verifies that FullRowsOnly does not draw partial rows

        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(LeftAligned)
            .vertical_alignment(CenterAligned)
            .height_mode(Exact(Visible))
            .build();

        // Drawing at Point(0, 3) so we don't draw outside the display due to vertical centering.
        TextBox::with_textbox_style(
            "word",
            Rectangle::new(Point::new(0, 3), Size::new(55, 3)),
            style,
        )
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "........................",
            "......................#.",
            "......................#.",
            "#...#...##...#.#....###.",
            "#.#.#..#..#..##.#..#..#.",
            "#.#.#..#..#..#.....#..#.",
            ".#.#....##...#......###.",
            "........................",
            "........................",
        ]);
    }

    #[test]
    fn hidden_only_displays_visible_rows() {
        // This test verifies that FullRowsOnly does not draw partial rows

        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(LeftAligned)
            .vertical_alignment(CenterAligned)
            .height_mode(Exact(Hidden))
            .build();

        TextBox::with_textbox_style(
            "word",
            Rectangle::new(Point::zero(), Size::new(55, 4)),
            style,
        )
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "......................#.",
            "#...#...##...#.#....###.",
            "#.#.#..#..#..##.#..#..#.",
            "#.#.#..#..#..#.....#..#.",
        ]);
    }
}
