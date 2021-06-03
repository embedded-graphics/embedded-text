//! Misc utilities

use embedded_graphics::{
    prelude::Point,
    text::{renderer::TextRenderer, Baseline},
};

use crate::parser::SPEC_CHAR_NBSP;

/// Measure the width of a piece of string.
pub fn str_width(renderer: &impl TextRenderer, s: &str) -> u32 {
    let width = |s: &str| -> u32 {
        renderer
            .measure_string(s, Point::zero(), Baseline::Top)
            .next_position
            .x as u32
    };

    let nbsp_count = s.chars().filter(|c| *c == SPEC_CHAR_NBSP).count() as u32;
    width(s) - nbsp_count * (width("\u{a0}").saturating_sub(width(" ")))
}

#[cfg(test)]
pub mod test {
    use embedded_graphics::{
        mono_font::{ascii::FONT_6X9, MonoFont, MonoTextStyle},
        pixelcolor::BinaryColor,
        prelude::Size,
    };

    use super::str_width;

    pub fn size_for(font: &MonoFont, chars: u32, lines: u32) -> Size {
        font.character_size.x_axis() * chars + font.character_size.y_axis() * lines
    }

    #[test]
    fn width_of_nbsp_is_single_space() {
        let renderer = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
        assert_eq!(str_width(&renderer, " "), str_width(&renderer, "\u{a0}"));
    }
}
