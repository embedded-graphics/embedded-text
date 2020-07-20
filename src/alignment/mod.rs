/// Center aligned text rendering
pub mod center;

/// Fully justified text rendering
pub mod justified;

/// Left aligned text rendering
pub mod left;

/// Right aligned text rendering
pub mod right;

/// Text alignment
pub trait TextAlignment: Copy {}

pub use center::CenterAligned;
pub use justified::Justified;
pub use left::LeftAligned;
pub use right::RightAligned;
