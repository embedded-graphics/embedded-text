//! Extends font types with some helper methods.
use core::str::Chars;
use embedded_graphics::{fonts::Font, geometry::Point};

/// `Font` extensions
pub trait FontExt {
    /// Measures a sequence of characters in a line with a determinate maximum width.
    ///
    /// Returns the width of the characters that fit into the given space and whether or not all of
    /// the input fits into the given space.
    fn max_fitting(iter: Chars<'_>, max_width: u32) -> (u32, bool);

    /// Returns the value of a pixel in a character in the font.
    fn character_point(c: char, p: Point) -> bool;

    /// Returns the total width of the character plus the character spacing.
    fn total_char_width(c: char) -> u32;

    /// Measures text height when rendered using a given width.
    fn measure_text(text: &str, max_width: u32) -> u32;
}

impl<F> FontExt for F
where
    F: Font,
{
    #[inline]
    #[must_use]
    fn max_fitting(iter: Chars<'_>, max_width: u32) -> (u32, bool) {
        let mut total_width = 0;
        for c in iter {
            let new_width = total_width + F::char_width(c);
            if new_width <= max_width {
                total_width = new_width;
            } else {
                return (total_width, false);
            }
        }

        (total_width, true)
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
                for c in line.chars() {
                    if c == '\r' {
                        total_width = 0;
                    } else {
                        let new_width = F::total_char_width(c);
                        if total_width + new_width <= max_width {
                            total_width += new_width;
                        } else {
                            current_rows += 1;
                            total_width = new_width;
                        }
                    }
                }
                current_rows
            })
            .sum::<u32>();

        line_count * F::CHARACTER_SIZE.height
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use embedded_graphics::fonts::Font6x8;

    #[test]
    fn test_max_fitting_empty() {
        assert_eq!(Font6x8::max_fitting("".chars(), 54), (0, true))
    }

    #[test]
    fn test_max_fitting_exact() {
        assert_eq!(Font6x8::max_fitting("somereall".chars(), 54), (54, true))
    }

    #[test]
    fn test_max_fitting_long_exact() {
        assert_eq!(
            Font6x8::max_fitting("somereallylongword".chars(), 54),
            (54, false)
        )
    }

    #[test]
    fn test_max_fitting_long() {
        assert_eq!(
            Font6x8::max_fitting("somereallylongword".chars(), 55),
            (54, false)
        )
    }

    #[test]
    fn test_height_empty() {
        let data = [
            ("", 0, 0),
            ("word", 50, 8),
            ("word\nnext", 50, 16),
            ("verylongword", 50, 16),
            ("some verylongword", 50, 24),
            ("1 23456 12345 61234 561", 36, 40),
        ];
        for (text, width, expected_height) in data.iter() {
            assert_eq!(Font6x8::measure_text(text, *width), *expected_height);
        }
    }
}
