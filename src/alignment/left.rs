//! Left aligned text.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        cursor::Cursor,
        line::{StyledLineIterator, UniformSpaceConfig},
        StateFactory, StyledTextBoxIterator,
    },
    style::StyledTextBox,
    utils::font_ext::FontExt,
};
use embedded_graphics::{drawable::Pixel, fonts::Font, pixelcolor::PixelColor};

/// Marks text to be rendered left aligned.
#[derive(Copy, Clone, Debug)]
pub struct LeftAligned;
impl HorizontalTextAlignment for LeftAligned {
    const STARTING_SPACES: bool = true;
    const ENDING_SPACES: bool = true;
}

/// State variable used by the left aligned text renderer.
#[derive(Debug)]
pub enum State<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Starts processing a line.
    NextLine(Option<Token<'a>>, Cursor<F>, Parser<'a>),

    /// Renders the processed line.
    DrawLine(StyledLineIterator<'a, C, F, UniformSpaceConfig, LeftAligned>),
}

impl<'a, C, F, V> StateFactory<'a, F> for StyledTextBox<'a, C, F, LeftAligned, V>
where
    C: PixelColor,
    F: Font + Copy,
    V: VerticalTextAlignment,
{
    type PixelIteratorState = State<'a, C, F>;

    #[inline]
    #[must_use]
    fn create_state(&self, cursor: Cursor<F>, parser: Parser<'a>) -> Self::PixelIteratorState {
        State::NextLine(None, cursor, parser)
    }
}

impl<C, F, V> Iterator for StyledTextBoxIterator<'_, C, F, LeftAligned, V>
where
    C: PixelColor,
    F: Font + Copy,
    V: VerticalTextAlignment,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                State::NextLine(ref carried_token, cursor, ref parser) => {
                    if !cursor.in_display_area() {
                        break None;
                    }

                    if carried_token.is_none() && parser.is_empty() {
                        break None;
                    }

                    self.state = State::DrawLine(StyledLineIterator::new(
                        parser.clone(),
                        cursor,
                        UniformSpaceConfig {
                            space_width: F::total_char_width(' '),
                        },
                        self.style.text_style,
                        carried_token.clone(),
                    ));
                }

                State::DrawLine(ref mut line_iterator) => {
                    if let pixel @ Some(_) = line_iterator.next() {
                        break pixel;
                    }

                    self.state = State::NextLine(
                        line_iterator.remaining_token(),
                        line_iterator.cursor,
                        line_iterator.parser.clone(),
                    );
                }
            };
        }
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        primitives::Rectangle,
    };

    use crate::{alignment::LeftAligned, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn simple_render() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new("word", Rectangle::new(Point::zero(), Point::new(54, 7)))
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
            ])
        );
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new("O\rX", Rectangle::new(Point::zero(), Point::new(54, 7)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "#####    ",
                "#   #    ",
                "## ##    ",
                "# # #    ",
                "## ##    ",
                "#   #    ",
                "#####    ",
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(54, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "......................#.......                  ",
                "......................#.......                  ",
                "#...#..###..#.##...##.#.......                  ",
                "#...#.#...#.##..#.#..##.......                  ",
                "#.#.#.#...#.#.....#...#.......                  ",
                "#.#.#.#...#.#.....#...#.......                  ",
                ".#.#...###..#......####.......                  ",
                "..............................                  ",
                "................................#...............",
                "................................................",
                "#...#.#.##...###..####..####...##...#.##...####.",
                "#...#.##..#.....#.#...#.#...#...#...##..#.#...#.",
                "#.#.#.#......####.#...#.#...#...#...#...#.#...#.",
                "#.#.#.#.....#...#.####..####....#...#...#..####.",
                ".#.#..#......####.#.....#......###..#...#.....#.",
                "..................#.....#..................###.."
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(47, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(30, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "......................#.......",
                "......................#.......",
                "#...#..###..#.##...##.#.......",
                "#...#.#...#.##..#.#..##.......",
                "#.#.#.#...#.#.....#...#.......",
                "#.#.#.#...#.#.....#...#.......",
                ".#.#...###..#......####.......",
                "..............................",
                "..............................",
                "..............................",
                "......#...#.#.##...###..####..",
                "......#...#.##..#.....#.#...#.",
                "......#.#.#.#......####.#...#.",
                "......#.#.#.#.....#...#.####..",
                ".......#.#..#......####.#.....",
                "........................#....."
            ])
        );
    }

    #[test]
    fn word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word somereallylongword",
            Rectangle::new(Point::zero(), Point::new(54, 23)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(54, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
            ])
        );
    }
}
