//! Extends font types with some helper methods.
use crate::parser::{Parser, Token};
use core::str::Chars;
use embedded_graphics::{fonts::Font, geometry::Point};

/// `Font` extensions
pub trait FontExt {
    /// Measures a sequence of characters in a line with a determinate maximum width.
    ///
    /// Returns the width of the characters that fit into the given space and whether or not all of
    /// the input fits into the given space.
    fn measure_line(iter: Chars<'_>, max_width: u32) -> LineMeasurement;

    /// Returns the value of a pixel in a character in the font.
    fn character_point(c: char, p: Point) -> bool;

    /// Returns the total width of the character plus the character spacing.
    fn total_char_width(c: char) -> u32;

    /// Measures text height when rendered using a given width.
    fn measure_text(text: &str, max_width: u32) -> u32;

    /// Measure text width
    fn str_width(s: &str) -> u32;
}

/// Result of a `measure_line` function call.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineMeasurement {
    /// The maximum width that still fits into the given width limit.
    pub width: u32,

    /// Whether or not the whole sequence fits into the given width limit.
    pub fits_line: bool,
}

impl LineMeasurement {
    /// Creates a new measurement result object.
    #[inline]
    #[must_use]
    pub const fn new(width: u32, fits_line: bool) -> Self {
        LineMeasurement { width, fits_line }
    }

    /// Creates a new measurement result object for an empty line.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self::new(0, true)
    }
}

impl<F> FontExt for F
where
    F: Font,
{
    #[inline]
    #[must_use]
    fn measure_line(chars: Chars<'_>, max_width: u32) -> LineMeasurement {
        let mut total_width = 0;

        for c in chars {
            let new_width = total_width + F::total_char_width(c);
            if new_width > max_width {
                return LineMeasurement::new(total_width, false);
            } else {
                total_width = new_width;
            }
        }

        LineMeasurement::new(total_width, true)
    }

    #[inline]
    #[must_use]
    fn character_point(c: char, p: Point) -> bool {
        Self::character_pixel(c, p.x as u32, p.y as u32)
    }

    #[inline]
    fn total_char_width(c: char) -> u32 {
        F::char_width(c) + F::CHARACTER_SPACING
    }

    #[inline]
    #[must_use]
    fn measure_text(text: &str, max_width: u32) -> u32 {
        let line_count = text
            .lines()
            .map(|line| {
                let mut current_rows = 1;
                let mut total_width = 0;
                for token in Parser::parse(line) {
                    match token {
                        Token::Word(w) => {
                            let mut word_width = 0;
                            for c in w.chars() {
                                let width = F::total_char_width(c);
                                if total_width + word_width + width <= max_width {
                                    // letter fits, letter is added to word width
                                    word_width += width;
                                } else {
                                    // letter (and word) doesn't fit this line, open a new one
                                    current_rows += 1;
                                    if total_width == 0 {
                                        // first word gets a line break in current pos
                                        word_width = width;
                                        total_width = width;
                                    } else {
                                        // other words get wrapped
                                        word_width += width;
                                        total_width = 0;
                                    }
                                }
                            }

                            total_width += word_width;
                        }

                        Token::Whitespace(n) => {
                            let width = F::total_char_width(' ');
                            for _ in 0..n {
                                if total_width + width <= max_width {
                                    total_width += width;
                                } else {
                                    current_rows += 1;
                                    total_width = width;
                                }
                            }
                        }

                        _ => {}
                    }
                }
                current_rows
            })
            .sum::<u32>();

        line_count * F::CHARACTER_SIZE.height
    }

    #[inline]
    fn str_width(s: &str) -> u32 {
        s.chars().map(F::total_char_width).sum::<u32>()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use embedded_graphics::fonts::Font6x8;

    #[test]
    fn test_max_fitting_empty() {
        assert_eq!(
            Font6x8::measure_line("".chars(), 54),
            LineMeasurement::new(0, true)
        )
    }

    #[test]
    fn test_max_fitting_exact() {
        let measurement = Font6x8::measure_line("somereall".chars(), 54);
        assert_eq!(measurement, LineMeasurement::new(54, true));
    }

    #[test]
    fn test_max_fitting_long_exact() {
        let measurement = Font6x8::measure_line("somereallylongword".chars(), 54);
        assert_eq!(measurement, LineMeasurement::new(54, false));
    }

    #[test]
    fn test_max_fitting_long() {
        let measurement = Font6x8::measure_line("somereallylongword".chars(), 55);
        assert_eq!(measurement, LineMeasurement::new(54, false));
    }

    #[test]
    fn test_height() {
        let data = [
            ("", 0, 0),
            ("word", 50, 8),
            ("word\nnext", 50, 16),
            ("verylongword", 50, 16),
            ("some verylongword", 50, 24),
            ("1 23456 12345 61234 561", 36, 40),
        ];
        for (text, width, expected_height) in data.iter() {
            let height = Font6x8::measure_text(text, *width);
            assert_eq!(
                height, *expected_height,
                "Height of \"{}\" is {} but is expected to be {}",
                text, height, expected_height
            );
        }
    }
}
