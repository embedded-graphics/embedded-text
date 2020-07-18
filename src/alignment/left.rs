use crate::{alignment::TextAlignment, rendering::StyledFramedTextIterator};
use embedded_graphics::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct LeftAligned;
impl TextAlignment for LeftAligned {}

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
