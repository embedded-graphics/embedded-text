use crate::{
    alignment::TextAlignment,
    rendering::{StateFactory, StyledFramedTextIterator},
    style::StyledTextBox,
};
use embedded_graphics::prelude::*;

use core::str::Chars;

pub enum LeftAlignedState<'a> {
    StartNewLine,
    DrawWord(Chars<'a>),
}

impl Default for LeftAlignedState<'_> {
    fn default() -> Self {
        Self::StartNewLine
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
    type PixelIteratorState = LeftAlignedState<'a>;
}

impl<C, F> Iterator for StyledFramedTextIterator<'_, C, F, LeftAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
