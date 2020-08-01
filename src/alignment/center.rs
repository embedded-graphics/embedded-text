//! Center aligned text.
use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{
        cursor::Cursor,
        line::{StyledLineIterator, UniformSpaceConfig},
        StateFactory, StyledTextBoxIterator,
    },
    style::StyledTextBox,
    utils::font_ext::FontExt,
};
use embedded_graphics::{drawable::Pixel, fonts::Font, pixelcolor::PixelColor};

/// Marks text to be rendered center aligned.
#[derive(Copy, Clone, Debug)]
pub struct CenterAligned;
impl TextAlignment for CenterAligned {
    const STARTING_SPACES: bool = false;
    const ENDING_SPACES: bool = false;
}

/// State variable used by the center aligned text renderer.
#[derive(Debug)]
pub enum State<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Starts processing a line.
    NextLine(Option<Token<'a>>, Cursor<F>),

    /// Renders the processed line.
    DrawLine(StyledLineIterator<'a, C, F, UniformSpaceConfig, CenterAligned>),
}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, CenterAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = State<'a, C, F>;

    #[inline]
    #[must_use]
    fn create_state(&self) -> Self::PixelIteratorState {
        State::NextLine(None, Cursor::new(self.text_box.bounds))
    }
}

impl<C, F> Iterator for StyledTextBoxIterator<'_, C, F, CenterAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                State::NextLine(ref carried_token, ref mut cursor) => {
                    if !cursor.in_display_area() {
                        break None;
                    }

                    if carried_token.is_none() && self.parser.is_empty() {
                        break None;
                    }

                    let max_line_width = cursor.line_width();
                    let (width, _, _) = self.style.measure_line(
                        &mut self.parser.clone(),
                        carried_token.clone(),
                        max_line_width,
                    );
                    cursor.advance((max_line_width - width + 1) / 2);

                    self.state = State::DrawLine(StyledLineIterator::new(
                        self.parser.clone(),
                        *cursor,
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

                    self.parser = line_iterator.parser.clone();

                    let carried_token = match line_iterator.remaining_token() {
                        Some(Token::NewLine) => {
                            line_iterator.cursor.new_line();
                            line_iterator.cursor.carriage_return();
                            None
                        }

                        Some(Token::CarriageReturn) => {
                            line_iterator.cursor.carriage_return();
                            None
                        }

                        c => {
                            line_iterator.cursor.new_line();
                            line_iterator.cursor.carriage_return();
                            c
                        }
                    };

                    self.state = State::NextLine(carried_token, line_iterator.cursor);
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

    use crate::{alignment::CenterAligned, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn simple_render() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
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
                "                ......................#.               ",
                "                ......................#.               ",
                "                #...#..###..#.##...##.#.               ",
                "                #...#.#...#.##..#.#..##.               ",
                "                #.#.#.#...#.#.....#...#.               ",
                "                #.#.#.#...#.#.....#...#.               ",
                "                .#.#...###..#......####.               ",
                "                ........................               ",
            ])
        );
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new("O\rX", Rectangle::new(Point::zero(), Point::new(54, 7)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                         #####    ",
                "                         #   #    ",
                "                         ## ##    ",
                "                         # # #    ",
                "                         ## ##    ",
                "                         #   #    ",
                "                         #####    ",
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(54, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
                "    ..................#.....#..................###..   "
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(54, 23)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
            ])
        );
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
