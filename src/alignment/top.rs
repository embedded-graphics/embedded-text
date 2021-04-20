//! Top vertical text alignment.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    rendering::cursor::Cursor,
    style::height_mode::HeightMode,
    StyledTextBox,
};
use embedded_graphics::text::renderer::TextRenderer;

/// Align text to the top of the TextBox.
#[derive(Copy, Clone, Debug)]
pub struct TopAligned;

impl VerticalTextAlignment for TopAligned {
    #[inline]
    fn apply_vertical_alignment<'a, F, A, H>(
        _cursor: &mut Cursor,
        _styled_text_box: &'a StyledTextBox<'a, F, A, Self, H>,
    ) where
        F: TextRenderer,
        A: HorizontalTextAlignment,
        H: HeightMode,
    {
        // nothing to do here
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

    use crate::{alignment::TopAligned, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn test_top_alignment() {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .vertical_alignment(TopAligned)
            .build();

        TextBox::new("word", Rectangle::new(Point::zero(), Size::new(55, 16)))
            .into_styled(style)
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
            "                        ",
            "                        ",
            "                        ",
            "                        ",
            "                        ",
            "                        ",
            "                        ",
            "                        ",
        ]);
    }
}
