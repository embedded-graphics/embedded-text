//! Horizontal overdraw options.
//!
//! These options tell `embedded-text` what to do when a single word is wider than the width of
//! the `TextBox`.

/// Implementors of this trait specify how drawing vertically outside the bounding box is handled.
pub trait HorizontalOverdraw: Copy {}

/// Render as many characters as possible, render remaining characters in next line.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wrap;
impl HorizontalOverdraw for Wrap {}

/// Render as much of the word as possible.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hidden;
impl HorizontalOverdraw for Hidden {}

/// Display text even if it's outside the bounding box.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Visible;
impl HorizontalOverdraw for Visible {}
