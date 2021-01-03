//! Horizontal and vertical center aligned text.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::Token,
    rendering::{cursor::Cursor, space_config::UniformSpaceConfig},
    style::height_mode::HeightMode,
    StyledTextBox,
};
use embedded_graphics::fonts::MonoFont;
use embedded_graphics_core::{geometry::Dimensions, pixelcolor::PixelColor};

/// Marks text to be rendered center aligned.
///
/// This alignment can be used as both horizontal or vertical alignment.
#[derive(Copy, Clone, Debug)]
pub struct CenterAligned;
impl HorizontalTextAlignment for CenterAligned {
    type SpaceConfig = UniformSpaceConfig;

    const STARTING_SPACES: bool = false;
    const ENDING_SPACES: bool = false;

    #[inline]
    fn place_line<F: MonoFont>(
        max_width: u32,
        text_width: u32,
        _n_spaces: u32,
        _carried_token: Option<Token>,
    ) -> (u32, Self::SpaceConfig) {
        (
            (max_width - text_width + 1) / 2,
            UniformSpaceConfig::new(F::CHARACTER_SIZE.width + F::CHARACTER_SPACING),
        )
    }
}

impl VerticalTextAlignment for CenterAligned {
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
        let offset = (box_height - text_height) / 2;

        cursor.position.y += offset;
    }
}

#[cfg(test)]
mod test_horizontal {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    };
    use embedded_graphics_core::primitives::Rectangle;

    use crate::{alignment::CenterAligned, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn simple_render() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new("word", Rectangle::new(Point::zero(), Size::new(55, 8)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "                ......................#.               ",
            "                ......................#.               ",
            "                #...#..###..#.##...##.#.               ",
            "                #...#.#...#.##..#.#..##.               ",
            "                #.#.#.#...#.#.....#...#.               ",
            "                #.#.#.#...#.#.....#...#.               ",
            "                .#.#...###..#......####.               ",
            "                ........................               ",
        ]);
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new("O\rX", Rectangle::new(Point::zero(), Size::new(55, 8)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "                         #####    ",
            "                         #   #    ",
            "                         ## ##    ",
            "                         # # #    ",
            "                         ## ##    ",
            "                         #   #    ",
            "                         #####    ",
        ]);
    }

    #[test]
    fn simple_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word wrapping",
            Rectangle::new(Point::zero(), Size::new(55, 16)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "                ......................#.               ",
            "                ......................#.               ",
            "                #...#..###..#.##...##.#.               ",
            "                #...#.#...#.##..#.#..##.               ",
            "                #.#.#.#...#.#.....#...#.               ",
            "                #.#.#.#...#.#.....#...#.               ",
            "                .#.#...###..#......####.               ",
            "                ........................               ",
            "    ................................#...............   ",
            "    ................................................   ",
            "    #...#.#.##...###..####..####...##...#.##...####.   ",
            "    #...#.##..#.....#.#...#.#...#...#...##..#.#...#.   ",
            "    #.#.#.#......####.#...#.#...#...#...#...#.#...#.   ",
            "    #.#.#.#.....#...#.####..####....#...#...#..####.   ",
            "    .#.#..#......####.#.....#......###..#...#.....#.   ",
            "    ..................#.....#..................###..   ",
        ]);
    }

    #[test]
    fn word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word somereallylongword",
            Rectangle::new(Point::zero(), Size::new(55, 24)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "                ......................#.               ",
            "                ......................#.               ",
            "                #...#..###..#.##...##.#.               ",
            "                #...#.#...#.##..#.#..##.               ",
            "                #.#.#.#...#.#.....#...#.               ",
            "                #.#.#.#...#.#.....#...#.               ",
            "                .#.#...###..#......####.               ",
            "                ........................               ",
            " ...........................................##....##...",
            " ............................................#.....#...",
            " .####..###..##.#...###..#.##...###...###....#.....#...",
            " #.....#...#.#.#.#.#...#.##..#.#...#.....#...#.....#...",
            " .###..#...#.#...#.#####.#.....#####..####...#.....#...",
            " ....#.#...#.#...#.#.....#.....#.....#...#...#.....#...",
            " ####...###..#...#..###..#......###...####..###...###..",
            " ......................................................",
            " .......##...........................................#.",
            " ........#...........................................#.",
            " #...#...#....###..#.##...####.#...#..###..#.##...##.#.",
            " #...#...#...#...#.##..#.#...#.#...#.#...#.##..#.#..##.",
            " #...#...#...#...#.#...#.#...#.#.#.#.#...#.#.....#...#.",
            " .####...#...#...#.#...#..####.#.#.#.#...#.#.....#...#.",
            " ....#..###...###..#...#.....#..#.#...###..#......####.",
            " .###.....................###..........................",
        ]);
    }

    #[test]
    fn first_word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "somereallylongword",
            Rectangle::new(Point::zero(), Size::new(55, 16)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            " ...........................................##....##...",
            " ............................................#.....#...",
            " .####..###..##.#...###..#.##...###...###....#.....#...",
            " #.....#...#.#.#.#.#...#.##..#.#...#.....#...#.....#...",
            " .###..#...#.#...#.#####.#.....#####..####...#.....#...",
            " ....#.#...#.#...#.#.....#.....#.....#...#...#.....#...",
            " ####...###..#...#..###..#......###...####..###...###..",
            " ......................................................",
            " .......##...........................................#.",
            " ........#...........................................#.",
            " #...#...#....###..#.##...####.#...#..###..#.##...##.#.",
            " #...#...#...#...#.##..#.#...#.#...#.#...#.##..#.#..##.",
            " #...#...#...#...#.#...#.#...#.#.#.#.#...#.#.....#...#.",
            " .####...#...#...#.#...#..####.#.#.#.#...#.#.....#...#.",
            " ....#..###...###..#...#.....#..#.#...###..#......####.",
            " .###.....................###..........................",
        ]);
    }
}

#[cfg(test)]
mod test_vertical {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    };
    use embedded_graphics_core::primitives::Rectangle;

    use crate::{alignment::CenterAligned, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn test_center_alignment() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new("word", Rectangle::new(Point::zero(), Size::new(55, 16)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
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
            "                        ",
            "                        ",
            "                        ",
            "                        ",
        ]);
    }

    #[test]
    fn soft_hyphen_rendering() {
        let text = "soft\u{AD}hyphen";

        let mut display = MockDisplay::new();

        let bounds = Rectangle::new(Point::new(0, 0), Size::new(36, 31));
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new(text, bounds)
            .into_styled(textbox_style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "                 ##   #             ",
            "                #  #  #             ",
            "    ####  ###   #    ###            ",
            "   #     #   # ###    #    #####    ",
            "    ###  #   #  #     #             ",
            "       # #   #  #     #  #          ",
            "   ####   ###   #      ##           ",
            "                                    ",
            "#                 #                 ",
            "#                 #                 ",
            "# ##  #   # ####  # ##   ###  # ##  ",
            "##  # #   # #   # ##  # #   # ##  # ",
            "#   # #   # #   # #   # ##### #   # ",
            "#   #  #### ####  #   # #     #   # ",
            "#   #     # #     #   #  ###  #   # ",
            "       ###  #                       ",
        ]);
    }
}
