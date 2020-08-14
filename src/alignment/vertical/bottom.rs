//! Bottom vertical text alignment.
use crate::{
    alignment::horizontal::HorizontalTextAlignment, rendering::cursor::Cursor, style::StyledTextBox,
};
use embedded_graphics::prelude::*;

use super::VerticalTextAlignment;

/// Align text to the bottom of the TextBox.
#[derive(Copy, Clone)]
pub struct Bottom;

impl VerticalTextAlignment for Bottom {
    #[inline]
    fn apply_vertical_alignment<'a, C, F, A>(
        cursor: &mut Cursor<F>,
        styled_text_box: &'a StyledTextBox<'a, C, F, A, Self>,
    ) where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
    {
        let text_height = styled_text_box
            .style
            .measure_text_height(styled_text_box.text_box.text, cursor.line_width());

        let box_height = styled_text_box.size().height;
        let offset = box_height - text_height;

        cursor.position.y += offset as i32
    }
}
