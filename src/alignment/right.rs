//! Right aligned text
use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{EmptySpaceIterator, StateFactory, StyledCharacterIterator, StyledTextBoxIterator},
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
    NextWord,

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
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }

                        // advance pos.x
                        self.cursor.position.x += (max_line_width - total_width) as i32;
                    }

                    self.state = RightAlignedState::DrawWord(remaining.clone());
                }

                RightAlignedState::NextWord => {
                    if let Some(token) = self.parser.next() {
                        match token {
                            Token::Word(w) => {
                                // measure w to see if it fits in current line
                                if self
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
                                if self.cursor.fits_in_line(width) {
                                    self.state = RightAlignedState::DrawWhitespace(
                                        n - 1,
                                        EmptySpaceIterator::new(
                                            self.cursor.position,
                                            width,
                                            self.style.text_style,
                                        ),
                                    );
                                } else if n != 0 {
                                    self.state = RightAlignedState::NextWord;
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
                        let width = F::char_width(c);

                        if self.cursor.fits_in_line(width) {
                            RightAlignedState::DrawCharacter(
                                copy,
                                StyledCharacterIterator::new(
                                    c,
                                    self.cursor.position,
                                    self.style.text_style,
                                ),
                            )
                        } else {
                            // word wrapping
                            RightAlignedState::LineBreak(chars_iterator.clone())
                        }
                    } else {
                        RightAlignedState::NextWord
                    }
                }

                RightAlignedState::DrawWhitespace(n, ref mut iterator) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    self.state = if n == 0 {
                        // no more spaces to draw
                        self.cursor.advance_char(' ');
                        RightAlignedState::NextWord
                    } else {
                        let width = F::char_width(' ');
                        if self.cursor.fits_in_line(width) {
                            // draw next space
                            self.cursor.advance(width);
                            RightAlignedState::DrawWhitespace(
                                n - 1,
                                EmptySpaceIterator::new(
                                    self.cursor.position,
                                    width,
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

                    self.cursor.advance_char(iterator.character);
                    self.state = RightAlignedState::DrawWord(chars_iterator.clone());
                }
            }
        }
    }
}
