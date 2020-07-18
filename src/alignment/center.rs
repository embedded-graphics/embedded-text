use crate::{
    alignment::TextAlignment,
    rendering::{StateFactory, StyledFramedTextIterator},
    style::StyledTextBox,
};
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
impl TextAlignment for CenterAligned {}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, CenterAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = CenterAlignedState;
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
