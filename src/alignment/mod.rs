//! Horizontal text alignment opitons.

pub mod center;
pub mod justified;
pub mod left;
pub mod right;

/// Text alignment base trait.
///
/// Use implementors to parametrize [`TextBoxStyle`] and [`TextBoxStyleBuilder`].
///
/// [`TextBoxStyle`]: ../style/struct.TextBoxStyle.html
/// [`TextBoxStyleBuilder`]: ../style/builder/struct.TextBoxStyleBuilder.html
pub trait TextAlignment: Copy {
    /// Whether or not render spaces in the start of the line.
    const STARTING_SPACES: bool;

    /// Whether or not render spaces in the end of the line.
    const ENDING_SPACES: bool;
}

pub use center::CenterAligned;
pub use justified::Justified;
pub use left::LeftAligned;
pub use right::RightAligned;
