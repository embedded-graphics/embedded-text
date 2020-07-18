use crate::{rendering::StyledFramedTextIterator, StyledTextBox};
use embedded_graphics::prelude::*;

pub mod center;
pub mod justified;
pub mod left;
pub mod right;

/// Text alignment
pub trait TextAlignment: Copy {
    type IteratorState: Default;

    fn into_pixel_iterator<'a, C, F>(
        text_box: &'a StyledTextBox<'a, C, F, Self>,
    ) -> StyledFramedTextIterator<'a, C, F, Self>
    where
        C: PixelColor,
        F: Font + Copy,
    {
        StyledFramedTextIterator::new(text_box)
    }
}

pub use center::CenterAligned;
pub use justified::Justified;
pub use left::LeftAligned;
pub use right::RightAligned;
