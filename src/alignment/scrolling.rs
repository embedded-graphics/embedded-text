//! Scrolling vertical text alignment.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    rendering::cursor::Cursor,
    style::{color::Rgb, height_mode::HeightMode},
    StyledTextBox,
};
use embedded_graphics::{
    geometry::Dimensions,
    text::{CharacterStyle, TextRenderer},
};

/// Align text to the TextBox so that the last lines are always displayed.
///
/// Scrolling alignment works well for terminal-like applications. When text fits into the bounding
/// box, it will be top aligned. After that, rendering switches to bottom aligned, making sure the
/// last lines are always visible.
#[derive(Copy, Clone, Debug)]
pub struct Scrolling;

impl VerticalTextAlignment for Scrolling {
    #[inline]
    fn apply_vertical_alignment<'a, F, A, H>(
        cursor: &mut Cursor,
        styled_text_box: &'a StyledTextBox<'a, F, A, Self, H>,
    ) where
        F: TextRenderer + CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb>,
        A: HorizontalTextAlignment,
        H: HeightMode,
    {
        let text_height = styled_text_box
            .style
            .measure_text_height(styled_text_box.text_box.text, cursor.line_width())
            as i32;

        let box_height = styled_text_box.bounding_box().size.height as i32;
        if text_height > box_height {
            let offset = box_height - text_height;

            cursor.position.y += offset
        }
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        mock_display::MockDisplay,
        mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        prelude::*,
        primitives::Rectangle,
    };

    use crate::{
        alignment::Scrolling,
        style::{height_mode::Exact, vertical_overdraw::Hidden, TextBoxStyleBuilder},
        utils::test::size_for,
        TextBox,
    };

    fn assert_rendered(text: &str, size: Size, pattern: &[&str]) {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .vertical_alignment(Scrolling)
            .build();

        TextBox::new(text, Rectangle::new(Point::zero(), size))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(pattern);
    }

    #[test]
    fn scrolling_behaves_as_top_if_lines_dont_overflow() {
        assert_rendered(
            "word",
            size_for(Font6x9, 4, 2),
            &[
                "........................",
                "......................#.",
                "......................#.",
                "#...#...##...#.#....###.",
                "#.#.#..#..#..##.#..#..#.",
                "#.#.#..#..#..#.....#..#.",
                ".#.#....##...#......###.",
                "........................",
                "........................",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
            ],
        );
    }

    #[test]
    fn scrolling_behaves_as_bottom_if_lines_overflow() {
        assert_rendered(
            "word word2 word3 word4",
            size_for(Font6x9, 5, 2),
            &[
                "..............................",
                "......................#..####.",
                "......................#....#..",
                "#...#...##...#.#....###...##..",
                "#.#.#..#..#..##.#..#..#.....#.",
                "#.#.#..#..#..#.....#..#.....#.",
                ".#.#....##...#......###..###..",
                "..............................",
                "..............................",
                "..............................",
                "......................#....#..",
                "......................#...##..",
                "#...#...##...#.#....###..#.#..",
                "#.#.#..#..#..##.#..#..#.#..#..",
                "#.#.#..#..#..#.....#..#.#####.",
                ".#.#....##...#......###....#..",
                "..............................",
                "..............................",
            ],
        );
    }

    #[test]
    fn scrolling_applies_full_rows_vertical_overflow() {
        assert_rendered(
            "word word2 word3 word4",
            size_for(Font6x9, 5, 2) - Size::new(0, 5),
            &[
                "                              ",
                "                              ",
                "                              ",
                "                              ",
                "..............................",
                "......................#....#..",
                "......................#...##..",
                "#...#...##...#.#....###..#.#..",
                "#.#.#..#..#..##.#..#..#.#..#..",
                "#.#.#..#..#..#.....#..#.#####.",
                ".#.#....##...#......###....#..",
                "..............................",
                "..............................",
            ],
        );
    }

    #[test]
    fn scrolling_applies_hidden_vertical_overflow() {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .vertical_alignment(Scrolling)
            .height_mode(Exact(Hidden))
            .build();

        TextBox::new(
            "word word2 word3 word4",
            Rectangle::new(Point::zero(), size_for(Font6x9, 5, 2) - Size::new(0, 5)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "#.#.#..#..#..#.....#..#.....#.",
            ".#.#....##...#......###..###..",
            "..............................",
            "..............................",
            "..............................",
            "......................#....#..",
            "......................#...##..",
            "#...#...##...#.#....###..#.#..",
            "#.#.#..#..#..##.#..#..#.#..#..",
            "#.#.#..#..#..#.....#..#.#####.",
            ".#.#....##...#......###....#..",
            "..............................",
            "..............................",
        ]);
    }
}
