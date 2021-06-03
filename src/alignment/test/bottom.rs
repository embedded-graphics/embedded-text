//! Bottom vertical text alignment.
use embedded_graphics::{geometry::Dimensions, text::renderer::TextRenderer};

use crate::{
    alignment::{HorizontalAlignment, VerticalAlignment},
    rendering::cursor::Cursor,
    TextBox,
};

#[cfg(test)]
mod test {
    use embedded_graphics::{
        mock_display::MockDisplay,
        mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        prelude::*,
        primitives::Rectangle,
        text::LineHeight,
    };

    use crate::{
        alignment::BottomAligned,
        style::{HeightMode, TextBoxStyle, TextBoxStyleBuilder, VerticalOverdraw},
        utils::test::size_for,
        TextBox,
    };

    fn assert_rendered(text: &str, size: Size, pattern: &[&str]) {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyle::with_vertical_alignment(BottomAligned);

        TextBox::with_textbox_style(
            text,
            Rectangle::new(Point::zero(), size),
            character_style,
            style,
        )
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(pattern);
    }

    #[test]
    fn test_bottom_alignment() {
        assert_rendered(
            "word",
            size_for(&FONT_6X9, 4, 2),
            &[
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "........................",
                "......................#.",
                "......................#.",
                "#...#...##...#.#....###.",
                "#.#.#..#..#..##.#..#..#.",
                "#.#.#..#..#..#.....#..#.",
                ".#.#....##...#......###.",
                "........................",
                "........................",
            ],
        );
    }

    #[test]
    fn test_bottom_alignment_tall_text() {
        assert_rendered(
            "word word2 word3 word4",
            size_for(&FONT_6X9, 5, 2),
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
    fn test_bottom_alignment_tall_text_with_line_spacing() {
        let mut display = MockDisplay::new();
        display.set_allow_out_of_bounds_drawing(true);

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .vertical_alignment(BottomAligned)
            .height_mode(HeightMode::Exact(VerticalOverdraw::Visible))
            .line_height(LineHeight::Pixels(11))
            .build();

        TextBox::with_textbox_style(
            "word1 word2 word3 word4",
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, 5, 2)),
            character_style,
            style,
        )
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "......................#....#..",
            "#...#...##...#.#....###...##..",
            "#.#.#..#..#..##.#..#..#.....#.",
            "#.#.#..#..#..#.....#..#.....#.",
            ".#.#....##...#......###..###..",
            "..............................",
            "..............................",
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
        ]);
    }
}
