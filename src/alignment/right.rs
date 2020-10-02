//! Right aligned text.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        cursor::Cursor, line::StyledLinePixelIterator, space_config::UniformSpaceConfig,
        StateFactory, StyledTextBoxIterator,
    },
    style::height_mode::HeightMode,
    StyledTextBox,
};
use embedded_graphics::{drawable::Pixel, fonts::Font, pixelcolor::PixelColor};

/// Marks text to be rendered right aligned.
#[derive(Copy, Clone, Debug)]
pub struct RightAligned;
impl HorizontalTextAlignment for RightAligned {
    const STARTING_SPACES: bool = false;
    const ENDING_SPACES: bool = false;
}

/// State variable used by the right aligned text renderer.
#[derive(Debug)]
pub enum State<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Starts processing a line.
    NextLine(Option<Token<'a>>, Cursor<F>, Parser<'a>),

    /// Renders the processed line.
    DrawLine(StyledLinePixelIterator<'a, C, F, UniformSpaceConfig<F>, RightAligned>),
}

impl<'a, C, F, V, H> StateFactory<'a, F> for StyledTextBox<'a, C, F, RightAligned, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type PixelIteratorState = State<'a, C, F>;

    #[inline]
    #[must_use]
    fn create_state(&self, cursor: Cursor<F>, parser: Parser<'a>) -> Self::PixelIteratorState {
        State::NextLine(None, cursor, parser)
    }
}

impl<C, F, V, H> Iterator for StyledTextBoxIterator<'_, C, F, RightAligned, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                State::NextLine(ref carried_token, mut cursor, ref mut parser) => {
                    if carried_token.is_none() && parser.is_empty() {
                        break None;
                    }

                    let parser_clone = parser.clone();
                    let max_line_width = cursor.line_width();
                    let (width, _, _) =
                        self.style
                            .measure_line(parser, carried_token.clone(), max_line_width);
                    cursor.advance_unchecked(max_line_width - width);

                    self.state = State::DrawLine(StyledLinePixelIterator::new(
                        parser_clone,
                        cursor,
                        UniformSpaceConfig::default(),
                        self.style,
                        carried_token.clone(),
                    ));
                }

                State::DrawLine(ref mut line_iterator) => {
                    if let pixel @ Some(_) = line_iterator.next() {
                        break pixel;
                    }

                    self.state = State::NextLine(
                        line_iterator.remaining_token(),
                        line_iterator.cursor(),
                        line_iterator.parser(),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        primitives::Rectangle,
    };

    use crate::{alignment::RightAligned, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn simple_render() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
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
                "                               ......................#.",
                "                               ......................#.",
                "                               #...#..###..#.##...##.#.",
                "                               #...#.#...#.##..#.#..##.",
                "                               #.#.#.#...#.#.....#...#.",
                "                               #.#.#.#...#.#.....#...#.",
                "                               .#.#...###..#......####.",
                "                               ........................",
            ])
        );
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new("O\rX", Rectangle::new(Point::zero(), Point::new(54, 7)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                                                 ##### ",
                "                                                 #   # ",
                "                                                 ## ## ",
                "                                                 # # # ",
                "                                                 ## ## ",
                "                                                 #   # ",
                "                                                 ##### ",
            ])
        );
    }

    #[test]
    fn simple_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
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
                "                               ......................#.",
                "                               ......................#.",
                "                               #...#..###..#.##...##.#.",
                "                               #...#.#...#.##..#.#..##.",
                "                               #.#.#.#...#.#.....#...#.",
                "                               #.#.#.#...#.#.....#...#.",
                "                               .#.#...###..#......####.",
                "                               ........................",
                "       ................................#...............",
                "       ................................................",
                "       #...#.#.##...###..####..####...##...#.##...####.",
                "       #...#.##..#.....#.#...#.#...#...#...##..#.#...#.",
                "       #.#.#.#......####.#...#.#...#...#...#...#.#...#.",
                "       #.#.#.#.....#...#.####..####....#...#...#..####.",
                "       .#.#..#......####.#.....#......###..#...#.....#.",
                "       ..................#.....#..................###.."
            ])
        );
    }

    #[test]
    fn word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
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
                "                               ......................#.",
                "                               ......................#.",
                "                               #...#..###..#.##...##.#.",
                "                               #...#.#...#.##..#.#..##.",
                "                               #.#.#.#...#.#.....#...#.",
                "                               #.#.#.#...#.#.....#...#.",
                "                               .#.#...###..#......####.",
                "                               ........................",
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
            ])
        );
    }

    #[test]
    fn first_word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
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
            ])
        );
    }
}
