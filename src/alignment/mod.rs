//! Text alignment options.
use crate::{rendering::cursor::Cursor, style::height_mode::HeightMode, StyledTextBox};
use embedded_graphics::prelude::*;

pub mod bottom;
pub mod center;
pub mod justified;
pub mod left;
pub mod right;
pub mod top;

/// Horizontal text alignment base trait.
///
/// Use implementors to parametrize [`TextBoxStyle`] and [`TextBoxStyleBuilder`].
///
/// [`TextBoxStyle`]: ../style/struct.TextBoxStyle.html
/// [`TextBoxStyleBuilder`]: ../style/builder/struct.TextBoxStyleBuilder.html
pub trait HorizontalTextAlignment: Copy {
    /// Whether or not render spaces in the start of the line.
    const STARTING_SPACES: bool;

    /// Whether or not render spaces in the end of the line.
    const ENDING_SPACES: bool;
}

/// Vertical text alignment base trait.
///
/// Use implementors to parametrize [`TextBoxStyle`] and [`TextBoxStyleBuilder`].
///
/// [`TextBoxStyle`]: ../style/struct.TextBoxStyle.html
/// [`TextBoxStyleBuilder`]: ../style/builder/struct.TextBoxStyleBuilder.html
pub trait VerticalTextAlignment: Copy {
    /// Set the cursor's initial vertical position
    fn apply_vertical_alignment<'a, C, F, A, H>(
        cursor: &mut Cursor<F>,
        styled_text_box: &'a StyledTextBox<'a, C, F, A, Self, H>,
    ) where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        H: HeightMode;
}

pub use bottom::BottomAligned;
pub use center::CenterAligned;
pub use justified::Justified;
pub use left::LeftAligned;
pub use right::RightAligned;
pub use top::TopAligned;
