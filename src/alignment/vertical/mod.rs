//! Vertical text alignment options.
use crate::{
    alignment::horizontal::HorizontalTextAlignment, rendering::cursor::Cursor, style::StyledTextBox,
};
use embedded_graphics::prelude::*;

pub mod bottom;
pub mod center;
pub mod top;

/// Vertical text alignment base trait.
///
/// Use implementors to parametrize [`TextBoxStyle`] and [`TextBoxStyleBuilder`].
///
/// [`TextBoxStyle`]: ../../style/struct.TextBoxStyle.html
/// [`TextBoxStyleBuilder`]: ../../style/builder/struct.TextBoxStyleBuilder.html
pub trait VerticalTextAlignment: Copy {
    /// Set the cursor's initial vertical position
    fn apply_vertical_alignment<'a, C, F, A>(
        cursor: &mut Cursor<F>,
        styled_text_box: &'a StyledTextBox<'a, C, F, A, Self>,
    ) where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment;
}

pub use bottom::Bottom;
pub use center::Center;
pub use top::Top;
