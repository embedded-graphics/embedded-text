use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{
        EmptySpaceIterator, StateFactory, StyledCharacterIterator, StyledFramedTextIterator,
    },
    style::StyledTextBox,
};
use embedded_graphics::prelude::*;

use core::str::Chars;

#[derive(Clone)]
pub enum LeftAlignedState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    NewWord,
    DrawWord(Chars<'a>),
    DrawCharacter(Chars<'a>, StyledCharacterIterator<C, F>),
    DrawWhitespace(u32, EmptySpaceIterator<C, F>),
    Invalid,
}

impl<C, F> Default for LeftAlignedState<'_, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    fn default() -> Self {
        Self::NewWord
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

impl<C, F> Iterator for StyledFramedTextIterator<'_, C, F, LeftAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.char_pos.y > self.bounds.bottom_right.y {
                break None;
            }

            match self.state.clone() {
                LeftAlignedState::NewWord => {
                    if let Some(token) = self.parser.next() {
                        match token {
                            Token::Word(w) => {
                                // measure w to see if it fits in current line
                                // TODO assume fits for now
                                // if not, wrap
                                // update state
                                self.state = LeftAlignedState::DrawWord(w.chars());
                            }
                            Token::Whitespace(n) => {
                                // TODO character spacing!
                                // word wrapping, also applied for whitespace sequences
                                let width = F::char_width(' ');
                                if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                                    self.char_pos.x = self.bounds.top_left.x;
                                    self.char_pos.y += F::CHARACTER_SIZE.height as i32;
                                }

                                self.state = LeftAlignedState::DrawWhitespace(
                                    n,
                                    EmptySpaceIterator::new(
                                        self.char_pos,
                                        width,
                                        self.style.text_style,
                                    ),
                                );
                            }

                            Token::NewLine => {
                                self.char_pos = Point::new(
                                    self.bounds.top_left.x,
                                    self.char_pos.y + F::CHARACTER_SIZE.height as i32,
                                );
                            }
                        }
                    } else {
                        break None;
                    }
                }

                LeftAlignedState::DrawWord(mut chars_iterator) => {
                    if let Some(c) = chars_iterator.next() {
                        // TODO character spacing!
                        // word wrapping
                        let width = F::char_width(c);
                        if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                            self.char_pos.x = self.bounds.top_left.x;
                            self.char_pos.y += F::CHARACTER_SIZE.height as i32;
                        }

                        self.state = LeftAlignedState::DrawCharacter(
                            chars_iterator,
                            StyledCharacterIterator::new(c, self.char_pos, self.style.text_style),
                        );
                    } else {
                        self.state = LeftAlignedState::NewWord;
                    }
                }

                LeftAlignedState::DrawWhitespace(n, ref mut iterator) => {
                    let pixel = iterator.next();
                    if pixel.is_some() {
                        break pixel;
                    }

                    let width = F::char_width(' ');
                    if n == 0 {
                        self.char_pos.x += width as i32;
                        self.state = LeftAlignedState::NewWord;
                    } else {
                        // word wrapping, also applied for whitespace sequences
                        if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                            self.char_pos.x = self.bounds.top_left.x;
                            self.char_pos.y += F::CHARACTER_SIZE.height as i32;
                        }

                        self.state = LeftAlignedState::DrawWhitespace(
                            n - 1,
                            EmptySpaceIterator::new(self.char_pos, width, self.style.text_style),
                        );
                    }
                }

                LeftAlignedState::DrawCharacter(chars_iterator, mut iterator) => {
                    let pixel = iterator.next();
                    if pixel.is_some() {
                        self.state = LeftAlignedState::DrawCharacter(chars_iterator, iterator);
                        break pixel;
                    } else {
                        self.char_pos.x += F::char_width(iterator.character) as i32;
                        self.state = LeftAlignedState::DrawWord(chars_iterator);
                    }
                }

                _ => panic!("Invalid state reached"),
            };
        }
    }
}
