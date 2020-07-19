use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{
        EmptySpaceIterator, StateFactory, StyledCharacterIterator, StyledFramedTextIterator,
    },
    style::StyledTextBox,
    utils::rect_ext::RectExt,
};
use embedded_graphics::prelude::*;

use core::str::Chars;

#[derive(Debug)]
pub enum RightAlignedState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    NextWord,
    LineBreak(Chars<'a>),
    MeasureLine(Chars<'a>),
    DrawWord(Chars<'a>),
    DrawCharacter(Chars<'a>, StyledCharacterIterator<C, F>),
    DrawWhitespace(u32, EmptySpaceIterator<C, F>),
}

impl<C, F> Default for RightAlignedState<'_, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    fn default() -> Self {
        Self::MeasureLine("".chars())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RightAligned;
impl TextAlignment for RightAligned {}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, RightAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = RightAlignedState<'a, C, F>;
}

impl<C, F> Iterator for StyledFramedTextIterator<'_, C, F, RightAligned>
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

            let max_line_width = RectExt::size(self.bounds).width;
            match &mut self.state {
                RightAlignedState::LineBreak(ref remaining) => {
                    self.char_pos = Point::new(
                        self.bounds.top_left.x,
                        self.char_pos.y + F::CHARACTER_SIZE.height as i32,
                    );
                    self.state = RightAlignedState::MeasureLine(remaining.clone());
                }

                RightAlignedState::MeasureLine(ref remaining) => {
                    // measure row
                    let copy = remaining.clone();

                    let mut total_width = 0;

                    for c in copy {
                        let width = F::char_width(c);
                        if total_width + width < max_line_width {
                            total_width += width;
                        } else {
                            break;
                        }
                    }

                    let has_remaining = total_width > 0;
                    let mut last_whitespace_width = 0;

                    let mut parser = self.parser.clone();
                    while let Some(token) = parser.next() {
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
                    self.char_pos.x += (max_line_width - total_width) as i32;

                    if has_remaining {
                        self.state = RightAlignedState::DrawWord(remaining.clone());
                    } else {
                        self.state = RightAlignedState::NextWord;
                    }
                }

                RightAlignedState::NextWord => {
                    if let Some(token) = self.parser.next() {
                        match token {
                            Token::Word(w) => {
                                // measure w to see if it fits in current line
                                let width = w.chars().map(F::char_width).sum::<u32>();
                                if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                                    self.state = RightAlignedState::LineBreak(w.chars());
                                } else {
                                    self.state = RightAlignedState::DrawWord(w.chars());
                                }
                            }
                            Token::Whitespace(n) => {
                                // TODO character spacing!
                                // word wrapping, also applied for whitespace sequences
                                let width = F::char_width(' ');
                                if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                                    self.state = RightAlignedState::NextWord;
                                } else if n != 0 {
                                    self.state = RightAlignedState::DrawWhitespace(
                                        n - 1,
                                        EmptySpaceIterator::new(
                                            self.char_pos,
                                            width,
                                            self.style.text_style,
                                        ),
                                    );
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
                    if let Some(c) = copy.next() {
                        // TODO character spacing!
                        // word wrapping

                        let width = F::char_width(c);
                        if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                            self.state = RightAlignedState::LineBreak(chars_iterator.clone())
                        } else {
                            self.state = RightAlignedState::DrawCharacter(
                                copy,
                                StyledCharacterIterator::new(
                                    c,
                                    self.char_pos,
                                    self.style.text_style,
                                ),
                            );
                        }
                    } else {
                        self.state = RightAlignedState::NextWord;
                    }
                }

                RightAlignedState::DrawWhitespace(n, ref mut iterator) => {
                    let pixel = iterator.next();
                    if pixel.is_some() {
                        break pixel;
                    }

                    let width = F::char_width(' ');
                    self.char_pos.x += width as i32;
                    if *n == 0 {
                        self.state = RightAlignedState::NextWord;
                    } else {
                        // word wrapping, also applied for whitespace sequences
                        if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                            self.state = RightAlignedState::LineBreak("".chars());
                        } else {
                            self.state = RightAlignedState::DrawWhitespace(
                                *n - 1,
                                EmptySpaceIterator::new(
                                    self.char_pos,
                                    width,
                                    self.style.text_style,
                                ),
                            );
                        }
                    }
                }

                RightAlignedState::DrawCharacter(chars_iterator, ref mut iterator) => {
                    let pixel = iterator.next();
                    if pixel.is_some() {
                        break pixel;
                    }

                    self.char_pos.x += F::char_width(iterator.character) as i32;
                    self.state = RightAlignedState::DrawWord(chars_iterator.clone());
                }
            }
        }
    }
}
