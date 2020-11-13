//! Scrolling vertical text alignment.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    rendering::cursor::Cursor,
    style::height_mode::HeightMode,
    StyledTextBox,
};
use embedded_graphics::prelude::*;

/// Align text to the TextBox so that the last lines are always displayed.
///
/// Scrolling alignment works well for terminal-like applications. When text fits into the bounding
/// box, it will be top aligned. After that, rendering switches to bottom aligned, making sure the
/// last lines are always visible.
#[derive(Copy, Clone, Debug)]
pub struct Scrolling;

impl VerticalTextAlignment for Scrolling {
    #[inline]
    fn apply_vertical_alignment<'a, C, F, A, H>(
        cursor: &mut Cursor<F>,
        styled_text_box: &'a StyledTextBox<'a, C, F, A, Self, H>,
    ) where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        H: HeightMode,
    {
        let text_height = styled_text_box
            .style
            .measure_text_height(styled_text_box.text_box.text, cursor.line_width())
            as i32;

        let box_height = styled_text_box.size().height as i32;
        if text_height > box_height {
            let offset = box_height - text_height;

            cursor.position.y += offset
        }
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        primitives::Rectangle,
    };

    use crate::{
        alignment::Scrolling,
        style::{height_mode::Exact, vertical_overdraw::Hidden, TextBoxStyleBuilder},
        TextBox,
    };

    #[test]
    fn scrolling_behaves_as_top_if_lines_dont_overflow() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(Scrolling)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new("word", Rectangle::new(Point::zero(), Point::new(54, 15)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "......................#.",
                "......................#.",
                "#...#..###..#.##...##.#.",
                "#...#.#...#.##..#.#..##.",
                "#.#.#.#...#.#.....#...#.",
                "#.#.#.#...#.#.....#...#.",
                ".#.#...###..#......####.",
                "........................",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
            ])
        );
    }

    #[test]
    fn scrolling_behaves_as_bottom_if_lines_overflow() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(Scrolling)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word word2 word3 word4",
            Rectangle::new(Point::zero(), Point::new(54, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "......................#..###..",
                "......................#.#...#.",
                "#...#..###..#.##...##.#.....#.",
                "#...#.#...#.##..#.#..##...##..",
                "#.#.#.#...#.#.....#...#.....#.",
                "#.#.#.#...#.#.....#...#.#...#.",
                ".#.#...###..#......####..###..",
                "..............................",
                "......................#....#..",
                "......................#...##..",
                "#...#..###..#.##...##.#..#.#..",
                "#...#.#...#.##..#.#..##.#..#..",
                "#.#.#.#...#.#.....#...#.#####.",
                "#.#.#.#...#.#.....#...#....#..",
                ".#.#...###..#......####....#..",
                "..............................",
            ])
        );
    }

    #[test]
    fn scrolling_applies_full_rows_vertical_overflow() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(Scrolling)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word word2 word3 word4",
            Rectangle::new(Point::zero(), Point::new(54, 12)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                              ",
                "                              ",
                "                              ",
                "                              ",
                "                              ",
                "......................#....#..",
                "......................#...##..",
                "#...#..###..#.##...##.#..#.#..",
                "#...#.#...#.##..#.#..##.#..#..",
                "#.#.#.#...#.#.....#...#.#####.",
                "#.#.#.#...#.#.....#...#....#..",
                ".#.#...###..#......####....#..",
                "..............................",
            ])
        );
    }

    #[test]
    fn scrolling_applies_hidden_vertical_overflow() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(Scrolling)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .height_mode(Exact(Hidden))
            .build();

        TextBox::new(
            "word word2 word3 word4",
            Rectangle::new(Point::zero(), Point::new(54, 12)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "#...#.#...#.##..#.#..##...##..",
                "#.#.#.#...#.#.....#...#.....#.",
                "#.#.#.#...#.#.....#...#.#...#.",
                ".#.#...###..#......####..###..",
                "..............................",
                "......................#....#..",
                "......................#...##..",
                "#...#..###..#.##...##.#..#.#..",
                "#...#.#...#.##..#.#..##.#..#..",
                "#.#.#.#...#.#.....#...#.#####.",
                "#.#.#.#...#.#.....#...#....#..",
                ".#.#...###..#......####....#..",
                "..............................",
            ])
        );
    }
}
