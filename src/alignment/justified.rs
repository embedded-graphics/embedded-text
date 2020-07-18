use crate::{
    alignment::TextAlignment,
    rendering::{StateFactory, StyledFramedTextIterator},
    style::StyledTextBox,
};
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
impl TextAlignment for Justified {}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, Justified>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = JustifiedState;
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
