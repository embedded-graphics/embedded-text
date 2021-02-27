//! Misc utilities

use embedded_graphics::{prelude::Point, text::TextRenderer};

use crate::parser::SPEC_CHAR_NBSP;

/// Measure the width of a piece of string.
pub fn str_width(renderer: &impl TextRenderer, s: &str) -> u32 {
    let width =
        |s: &str| -> u32 { renderer.measure_string(s, Point::zero()).next_position.x as u32 };

    let nbsp_count = s.chars().filter(|c| *c == SPEC_CHAR_NBSP).count() as u32;
    width(s) - nbsp_count * (width("\u{a0}").saturating_sub(width(" ")))
}

#[cfg(test)]
pub mod test {
    use embedded_graphics::{
        mono_font::{ascii::Font6x9, MonoFont, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        prelude::Size,
    };

    use super::str_width;

    pub fn size_for<F: MonoFont>(_: F, chars: u32, lines: u32) -> Size {
        F::CHARACTER_SIZE.x_axis() * chars + F::CHARACTER_SIZE.y_axis() * lines
    }

    #[test]
    fn width_of_nbsp_is_single_space() {
        let renderer = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();
        assert_eq!(str_width(&renderer, " "), str_width(&renderer, "\u{a0}"));
    }
}
