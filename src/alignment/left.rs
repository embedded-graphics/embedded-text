//! Left aligned text.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    rendering::{
        line::StyledLinePixelIterator, space_config::UniformSpaceConfig, RendererFactory,
        StyledTextBoxIterator,
    },
    style::{color::Rgb, height_mode::HeightMode},
    StyledTextBox,
};
use embedded_graphics::{fonts::MonoFont, pixelcolor::PixelColor};

/// Marks text to be rendered left aligned.
#[derive(Copy, Clone, Debug)]
pub struct LeftAligned;
impl HorizontalTextAlignment for LeftAligned {
    const STARTING_SPACES: bool = true;
    const ENDING_SPACES: bool = true;
}

impl<'a, C, F, V, H> RendererFactory<'a, C> for StyledTextBox<'a, C, F, LeftAligned, V, H>
where
    C: PixelColor + From<Rgb>,
    F: MonoFont,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Renderer = StyledTextBoxIterator<'a, C, F, LeftAligned, V, H, UniformSpaceConfig<F>>;

    #[inline]
    #[must_use]
    fn create_renderer(&self) -> Self::Renderer {
        StyledTextBoxIterator::new(self, |style, carried, cursor, parser| {
            StyledLinePixelIterator::new(
                parser,
                cursor,
                UniformSpaceConfig::default(),
                style,
                carried,
            )
        })
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    };
    use embedded_graphics_core::primitives::Rectangle;

    use crate::{alignment::LeftAligned, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn simple_render() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new("word", Rectangle::new(Point::zero(), Size::new(55, 8)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "......................#.",
            "......................#.",
            "#...#..###..#.##...##.#.",
            "#...#.#...#.##..#.#..##.",
            "#.#.#.#...#.#.....#...#.",
            "#.#.#.#...#.#.....#...#.",
            ".#.#...###..#......####.",
            "........................",
        ]);
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new("O\rX", Rectangle::new(Point::zero(), Size::new(55, 8)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "#####    ",
            "#   #    ",
            "## ##    ",
            "# # #    ",
            "## ##    ",
            "#   #    ",
            "#####    ",
        ]);
    }

    #[test]
    fn simple_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
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
            "......................#.                        ",
            "......................#.                        ",
            "#...#..###..#.##...##.#.                        ",
            "#...#.#...#.##..#.#..##.                        ",
            "#.#.#.#...#.#.....#...#.                        ",
            "#.#.#.#...#.#.....#...#.                        ",
            ".#.#...###..#......####.                        ",
            "........................                        ",
            "................................#...............",
            "................................................",
            "#...#.#.##...###..####..####...##...#.##...####.",
            "#...#.##..#.....#.#...#.#...#...#...##..#.#...#.",
            "#.#.#.#......####.#...#.#...#...#...#...#.#...#.",
            "#.#.#.#.....#...#.####..####....#...#...#..####.",
            ".#.#..#......####.#.....#......###..#...#.....#.",
            "..................#.....#..................###..",
        ]);
    }

    #[test]
    fn simple_word_wrapping_by_space() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "wrapping word",
            Rectangle::new(Point::zero(), Size::new(48, 16)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "................................#...............",
            "................................................",
            "#...#.#.##...###..####..####...##...#.##...####.",
            "#...#.##..#.....#.#...#.#...#...#...##..#.#...#.",
            "#.#.#.#......####.#...#.#...#...#...#...#.#...#.",
            "#.#.#.#.....#...#.####..####....#...#...#..####.",
            ".#.#..#......####.#.....#......###..#...#.....#.",
            "..................#.....#..................###..",
            "......................#.                        ",
            "......................#.                        ",
            "#...#..###..#.##...##.#.                        ",
            "#...#.#...#.##..#.#..##.                        ",
            "#.#.#.#...#.#.....#...#.                        ",
            "#.#.#.#...#.#.....#...#.                        ",
            ".#.#...###..#......####.                        ",
            "........................                        ",
        ]);
    }

    #[test]
    fn simple_word_wrapping_with_line_spacing() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .line_spacing(2)
            .build();

        TextBox::new(
            "wrapping word",
            Rectangle::new(Point::zero(), Size::new(48, 51)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "................................#...............",
            "................................................",
            "#...#.#.##...###..####..####...##...#.##...####.",
            "#...#.##..#.....#.#...#.#...#...#...##..#.#...#.",
            "#.#.#.#......####.#...#.#...#...#...#...#.#...#.",
            "#.#.#.#.....#...#.####..####....#...#...#..####.",
            ".#.#..#......####.#.....#......###..#...#.....#.",
            "..................#.....#..................###..",
            "                                                ",
            "                                                ",
            "......................#.                        ",
            "......................#.                        ",
            "#...#..###..#.##...##.#.                        ",
            "#...#.#...#.##..#.#..##.                        ",
            "#.#.#.#...#.#.....#...#.                        ",
            "#.#.#.#...#.#.....#...#.                        ",
            ".#.#...###..#......####.                        ",
            "........................                        ",
        ]);
    }

    #[test]
    fn simple_word_wrapping_with_negative_line_spacing() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .line_spacing(-1)
            .build();

        TextBox::new(
            "wrapping word",
            Rectangle::new(Point::zero(), Size::new(48, 51)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "................................#...............",
            "................................................",
            "#...#.#.##...###..####..####...##...#.##...####.",
            "#...#.##..#.....#.#...#.#...#...#...##..#.#...#.",
            "#.#.#.#......####.#...#.#...#...#...#...#.#...#.",
            "#.#.#.#.....#...#.####..####....#...#...#..####.",
            ".#.#..#......####.#.....#......###..#...#.....#.",
            "......................#.#..................###..", // note the first p being drawn over
            "......................#.                        ",
            "#...#..###..#.##...##.#.                        ",
            "#...#.#...#.##..#.#..##.                        ",
            "#.#.#.#...#.#.....#...#.                        ",
            "#.#.#.#...#.#.....#...#.                        ",
            ".#.#...###..#......####.                        ",
            "........................                        ",
        ]);
    }

    #[test]
    fn whitespace_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word  wrap",
            Rectangle::new(Point::zero(), Size::new(31, 16)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "......................#.......",
            "......................#.......",
            "#...#..###..#.##...##.#.......",
            "#...#.#...#.##..#.#..##.......",
            "#.#.#.#...#.#.....#...#.......",
            "#.#.#.#...#.#.....#...#.......",
            ".#.#...###..#......####.......",
            "..............................",
            "........................      ",
            "........................      ",
            "#...#.#.##...###..####..      ",
            "#...#.##..#.....#.#...#.      ",
            "#.#.#.#......####.#...#.      ",
            "#.#.#.#.....#...#.####..      ",
            ".#.#..#......####.#.....      ",
            "..................#.....      ",
        ]);
    }

    #[test]
    fn word_longer_than_line_wraps_word_and_removes_a_space() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word  somereallylongword",
            Rectangle::new(Point::zero(), Size::new(55, 24)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "......................#.......                        ",
            "......................#.......                        ",
            "#...#..###..#.##...##.#.......                        ",
            "#...#.#...#.##..#.#..##.......                        ",
            "#.#.#.#...#.#.....#...#.......                        ",
            "#.#.#.#...#.#.....#...#.......                        ",
            ".#.#...###..#......####.......                        ",
            "..............................                        ",
            "...........................................##....##...",
            "............................................#.....#...",
            ".####..###..##.#...###..#.##...###...###....#.....#...",
            "#.....#...#.#.#.#.#...#.##..#.#...#.....#...#.....#...",
            ".###..#...#.#...#.#####.#.....#####..####...#.....#...",
            "....#.#...#.#...#.#.....#.....#.....#...#...#.....#...",
            "####...###..#...#..###..#......###...####..###...###..",
            "......................................................",
            ".......##...........................................#.",
            "........#...........................................#.",
            "#...#...#....###..#.##...####.#...#..###..#.##...##.#.",
            "#...#...#...#...#.##..#.#...#.#...#.#...#.##..#.#..##.",
            "#...#...#...#...#.#...#.#...#.#.#.#.#...#.#.....#...#.",
            ".####...#...#...#.#...#..####.#.#.#.#...#.#.....#...#.",
            "....#..###...###..#...#.....#..#.#...###..#......####.",
            ".###.....................###..........................",
        ]);
    }

    #[test]
    fn first_word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
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
            "...........................................##....##...",
            "............................................#.....#...",
            ".####..###..##.#...###..#.##...###...###....#.....#...",
            "#.....#...#.#.#.#.#...#.##..#.#...#.....#...#.....#...",
            ".###..#...#.#...#.#####.#.....#####..####...#.....#...",
            "....#.#...#.#...#.#.....#.....#.....#...#...#.....#...",
            "####...###..#...#..###..#......###...####..###...###..",
            "......................................................",
            ".......##...........................................#.",
            "........#...........................................#.",
            "#...#...#....###..#.##...####.#...#..###..#.##...##.#.",
            "#...#...#...#...#.##..#.#...#.#...#.#...#.##..#.#..##.",
            "#...#...#...#...#.#...#.#...#.#.#.#.#...#.#.....#...#.",
            ".####...#...#...#.#...#..####.#.#.#.#...#.#.....#...#.",
            "....#..###...###..#...#.....#..#.#...###..#......####.",
            ".###.....................###..........................",
        ]);
    }

    #[test]
    fn soft_hyphen_rendering() {
        let text = "soft\u{AD}hyphen";

        let mut display = MockDisplay::new();

        let bounds = Rectangle::new(Point::new(0, 0), Size::new(36, 31));
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new(text, bounds)
            .into_styled(textbox_style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "              ##   #                 ",
            "             #  #  #                 ",
            " ####  ###   #    ###                ",
            "#     #   # ###    #    #####        ",
            " ###  #   #  #     #                 ",
            "    # #   #  #     #  #              ",
            "####   ###   #      ##               ",
            "                                     ",
            "#                 #                  ",
            "#                 #                  ",
            "# ##  #   # ####  # ##   ###  # ##   ",
            "##  # #   # #   # ##  # #   # ##  #  ",
            "#   # #   # #   # #   # ##### #   #  ",
            "#   #  #### ####  #   # #     #   #  ",
            "#   #     # #     #   #  ###  #   #  ",
            "       ###  #                        ",
        ]);
    }
}
