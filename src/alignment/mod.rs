//! Text alignment options.
use crate::{
    rendering::{cursor::Cursor, space_config::SpaceConfig},
    style::{color::Rgb, height_mode::HeightMode, LineMeasurement},
    TextBox,
};
use embedded_graphics::text::renderer::{CharacterStyle, TextRenderer};

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

    /// Whether to move cursor via leading spaces in the first line.
    const IGNORE_LEADING_SPACES: bool = true;

    /// Calculate offset from the left side and whitespace information.
    fn place_line(
        line: &str,
        renderer: &impl TextRenderer,
        measurement: LineMeasurement,
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
    fn apply_vertical_alignment<'a, F, A, H>(
        cursor: &mut Cursor,
        styled_text_box: &'a TextBox<'a, F, A, Self, H>,
    ) where
        F: TextRenderer + CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb>,
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
