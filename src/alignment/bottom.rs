//! Bottom vertical text alignment.
use embedded_graphics::{
    geometry::Dimensions,
    text::renderer::{CharacterStyle, TextRenderer},
};

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    rendering::cursor::Cursor,
    style::{color::Rgb, height_mode::HeightMode},
    TextBox,
};

/// Align text to the bottom of the TextBox.
#[derive(Copy, Clone, Debug)]
pub struct BottomAligned;

impl VerticalTextAlignment for BottomAligned {
    #[inline]
    fn apply_vertical_alignment<'a, F, A, H>(
        cursor: &mut Cursor,
        styled_text_box: &'a TextBox<'a, F, A, Self, H>,
    ) where
        F: TextRenderer + CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb>,
        A: HorizontalTextAlignment,
        H: HeightMode,
    {
        let text_height = styled_text_box.style.measure_text_height(
            &styled_text_box.character_style,
            styled_text_box.text,
            cursor.line_width(),
        ) as i32;

        let box_height = styled_text_box.bounding_box().size.height as i32;
        let offset = box_height - text_height;

        cursor.y += offset
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        mock_display::MockDisplay,
        mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        prelude::*,
        primitives::Rectangle,
    };

    use crate::{
        alignment::BottomAligned,
        style::{height_mode::Exact, vertical_overdraw::Visible, TextBoxStyleBuilder},
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

        let style = TextBoxStyleBuilder::new()
            .vertical_alignment(BottomAligned)
            .build();

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
            .height_mode(Exact(Visible))
            .line_spacing(2)
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
