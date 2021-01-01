//! Misc utilities

use embedded_graphics::{prelude::Point, text::TextRenderer};

/// Measure the width of a piece of string.
pub fn str_width(renderer: &impl TextRenderer, s: &str) -> u32 {
    renderer.measure_string(s, Point::zero()).next_position.x as u32
}

#[cfg(test)]
pub mod test {
    use embedded_graphics::{mono_font::MonoFont, prelude::Size};

    pub fn size_for<F: MonoFont>(_: F, chars: u32, lines: u32) -> Size {
        F::CHARACTER_SIZE.x_axis() * chars + F::CHARACTER_SIZE.y_axis() * lines
    }
}
