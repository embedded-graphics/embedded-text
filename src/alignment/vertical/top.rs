//! Top vertical text alignment.
use crate::{
    alignment::horizontal::HorizontalTextAlignment, rendering::cursor::Cursor, style::StyledTextBox,
};
use embedded_graphics::prelude::*;

use super::VerticalTextAlignment;

/// Align text to the top of the TextBox.
#[derive(Copy, Clone)]
pub struct Top;

impl VerticalTextAlignment for Top {
    #[inline]
    fn apply_vertical_alignment<'a, C, F, A>(
        _cursor: &mut Cursor<F>,
        _styled_text_box: &'a StyledTextBox<'a, C, F, A, Self>,
    ) where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
    {
        // nothing to do here
    }
}
