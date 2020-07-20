use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{
        EmptySpaceIterator, StateFactory, StyledCharacterIterator, StyledFramedTextIterator,
    },
    style::StyledTextBox,
    utils::{font_ext::FontExt, rect_ext::RectExt},
};
use embedded_graphics::prelude::*;

use core::str::Chars;

#[derive(Debug)]
pub enum CenterAlignedState<'a, C, F>
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

impl<C, F> Default for CenterAlignedState<'_, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    fn default() -> Self {
        Self::MeasureLine("".chars())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CenterAligned;
impl TextAlignment for CenterAligned {}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, CenterAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = CenterAlignedState<'a, C, F>;
}

impl<C, F> Iterator for StyledFramedTextIterator<'_, C, F, CenterAligned>
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

            match &mut self.state {
                CenterAlignedState::LineBreak(ref remaining) => {
                    self.char_pos = Point::new(
                        self.bounds.top_left.x,
                        self.char_pos.y + F::CHARACTER_SIZE.height as i32,
                    );
                    self.state = CenterAlignedState::MeasureLine(remaining.clone());
                }

                CenterAlignedState::MeasureLine(ref remaining) => {
                    let max_line_width = RectExt::size(self.bounds).width;

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
                                    let line_width =
                                        total_width + word_width + last_whitespace_width;
                                    if line_width <= max_line_width {
                                        total_width = line_width;
                                        last_whitespace_width = 0;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }

                        // advance pos.x
                        self.char_pos.x += (max_line_width - total_width) as i32 / 2;
                    }

                    self.state = CenterAlignedState::DrawWord(remaining.clone());
                }

                CenterAlignedState::NextWord => {
                    if let Some(token) = self.parser.next() {
                        match token {
                            Token::Word(w) => {
                                // measure w to see if it fits in current line
                                let width = w.chars().map(F::char_width).sum::<u32>();
                                if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                                    self.state = CenterAlignedState::LineBreak(w.chars());
                                } else {
                                    self.state = CenterAlignedState::DrawWord(w.chars());
                                }
                            }
                            Token::Whitespace(n) => {
                                // TODO character spacing!
                                // word wrapping, also applied for whitespace sequences
                                let width = F::char_width(' ');
                                let mut lookahead = self.parser.clone();
                                if let Some(Token::Word(w)) = lookahead.next() {
                                    // only render whitespace if next is word and next doesn't wrap
                                    let n_width = w.chars().map(F::char_width).sum::<u32>();

                                    if self.char_pos.x
                                        > self.bounds.bottom_right.x - n_width as i32 - width as i32
                                            + 1
                                    {
                                        self.state = CenterAlignedState::NextWord;
                                    } else if n != 0 {
                                        self.state = CenterAlignedState::DrawWhitespace(
                                            n - 1,
                                            EmptySpaceIterator::new(
                                                self.char_pos,
                                                width,
                                                self.style.text_style,
                                            ),
                                        );
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
                    if let Some(c) = copy.next() {
                        // TODO character spacing!
                        // word wrapping

                        let width = F::char_width(c);
                        if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                            self.state = CenterAlignedState::LineBreak(chars_iterator.clone())
                        } else {
                            self.state = CenterAlignedState::DrawCharacter(
                                copy,
                                StyledCharacterIterator::new(
                                    c,
                                    self.char_pos,
                                    self.style.text_style,
                                ),
                            );
                        }
                    } else {
                        self.state = CenterAlignedState::NextWord;
                    }
                }

                CenterAlignedState::DrawWhitespace(n, ref mut iterator) => {
                    let pixel = iterator.next();
                    if pixel.is_some() {
                        break pixel;
                    }

                    let width = F::char_width(' ');
                    self.char_pos.x += width as i32;
                    if *n == 0 {
                        self.state = CenterAlignedState::NextWord;
                    } else {
                        // word wrapping, also applied for whitespace sequences
                        if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                            self.state = CenterAlignedState::LineBreak("".chars());
                        } else {
                            self.state = CenterAlignedState::DrawWhitespace(
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

                CenterAlignedState::DrawCharacter(chars_iterator, ref mut iterator) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    self.char_pos.x += F::char_width(iterator.character) as i32;
                    self.state = CenterAlignedState::DrawWord(chars_iterator.clone());
                }
            }
        }
    }
}
