//! Right aligned text
use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{
        character::StyledCharacterIterator, whitespace::EmptySpaceIterator, StateFactory,
        StyledTextBoxIterator,
    },
    style::StyledTextBox,
    utils::{font_ext::FontExt, rect_ext::RectExt},
};
use embedded_graphics::prelude::*;

use core::str::Chars;

/// Marks text to be rendered right aligned
#[derive(Copy, Clone, Debug)]
pub struct RightAligned;
impl TextAlignment for RightAligned {}

/// State variable used by the right aligned text renderer
#[derive(Debug)]
pub enum RightAlignedState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// This state processes the next token in the text.
    NextWord(bool),

    /// This state handles a line break after a newline character or word wrapping.
    LineBreak(Chars<'a>),

    /// This state measures the next line to calculate the position of the first word.
    MeasureLine(Chars<'a>),

    /// This state processes the next character in a word.
    DrawWord(Chars<'a>),

    /// This state renders a character, then passes the rest of the character iterator to DrawWord.
    DrawCharacter(Chars<'a>, StyledCharacterIterator<C, F>),

    /// This state renders whitespace.
    DrawWhitespace(u32, EmptySpaceIterator<C, F>),
}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, RightAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = RightAlignedState<'a, C, F>;

    #[inline]
    #[must_use]
    fn create_state() -> Self::PixelIteratorState {
        RightAlignedState::MeasureLine("".chars())
    }
}

impl<C, F> Iterator for StyledTextBoxIterator<'_, C, F, RightAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.cursor.in_display_area() {
                break None;
            }

            match self.state {
                RightAlignedState::LineBreak(ref remaining) => {
                    self.cursor.carriage_return();
                    self.cursor.new_line();
                    self.state = RightAlignedState::MeasureLine(remaining.clone());
                }

                RightAlignedState::MeasureLine(ref remaining) => {
                    let max_line_width = RectExt::size(self.cursor.bounds).width;

                    // initial width is the width of the characters carried over to this row
                    let (mut total_width, fits) = F::max_fitting(remaining.clone(), max_line_width);

                    // in some rare cases, the carried over text may not fit into a single line
                    if fits {
                        let mut last_whitespace_width = 0;
                        let mut first_word = true;
                        for token in self.parser.clone() {
                            if total_width >= max_line_width {
                                break;
                            }
                            match token {
                                Token::NewLine => {
                                    break;
                                }

                                Token::Whitespace(n) if total_width == 0 => {
                                    total_width = (n * F::char_width(' ')).min(max_line_width);
                                }

                                Token::Whitespace(n) => {
                                    last_whitespace_width =
                                        (n * F::char_width(' ')).min(max_line_width - total_width);
                                }

                                Token::Word(w) => {
                                    let word_width = w.chars().map(F::char_width).sum::<u32>();
                                    if last_whitespace_width + word_width + total_width
                                        <= max_line_width
                                    {
                                        total_width += last_whitespace_width + word_width;
                                        last_whitespace_width = 0;
                                        first_word = false;
                                    } else {
                                        if first_word {
                                            total_width =
                                                F::max_fitting(w.chars(), max_line_width).0;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    self.cursor.advance(max_line_width - total_width);
                    self.state = if remaining.clone().next().is_none() {
                        RightAlignedState::NextWord(true)
                    } else {
                        RightAlignedState::DrawWord(remaining.clone())
                    }
                }

                RightAlignedState::NextWord(first_word) => {
                    if let Some(token) = self.parser.next() {
                        match token {
                            Token::Word(w) => {
                                // measure w to see if it fits in current line
                                if first_word
                                    || self
                                        .cursor
                                        .fits_in_line(w.chars().map(F::char_width).sum::<u32>())
                                {
                                    self.state = RightAlignedState::DrawWord(w.chars());
                                } else {
                                    self.state = RightAlignedState::LineBreak(w.chars());
                                }
                            }

                            Token::Whitespace(n) => {
                                // TODO character spacing!
                                // word wrapping, also applied for whitespace sequences
                                let width = F::char_width(' ');
                                self.state = if self.cursor.fits_in_line(width) {
                                    RightAlignedState::DrawWhitespace(
                                        n - 1,
                                        EmptySpaceIterator::new(
                                            width,
                                            self.cursor.position,
                                            self.style.text_style,
                                        ),
                                    )
                                } else {
                                    RightAlignedState::NextWord(first_word)
                                }
                            }

                            Token::NewLine => {
                                self.state = RightAlignedState::LineBreak("".chars());
                            }
                        }
                    } else {
                        break None;
                    }
                }

                RightAlignedState::DrawWord(ref mut chars_iterator) => {
                    let mut copy = chars_iterator.clone();
                    self.state = if let Some(c) = copy.next() {
                        // TODO character spacing!
                        let current_pos = self.cursor.position;

                        if self.cursor.advance_char(c) {
                            RightAlignedState::DrawCharacter(
                                copy,
                                StyledCharacterIterator::new(c, current_pos, self.style.text_style),
                            )
                        } else {
                            // word wrapping
                            RightAlignedState::LineBreak(chars_iterator.clone())
                        }
                    } else {
                        RightAlignedState::NextWord(false)
                    }
                }

                RightAlignedState::DrawWhitespace(n, ref mut iterator) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    self.state = if n == 0 {
                        // no more spaces to draw
                        self.cursor.advance_char(' ');
                        RightAlignedState::NextWord(false)
                    } else {
                        let width = F::char_width(' ');
                        if self.cursor.advance(width) {
                            // draw next space
                            RightAlignedState::DrawWhitespace(
                                n - 1,
                                EmptySpaceIterator::new(
                                    width,
                                    self.cursor.position,
                                    self.style.text_style,
                                ),
                            )
                        } else {
                            // word wrapping, also applied for whitespace sequences
                            // eat the spaces from the start of next line
                            RightAlignedState::LineBreak("".chars())
                        }
                    }
                }

                RightAlignedState::DrawCharacter(ref chars_iterator, ref mut iterator) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    self.state = RightAlignedState::DrawWord(chars_iterator.clone());
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

        TextBox::new("word", Rectangle::new(Point::zero(), Point::new(54, 54)))
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
    fn simple_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word wrapping",
            Rectangle::new(Point::zero(), Point::new(54, 54)),
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
            Rectangle::new(Point::zero(), Point::new(54, 54)),
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
            Rectangle::new(Point::zero(), Point::new(54, 54)),
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
