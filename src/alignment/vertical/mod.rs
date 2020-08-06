//! Vertical text alignment options.

pub mod bottom;
pub mod center;
pub mod top;

/// Vertical text alignment base trait.
///
/// Use implementors to parametrize [`TextBoxStyle`] and [`TextBoxStyleBuilder`].
///
/// [`TextBoxStyle`]: ../style/struct.TextBoxStyle.html
/// [`TextBoxStyleBuilder`]: ../style/builder/struct.TextBoxStyleBuilder.html
pub trait VerticalTextAlignment {}
