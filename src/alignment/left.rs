//! Left aligned text
use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{
        character::StyledCharacterIterator, whitespace::EmptySpaceIterator, StateFactory,
        StyledTextBoxIterator,
    },
    style::StyledTextBox,
    utils::font_ext::FontExt,
};
use embedded_graphics::prelude::*;

use core::str::Chars;

/// Marks text to be rendered left aligned
#[derive(Copy, Clone, Debug)]
pub struct LeftAligned;
impl TextAlignment for LeftAligned {}

/// State variable used by the left aligned text renderer
#[derive(Debug)]
pub enum LeftAlignedState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// This state processes the next token in the text.
    NextWord,

    /// This state processes the next character in a word.
    DrawWord(Chars<'a>),

    /// This state renders a character, then passes the rest of the character iterator to DrawWord.
    DrawCharacter(Chars<'a>, StyledCharacterIterator<C, F>),

    /// This state renders whitespace.
    DrawWhitespace(u32, EmptySpaceIterator<C, F>),
}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, LeftAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = LeftAlignedState<'a, C, F>;

    #[inline]
    #[must_use]
    fn create_state() -> Self::PixelIteratorState {
        LeftAlignedState::NextWord
    }
}

impl<C, F> Iterator for StyledTextBoxIterator<'_, C, F, LeftAligned>
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
                LeftAlignedState::NextWord => {
                    if let Some(token) = self.parser.next() {
                        match token {
                            Token::Word(w) => {
                                // measure w to see if it fits in current line
                                if !self.cursor.is_start_of_line()
                                    && !self.cursor.fits_in_line(
                                        w.chars().map(F::total_char_width).sum::<u32>(),
                                    )
                                {
                                    self.cursor.carriage_return();
                                    self.cursor.new_line();
                                }

                                self.state = LeftAlignedState::DrawWord(w.chars());
                            }
                            Token::Whitespace(n) => {
                                // word wrapping, also applied for whitespace sequences
                                let width = F::total_char_width(' ');
                                let pos = self.cursor.position;
                                self.state = if self.cursor.advance(width) {
                                    LeftAlignedState::DrawWhitespace(
                                        n - 1,
                                        EmptySpaceIterator::new(width, pos, self.style.text_style),
                                    )
                                } else {
                                    LeftAlignedState::NextWord
                                }
                            }

                            Token::NewLine => {
                                self.cursor.carriage_return();
                                self.cursor.new_line();
                            }
                        }
                    } else {
                        break None;
                    }
                }

                LeftAlignedState::DrawWord(ref mut chars_iterator) => {
                    let mut copy = chars_iterator.clone();
                    if let Some(c) = copy.next() {
                        let current_pos = self.cursor.position;

                        if self.cursor.advance_char(c) {
                            self.state = LeftAlignedState::DrawCharacter(
                                copy,
                                StyledCharacterIterator::new(c, current_pos, self.style.text_style),
                            );
                        } else {
                            // word wrapping
                            self.cursor.carriage_return();
                            self.cursor.new_line();
                        }
                    } else {
                        self.state = LeftAlignedState::NextWord;
                    }
                }

                LeftAlignedState::DrawWhitespace(n, ref mut iterator) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    self.state = if n == 0 {
                        // no more spaces to draw
                        LeftAlignedState::NextWord
                    } else {
                        // word wrapping, also applied for whitespace sequences
                        let width = F::total_char_width(' ');

                        // use the current position, except if wrapping
                        let mut pos = self.cursor.position;
                        if !self.cursor.advance(width) {
                            self.cursor.carriage_return();
                            self.cursor.new_line();
                            pos = self.cursor.position;
                            self.cursor.advance(width);
                        }

                        LeftAlignedState::DrawWhitespace(
                            n - 1,
                            EmptySpaceIterator::new(width, pos, self.style.text_style),
                        )
                    }
                }

                LeftAlignedState::DrawCharacter(ref chars_iterator, ref mut iterator) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    self.state = LeftAlignedState::DrawWord(chars_iterator.clone());
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

        TextBox::new("word", Rectangle::new(Point::zero(), Point::new(54, 54)))
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
    fn simple_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
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
    fn whitespace_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word  wrap",
            Rectangle::new(Point::zero(), Point::new(30, 54)),
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
            Rectangle::new(Point::zero(), Point::new(54, 54)),
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
            Rectangle::new(Point::zero(), Point::new(54, 54)),
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
