//! Space rendering config

use embedded_graphics::text::renderer::TextRenderer;

use crate::utils::str_width;

#[derive(Copy, Clone, Debug)]
pub struct SpaceConfig {
    /// The width of the whitespace characters.
    width: u32,

    /// Stores how many characters are rendered using the `width` width. This field changes
    /// during rendering.
    count: Option<u32>,
}

/// Retrieves size of space characters.
impl SpaceConfig {
    /// Creates a new SpaceConfig object.
    pub const fn new(width: u32, count: Option<u32>) -> Self {
        Self { width, count }
    }

    pub fn new_from_renderer(renderer: &impl TextRenderer) -> Self {
        let width = str_width(renderer, " ");
        Self::new(width, None)
    }

    /// Look at the size of next n spaces, without advancing.
    pub fn peek_next_width(&self, n: u32) -> u32 {
        match self.count {
            None => n * self.width,
            Some(count) => n * self.width + count.min(n),
        }
    }

    /// Advance the internal state
    pub fn consume(&mut self, n: u32) -> u32 {
        let w = self.peek_next_width(n);

        if let Some(count) = self.count.as_mut() {
            *count = count.saturating_sub(n);
        }

        w
    }
}
