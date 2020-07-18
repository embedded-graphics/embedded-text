use crate::{alignment::TextAlignment, rendering::StyledFramedTextIterator};
use embedded_graphics::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct RightAligned;
impl TextAlignment for RightAligned {}

impl<C, F> Iterator for StyledFramedTextIterator<'_, C, F, RightAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
