//! Top vertical text alignment.
use crate::{
    alignment::horizontal::HorizontalTextAlignment, rendering::cursor::Cursor, style::StyledTextBox,
};
use embedded_graphics::prelude::*;

use super::VerticalTextAlignment;

/// Align text to the top of the TextBox.
#[derive(Copy, Clone)]
pub struct Top;

impl VerticalTextAlignment for Top {
    #[inline]
    fn apply_vertical_alignment<'a, C, F, A>(
        _cursor: &mut Cursor<F>,
        _styled_text_box: &'a StyledTextBox<'a, C, F, A, Self>,
    ) where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
    {
        // nothing to do here
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
    fn test_top_alignment() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(vertical::Top)
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
}
