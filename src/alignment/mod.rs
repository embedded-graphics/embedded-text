//! Text alignment options.
use crate::{
    parser::Token,
    rendering::{cursor::Cursor, space_config::SpaceConfig},
    style::height_mode::HeightMode,
    StyledTextBox,
};
use embedded_graphics::prelude::*;

pub mod bottom;
pub mod center;
pub mod justified;
pub mod left;
pub mod right;
pub mod scrolling;
pub mod top;

/// Horizontal text alignment base trait.
///
/// Use implementors to parametrize [`TextBoxStyle`] and [`TextBoxStyleBuilder`].
///
/// [`TextBoxStyle`]: ../style/struct.TextBoxStyle.html
/// [`TextBoxStyleBuilder`]: ../style/builder/struct.TextBoxStyleBuilder.html
pub trait HorizontalTextAlignment: Copy {
    /// Type of the associated whitespace information.
    type SpaceConfig: SpaceConfig;

    /// Whether or not render spaces in the start of the line.
    const STARTING_SPACES: bool;

    /// Whether or not render spaces in the end of the line.
    const ENDING_SPACES: bool;

    /// Calculate offset from the left side and whitespace information.
    fn place_line<F: MonoFont>(
        max_width: u32,
        text_width: u32,
        n_spaces: u32,
        carried_token: Option<Token>,
    ) -> (u32, Self::SpaceConfig);
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
        F: MonoFont,
        A: HorizontalTextAlignment,
        H: HeightMode;
}

pub use bottom::BottomAligned;
pub use center::CenterAligned;
pub use justified::Justified;
pub use left::LeftAligned;
pub use right::RightAligned;
pub use scrolling::Scrolling;
pub use top::TopAligned;
