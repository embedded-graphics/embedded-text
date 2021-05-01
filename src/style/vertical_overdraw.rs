//! Vertical overdraw options.
use crate::rendering::cursor::Cursor;
use core::ops::Range;

/// Vertical overdraw options used by height modes that don't conform exactly to the text size.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum VerticalOverdraw {
    /// Only render full rows of text.
    FullRowsOnly,
    /// Render partially visible rows, but only inside the bounding box.
    Hidden,
    /// Display text even if it's outside the bounding box.
    Visible,
}

impl VerticalOverdraw {
    /// Calculate the range of rows of the current line that can be drawn.
    pub fn calculate_displayed_row_range(self, cursor: &Cursor) -> Range<i32> {
        match self {
            VerticalOverdraw::FullRowsOnly => {
                if cursor.in_display_area() {
                    0..cursor.line_height()
                } else {
                    0..0
                }
            }

            VerticalOverdraw::Hidden => {
                let offset_top = (cursor.top_left().y - cursor.y).max(0);
                let offset_bottom =
                    (cursor.bottom_right().y - cursor.y + 1).min(cursor.line_height());

                offset_top..offset_bottom
            }

            VerticalOverdraw::Visible => 0..cursor.line_height(),
        }
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
        style::{height_mode::Exact, vertical_overdraw::VerticalOverdraw, TextBoxStyleBuilder},
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

        let style = TextBoxStyleBuilder::new().alignment(LeftAligned).build();

        TextBox::with_textbox_style(
            "word and other words",
            Rectangle::new(Point::zero(), Size::new(55, 15)),
            character_style,
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
            .alignment(LeftAligned)
            .vertical_alignment(CenterAligned)
            .height_mode(Exact(VerticalOverdraw::Visible))
            .build();

        // Drawing at Point(0, 3) so we don't draw outside the display due to vertical centering.
        TextBox::with_textbox_style(
            "word",
            Rectangle::new(Point::new(0, 3), Size::new(55, 3)),
            character_style,
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
            .alignment(LeftAligned)
            .vertical_alignment(CenterAligned)
            .height_mode(Exact(VerticalOverdraw::Hidden))
            .build();

        TextBox::with_textbox_style(
            "word",
            Rectangle::new(Point::zero(), Size::new(55, 4)),
            character_style,
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
