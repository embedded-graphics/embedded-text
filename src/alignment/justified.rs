use crate::{alignment::TextAlignment, rendering::StyledFramedTextIterator};
use embedded_graphics::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum JustifiedState {
    StartNewLine,
}

impl Default for JustifiedState {
    fn default() -> Self {
        Self::StartNewLine
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Justified;
impl TextAlignment for Justified {
    type IteratorState = JustifiedState;
}

impl<C, F> Iterator for StyledFramedTextIterator<'_, C, F, Justified>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
