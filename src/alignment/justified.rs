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

#[derive(Copy, Clone, Debug)]
pub struct SpaceInfo {
    pub space_width: u32,
    pub space_count: u32,
    pub remaining_space_width: u32,
}

impl SpaceInfo {
    fn default<F: Font>() -> Self {
        SpaceInfo::new(F::char_width(' '), 0)
    }

    fn new(space_width: u32, extra_pixel_count: u32) -> Self {
        SpaceInfo {
            space_width: space_width + 1,
            space_count: extra_pixel_count,
            remaining_space_width: space_width,
        }
    }

    fn space_width(&mut self) -> u32 {
        if self.space_count == 0 {
            self.remaining_space_width
        } else {
            self.space_count -= 1;
            self.space_width
        }
    }

    fn peek_space_width(&self, whitespace_count: u32) -> u32 {
        let above_limit = whitespace_count.saturating_sub(self.space_count);
        self.space_width * self.space_count + above_limit * self.remaining_space_width
    }
}

#[derive(Debug)]
pub enum JustifiedState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    NextWord(SpaceInfo),
    LineBreak(Chars<'a>),
    MeasureLine(Chars<'a>),
    DrawWord(Chars<'a>, SpaceInfo),
    DrawCharacter(Chars<'a>, StyledCharacterIterator<C, F>, SpaceInfo),
    DrawWhitespace(u32, EmptySpaceIterator<C, F>, SpaceInfo),
}

impl<C, F> Default for JustifiedState<'_, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    fn default() -> Self {
        Self::MeasureLine("".chars())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Justified;
impl TextAlignment for Justified {}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, Justified>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = JustifiedState<'a, C, F>;
}

impl<C, F> Iterator for StyledFramedTextIterator<'_, C, F, Justified>
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
                JustifiedState::LineBreak(ref remaining) => {
                    self.char_pos = Point::new(
                        self.bounds.top_left.x,
                        self.char_pos.y + F::CHARACTER_SIZE.height as i32,
                    );
                    self.state = JustifiedState::MeasureLine(remaining.clone());
                }

                JustifiedState::MeasureLine(ref remaining) => {
                    let max_line_width = RectExt::size(self.bounds).width;

                    // initial width is the width of the characters carried over to this row
                    let (mut total_width, fits) = F::max_fitting(remaining.clone(), max_line_width);

                    let mut total_whitespace_count = 0;
                    let mut stretch_line = true;

                    // in some rare cases, the carried over text may not fit into a single line
                    if fits {
                        let mut last_whitespace_width = 0;
                        let mut last_whitespace_count = 0;
                        let mut total_whitespace_width = 0;

                        for token in self.parser.clone() {
                            if total_width >= max_line_width {
                                break;
                            }
                            match token {
                                Token::NewLine => {
                                    stretch_line = false;
                                    break;
                                }

                                Token::Whitespace(_) if total_width == 0 => {
                                    // eat spaces at the start of line
                                }

                                Token::Whitespace(n) => {
                                    last_whitespace_count = n;
                                    last_whitespace_width =
                                        (n * F::char_width(' ')).min(max_line_width - total_width);
                                }

                                Token::Word(w) => {
                                    let word_width = w.chars().map(F::char_width).sum::<u32>();
                                    let new_total_width = total_width + word_width;
                                    let new_whitespace_width =
                                        total_whitespace_width + last_whitespace_width;
                                    if new_whitespace_width + new_total_width <= max_line_width {
                                        total_width = new_total_width;
                                        total_whitespace_width = new_whitespace_width;
                                        total_whitespace_count += last_whitespace_count;

                                        last_whitespace_count = 0;
                                        last_whitespace_width = 0;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    let chars = remaining.clone();
                    if stretch_line && total_whitespace_count != 0 {
                        let total_space_width = max_line_width - total_width;
                        let space_width =
                            (total_space_width / total_whitespace_count).max(F::char_width(' '));
                        let extra_pixels = total_space_width - space_width * total_whitespace_count;

                        self.state = JustifiedState::DrawWord(
                            chars,
                            SpaceInfo::new(space_width, extra_pixels),
                        );
                    } else {
                        self.state = JustifiedState::DrawWord(chars, SpaceInfo::default::<F>());
                    }
                }

                JustifiedState::NextWord(space_info) => {
                    if let Some(token) = self.parser.next() {
                        match token {
                            Token::Word(w) => {
                                // measure w to see if it fits in current line
                                let width = w.chars().map(F::char_width).sum::<u32>();
                                if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                                    self.state = JustifiedState::LineBreak(w.chars());
                                } else {
                                    self.state = JustifiedState::DrawWord(w.chars(), *space_info);
                                }
                            }
                            Token::Whitespace(n) => {
                                // TODO character spacing!
                                // word wrapping, also applied for whitespace sequences
                                let width = space_info.peek_space_width(n);
                                let mut lookahead = self.parser.clone();
                                if let Some(Token::Word(w)) = lookahead.next() {
                                    // only render whitespace if next is word and next doesn't wrap
                                    let n_width = w.chars().map(F::char_width).sum::<u32>();

                                    if self.char_pos.x
                                        > self.bounds.bottom_right.x - n_width as i32 - width as i32
                                            + 1
                                    {
                                        self.state = JustifiedState::NextWord(*space_info);
                                    } else if n != 0 {
                                        self.state = JustifiedState::DrawWhitespace(
                                            n - 1,
                                            EmptySpaceIterator::new(
                                                self.char_pos,
                                                width,
                                                self.style.text_style,
                                            ),
                                            *space_info,
                                        );
                                    }
                                } else {
                                    // don't render
                                }
                            }

                            Token::NewLine => {
                                self.state = JustifiedState::LineBreak("".chars());
                            }
                        }
                    } else {
                        break None;
                    }
                }

                JustifiedState::DrawWord(ref mut chars_iterator, space_info) => {
                    let mut copy = chars_iterator.clone();
                    self.state = if let Some(c) = copy.next() {
                        // TODO character spacing!
                        let width = F::char_width(c);

                        if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                            // word wrapping
                            JustifiedState::LineBreak(chars_iterator.clone())
                        } else {
                            JustifiedState::DrawCharacter(
                                copy,
                                StyledCharacterIterator::new(
                                    c,
                                    self.char_pos,
                                    self.style.text_style,
                                ),
                                *space_info,
                            )
                        }
                    } else {
                        JustifiedState::NextWord(*space_info)
                    }
                }

                JustifiedState::DrawWhitespace(n, ref mut iterator, space_info) => {
                    let pixel = iterator.next();
                    if pixel.is_some() {
                        break pixel;
                    }

                    let width = space_info.space_width();
                    self.char_pos.x += width as i32;
                    if *n == 0 {
                        self.state = JustifiedState::NextWord(*space_info);
                    } else {
                        // word wrapping, also applied for whitespace sequences
                        if self.char_pos.x > self.bounds.bottom_right.x - width as i32 + 1 {
                            self.state = JustifiedState::LineBreak("".chars());
                        } else {
                            self.state = JustifiedState::DrawWhitespace(
                                *n - 1,
                                EmptySpaceIterator::new(
                                    self.char_pos,
                                    width,
                                    self.style.text_style,
                                ),
                                *space_info,
                            );
                        }
                    }
                }

                JustifiedState::DrawCharacter(chars_iterator, ref mut iterator, space_info) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    self.char_pos.x += F::char_width(iterator.character) as i32;
                    self.state = JustifiedState::DrawWord(chars_iterator.clone(), *space_info);
                }
            }
        }
    }
}
