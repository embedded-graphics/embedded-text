use crate::{alignment::TextAlignment, rendering::StyledFramedTextIterator};
use embedded_graphics::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum CenterAlignedState {
    StartNewLine,
}

impl Default for CenterAlignedState {
    fn default() -> Self {
        Self::StartNewLine
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CenterAligned;
impl TextAlignment for CenterAligned {
    type IteratorState = CenterAlignedState;
}

impl<C, F> Iterator for StyledFramedTextIterator<'_, C, F, CenterAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
