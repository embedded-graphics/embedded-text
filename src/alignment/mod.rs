//! Horizontal text alignment opitons.

pub mod center;
pub mod justified;
pub mod left;
pub mod right;

/// Text alignment base trait.
pub trait TextAlignment: Copy {}

pub use center::CenterAligned;
pub use justified::Justified;
pub use left::LeftAligned;
pub use right::RightAligned;
