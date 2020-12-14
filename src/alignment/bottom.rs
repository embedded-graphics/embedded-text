//! Bottom vertical text alignment.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    rendering::cursor::Cursor,
    style::height_mode::HeightMode,
    StyledTextBox,
};
use embedded_graphics::prelude::*;

/// Align text to the bottom of the TextBox.
#[derive(Copy, Clone, Debug)]
pub struct BottomAligned;

impl VerticalTextAlignment for BottomAligned {
    #[inline]
    fn apply_vertical_alignment<'a, C, F, A, H>(
        cursor: &mut Cursor<F>,
        styled_text_box: &'a StyledTextBox<'a, C, F, A, Self, H>,
    ) where
        C: PixelColor,
        F: MonoFont,
        A: HorizontalTextAlignment,
        H: HeightMode,
    {
        let text_height = styled_text_box
            .style
            .measure_text_height(styled_text_box.text_box.text, cursor.line_width())
            as i32;

        let box_height = styled_text_box.bounding_box().size.height as i32;
        let offset = box_height - text_height;

        cursor.position.y += offset
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    };
    use embedded_graphics_core::primitives::Rectangle;

    use crate::{
        alignment::BottomAligned, style::height_mode::Exact, style::vertical_overdraw::Visible,
        style::TextBoxStyleBuilder, TextBox,
    };

    #[test]
    fn test_bottom_alignment() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(BottomAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new("word", Rectangle::new(Point::zero(), Size::new(54, 16)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "......................#.",
                "......................#.",
                "#...#..###..#.##...##.#.",
                "#...#.#...#.##..#.#..##.",
                "#.#.#.#...#.#.....#...#.",
                "#.#.#.#...#.#.....#...#.",
                ".#.#...###..#......####.",
                "........................",
            ])
        );
    }

    #[test]
    fn test_bottom_alignment_tall_text() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(BottomAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word1 word2 word3 word4",
            Rectangle::new(Point::zero(), Size::new(30, 16)),
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
    fn test_bottom_alignment_tall_text_with_line_spacing() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(BottomAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .height_mode(Exact(Visible))
            .line_spacing(2)
            .build();

        TextBox::new(
            "word1 word2 word3 word4",
            Rectangle::new(Point::zero(), Size::new(30, 16)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "#...#..###..#.##...##.#.....#.",
                "#...#.#...#.##..#.#..##...##..",
                "#.#.#.#...#.#.....#...#.....#.",
                "#.#.#.#...#.#.....#...#.#...#.",
                ".#.#...###..#......####..###..",
                "..............................",
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
}
