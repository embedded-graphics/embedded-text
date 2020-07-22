//! Left aligned text
use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{EmptySpaceIterator, StateFactory, StyledCharacterIterator, StyledTextBoxIterator},
    style::StyledTextBox,
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
                                if !self
                                    .cursor
                                    .fits_in_line(w.chars().map(F::char_width).sum::<u32>())
                                {
                                    self.cursor.carriage_return();
                                    self.cursor.new_line();
                                }

                                self.state = LeftAlignedState::DrawWord(w.chars());
                            }
                            Token::Whitespace(n) => {
                                // TODO character spacing!
                                // word wrapping, also applied for whitespace sequences
                                let width = F::char_width(' ');
                                if self.cursor.fits_in_line(width) {
                                    self.state = LeftAlignedState::DrawWhitespace(
                                        n - 1,
                                        EmptySpaceIterator::new(
                                            self.cursor.position,
                                            width,
                                            self.style.text_style,
                                        ),
                                    );
                                } else {
                                    self.state = LeftAlignedState::NextWord;
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
                        // TODO character spacing!
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
                        self.cursor.advance_char(' ');
                        LeftAlignedState::NextWord
                    } else {
                        // word wrapping, also applied for whitespace sequences
                        let width = F::char_width(' ');
                        if !self.cursor.advance(width) {
                            self.cursor.carriage_return();
                            self.cursor.new_line();
                        }

                        LeftAlignedState::DrawWhitespace(
                            n - 1,
                            EmptySpaceIterator::new(
                                self.cursor.position,
                                width,
                                self.style.text_style,
                            ),
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
