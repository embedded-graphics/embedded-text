//! Bottom vertical text alignment.
use crate::{
    alignment::horizontal::HorizontalTextAlignment, rendering::cursor::Cursor, style::StyledTextBox,
};
use embedded_graphics::prelude::*;

use super::VerticalTextAlignment;

/// Align text to the bottom of the TextBox.
#[derive(Copy, Clone)]
pub struct Bottom;

impl VerticalTextAlignment for Bottom {
    #[inline]
    fn apply_vertical_alignment<'a, C, F, A>(
        cursor: &mut Cursor<F>,
        styled_text_box: &'a StyledTextBox<'a, C, F, A, Self>,
    ) where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
    {
        let text_height = styled_text_box
            .style
            .measure_text_height(styled_text_box.text_box.text, cursor.line_width());

        let box_height = styled_text_box.size().height;
        let offset = box_height - text_height;

        cursor.position.y += offset as i32
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        primitives::Rectangle,
    };

    use crate::{alignment::vertical, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn test_bottom_alignment() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(vertical::Bottom)
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
}
