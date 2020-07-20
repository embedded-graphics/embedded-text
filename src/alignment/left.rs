use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{EmptySpaceIterator, StateFactory, StyledCharacterIterator, StyledTextBoxIterator},
    style::StyledTextBox,
};
use embedded_graphics::prelude::*;

use core::str::Chars;

#[derive(Debug)]
pub enum LeftAlignedState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    NextWord,
    DrawWord(Chars<'a>),
    DrawCharacter(Chars<'a>, StyledCharacterIterator<C, F>),
    DrawWhitespace(u32, EmptySpaceIterator<C, F>),
}

impl<C, F> Default for LeftAlignedState<'_, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::NextWord
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LeftAligned;
impl TextAlignment for LeftAligned {}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, LeftAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = LeftAlignedState<'a, C, F>;
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
                                    self.cursor.new_line();
                                }

                                self.state = LeftAlignedState::DrawWord(w.chars());
                            }
                            Token::Whitespace(n) => {
                                // TODO character spacing!
                                // word wrapping, also applied for whitespace sequences
                                let width = F::char_width(' ');
                                if !self.cursor.fits_in_line(width) {
                                    self.state = LeftAlignedState::NextWord;
                                } else if n != 0 {
                                    self.state = LeftAlignedState::DrawWhitespace(
                                        n - 1,
                                        EmptySpaceIterator::new(
                                            self.cursor.position,
                                            width,
                                            self.style.text_style,
                                        ),
                                    );
                                }
                            }

                            Token::NewLine => {
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
                        let width = F::char_width(c);

                        if self.cursor.fits_in_line(width) {
                            self.state = LeftAlignedState::DrawCharacter(
                                copy,
                                StyledCharacterIterator::new(
                                    c,
                                    self.cursor.position,
                                    self.style.text_style,
                                ),
                            );
                        } else {
                            // word wrapping
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
                        if self.cursor.fits_in_line(width) {
                            self.cursor.advance(width);
                        } else {
                            // duplicate line break because LineBreak state can't handle whitespaces
                            // carried-over
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

                    self.cursor.advance_char(iterator.character);
                    self.state = LeftAlignedState::DrawWord(chars_iterator.clone());
                }
            };
        }
    }
}
