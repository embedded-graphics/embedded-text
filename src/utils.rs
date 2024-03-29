//! Misc utilities

use embedded_graphics::{
    prelude::Point,
    text::{renderer::TextRenderer, Baseline},
};

/// Measure the width of a piece of string.
pub fn str_width(renderer: &impl TextRenderer, s: &str) -> u32 {
    renderer
        .measure_string(s, Point::zero(), Baseline::Top)
        .next_position
        .x as u32
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
