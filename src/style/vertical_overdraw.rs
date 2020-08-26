//!

/// Implementors of this trait specify how drawing vertically outside the bounding box is handled.
pub trait VerticalOverdraw {}

/// Only render full rows of text.
pub struct FullRowsOnly;
impl VerticalOverdraw for FullRowsOnly {}

/// Render partially visible rows, but only inside the bounding box.
pub struct Hidden;
impl VerticalOverdraw for Hidden {}

/// Display text even if it's outside the bounding box.
pub struct Visible;
impl VerticalOverdraw for Visible {}
