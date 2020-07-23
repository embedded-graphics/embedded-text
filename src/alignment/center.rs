//! Center aligned text
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

/// Marks text to be rendered center aligned
#[derive(Copy, Clone, Debug)]
pub struct CenterAligned;
impl TextAlignment for CenterAligned {}

/// State variable used by the center aligned text renderer
#[derive(Debug)]
pub enum CenterAlignedState<'a, C, F>
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

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, CenterAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = CenterAlignedState<'a, C, F>;

    #[inline]
    #[must_use]
    fn create_state() -> Self::PixelIteratorState {
        CenterAlignedState::MeasureLine("".chars())
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
            if !self.cursor.in_display_area() {
                break None;
            }

            match self.state {
                CenterAlignedState::LineBreak(ref remaining) => {
                    self.cursor.carriage_return();
                    self.cursor.new_line();
                    self.state = CenterAlignedState::MeasureLine(remaining.clone());
                }

                CenterAlignedState::MeasureLine(ref remaining) => {
                    let max_line_width = RectExt::size(self.cursor.bounds).width;

                    // initial width is the width of the characters carried over to this row
                    let measurement = F::measure_line(remaining.clone(), max_line_width);

                    let mut total_width = measurement.width;

                    // in some rare cases, the carried over text may not fit into a single line
                    if measurement.fits_line {
                        let mut last_whitespace_width = 0;
                        let mut first_word = true;

                        for token in self.parser.clone() {
                            match token {
                                Token::NewLine => {
                                    break;
                                }

                                Token::Whitespace(n) if total_width == 0 => {
                                    total_width =
                                        (n * F::total_char_width(' ')).min(max_line_width);
                                }

                                Token::Whitespace(n) => {
                                    last_whitespace_width = (n * F::total_char_width(' '))
                                        .min(max_line_width - total_width);
                                }

                                Token::Word(w) => {
                                    let word_measurement = F::measure_line(
                                        w.chars(),
                                        max_line_width - total_width - last_whitespace_width,
                                    );
                                    if word_measurement.fits_line {
                                        total_width +=
                                            last_whitespace_width + word_measurement.width;
                                        last_whitespace_width = 0;
                                        first_word = false;
                                    } else {
                                        if first_word {
                                            total_width =
                                                F::measure_line(w.chars(), max_line_width).width;
                                        }
                                        break;
                                    }
                                }
                            }
                            if total_width >= max_line_width {
                                break;
                            }
                        }
                    }

                    self.cursor.advance((max_line_width - total_width + 1) / 2);
                    self.state = if remaining.as_str().is_empty() {
                        CenterAlignedState::NextWord(true)
                    } else {
                        CenterAlignedState::DrawWord(remaining.clone())
                    }
                }

                CenterAlignedState::NextWord(first_word) => {
                    if let Some(token) = self.parser.next() {
                        match token {
                            Token::Word(w) => {
                                // measure w to see if it fits in current line
                                if first_word || self.cursor.fits_in_line(F::str_width(w)) {
                                    self.state = CenterAlignedState::DrawWord(w.chars());
                                } else {
                                    self.state = CenterAlignedState::LineBreak(w.chars());
                                }
                            }

                            Token::Whitespace(n) => {
                                // word wrapping, also applied for whitespace sequences
                                let width = F::total_char_width(' ');
                                let mut lookahead = self.parser.clone();
                                if let Some(Token::Word(w)) = lookahead.next() {
                                    // only render whitespace if next is word and next doesn't wrap
                                    let n_width = F::str_width(w);

                                    let pos = self.cursor.position;
                                    self.state = if self.cursor.fits_in_line(n_width + width) {
                                        self.cursor.advance(width);
                                        CenterAlignedState::DrawWhitespace(
                                            n - 1,
                                            EmptySpaceIterator::new(
                                                width,
                                                pos,
                                                self.style.text_style,
                                            ),
                                        )
                                    } else {
                                        CenterAlignedState::NextWord(first_word)
                                    }
                                } else {
                                    // don't render
                                }
                            }

                            Token::NewLine => {
                                self.state = CenterAlignedState::LineBreak("".chars());
                            }
                        }
                    } else {
                        break None;
                    }
                }

                CenterAlignedState::DrawWord(ref mut chars_iterator) => {
                    let mut copy = chars_iterator.clone();
                    self.state = if let Some(c) = copy.next() {
                        let current_pos = self.cursor.position;

                        if self.cursor.advance_char(c) {
                            CenterAlignedState::DrawCharacter(
                                copy,
                                StyledCharacterIterator::new(c, current_pos, self.style.text_style),
                            )
                        } else {
                            // word wrapping
                            CenterAlignedState::LineBreak(chars_iterator.clone())
                        }
                    } else {
                        CenterAlignedState::NextWord(false)
                    }
                }

                CenterAlignedState::DrawWhitespace(n, ref mut iterator) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    self.state = if n == 0 {
                        // no more spaces to draw
                        CenterAlignedState::NextWord(false)
                    } else {
                        let width = F::total_char_width(' ');

                        let pos = self.cursor.position;
                        if self.cursor.advance(width) {
                            // draw next space
                            CenterAlignedState::DrawWhitespace(
                                n - 1,
                                EmptySpaceIterator::new(width, pos, self.style.text_style),
                            )
                        } else {
                            // word wrapping, also applied for whitespace sequences
                            // eat the spaces from the start of next line
                            CenterAlignedState::LineBreak("".chars())
                        }
                    }
                }

                CenterAlignedState::DrawCharacter(ref chars_iterator, ref mut iterator) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    self.state = CenterAlignedState::DrawWord(chars_iterator.clone());
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

        TextBox::new("word", Rectangle::new(Point::zero(), Point::new(54, 54)))
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
    fn simple_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
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
            Rectangle::new(Point::zero(), Point::new(54, 54)),
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
