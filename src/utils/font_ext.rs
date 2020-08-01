//! Font helper extensions.
//!
//! Extends font types with some helper methods.
use embedded_graphics::fonts::Font;

/// `Font` extensions
pub trait FontExt {
    /// Measures a sequence of characters in a line with a determinate maximum width.
    ///
    /// Returns the width of the characters that fit into the given space and whether or not all of
    /// the input fits into the given space.
    fn measure_line(line: &str, max_width: u32) -> LineMeasurement;

    /// This function is identical to [`measure_line`] except it does **not** handle carriage
    /// return characters.
    ///
    /// [`measure_line`]: #method.measure_line
    fn measure_line_nocr(line: &str, max_width: u32) -> LineMeasurement;

    /// Returns the total width of the character plus the character spacing.
    fn total_char_width(c: char) -> u32;

    /// Measure text width
    fn str_width(s: &str) -> u32;

    /// This function is identical to [`str_width`] except it does **not** handle carriage
    /// return characters.
    ///
    /// [`str_width`]: #method.str_width
    fn str_width_nocr(s: &str) -> u32;

    /// Measures a sequence of characters in a line with a determinate maximum width.
    ///
    /// Returns the width of the characters that fit into the given space and the processed string.
    fn max_str_width(s: &str, max_width: u32) -> (u32, &str);

    /// This function is identical to [`max_str_width`] except it does **not** handle carriage
    /// return characters.
    ///
    /// [`max_str_width`]: #method.max_str_width
    fn max_str_width_nocr(s: &str, max_width: u32) -> (u32, &str);

    /// Measures a sequence of spaces in a line with a determinate maximum width.
    ///
    /// Returns the width of the spaces that fit into the given space and the number of spaces that
    /// fit.
    fn max_space_width(n: u32, max_width: u32) -> (u32, u32);
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
    fn measure_line(line: &str, max_width: u32) -> LineMeasurement {
        let (width, processed) = Self::max_str_width(line, max_width);

        LineMeasurement::new(width, processed == line)
    }

    #[inline]
    #[must_use]
    fn measure_line_nocr(line: &str, max_width: u32) -> LineMeasurement {
        let (width, processed) = Self::max_str_width_nocr(line, max_width);

        LineMeasurement::new(width, processed == line)
    }

    #[inline]
    fn total_char_width(c: char) -> u32 {
        if c == '\u{A0}' {
            // A non-breaking space is as wide as a regular one
            return F::char_width(' ') + F::CHARACTER_SPACING;
        }
        F::char_width(c) + F::CHARACTER_SPACING
    }

    #[inline]
    fn str_width(s: &str) -> u32 {
        let mut width = 0;
        let mut current_width = 0;
        for c in s.chars() {
            if c == '\r' {
                width = current_width.max(width);
                current_width = 0;
            } else {
                current_width += F::total_char_width(c);
            }
        }

        current_width.max(width)
    }

    #[inline]
    fn str_width_nocr(s: &str) -> u32 {
        s.chars().map(F::total_char_width).sum::<u32>()
    }

    #[inline]
    #[must_use]
    fn max_str_width(s: &str, max_width: u32) -> (u32, &str) {
        let mut width = 0;
        let mut current_width = 0;
        for (idx, c) in s.char_indices() {
            if c == '\r' {
                width = current_width.max(width);
                current_width = 0;
            } else {
                let new_width = current_width + F::total_char_width(c);
                if new_width > max_width {
                    width = current_width.max(width);
                    return (width, unsafe { s.get_unchecked(0..idx) });
                } else {
                    current_width = new_width;
                }
            }
        }
        width = current_width.max(width);
        (width, s)
    }

    #[inline]
    #[must_use]
    fn max_str_width_nocr(s: &str, max_width: u32) -> (u32, &str) {
        let mut width = 0;
        for (idx, c) in s.char_indices() {
            let new_width = width + F::total_char_width(c);
            if new_width > max_width {
                return (width, unsafe { s.get_unchecked(0..idx) });
            } else {
                width = new_width;
            }
        }
        (width, s)
    }

    #[inline]
    #[must_use]
    fn max_space_width(n: u32, max_width: u32) -> (u32, u32) {
        let space_width = F::total_char_width(' ');
        let num_spaces = (max_width / space_width).min(n);

        (num_spaces * space_width, num_spaces)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use embedded_graphics::fonts::{Font6x6, Font6x8};

    #[test]
    fn nbsp_width_equal_to_space() {
        assert_eq!(
            Font6x8::total_char_width('\u{A0}'),
            Font6x8::total_char_width(' ')
        );
        assert_eq!(
            Font6x6::total_char_width('\u{A0}'),
            Font6x6::total_char_width(' ')
        );
    }

    #[test]
    fn test_str_width() {
        let data: [(&str, u32); 4] = [("", 0), ("foo", 3), ("foo\rbar", 3), ("foo\rfoobar", 6)];
        for (word, chars) in data.iter() {
            assert_eq!(chars * 6, Font6x8::str_width(word));
        }
    }

    #[test]
    fn test_max_fitting_empty() {
        assert_eq!(LineMeasurement::new(0, true), Font6x8::measure_line("", 54));
        assert_eq!(
            LineMeasurement::new(0, true),
            Font6x8::measure_line_nocr("", 54)
        );
        assert_eq!((0, ""), Font6x8::max_str_width("", 54));
        assert_eq!((0, ""), Font6x8::max_str_width_nocr("", 54));
    }

    #[test]
    fn test_max_fitting_exact() {
        assert_eq!(
            LineMeasurement::new(54, true),
            Font6x8::measure_line("somereall", 54)
        );
        assert_eq!(
            LineMeasurement::new(54, true),
            Font6x8::measure_line_nocr("somereall", 54)
        );
        assert_eq!((54, "somereall"), Font6x8::max_str_width("somereall", 54));
        assert_eq!(
            (54, "somereall"),
            Font6x8::max_str_width_nocr("somereall", 54)
        );
    }

    #[test]
    fn test_max_fitting_long_exact() {
        assert_eq!(
            LineMeasurement::new(54, false),
            Font6x8::measure_line("somereallylongword", 54)
        );
        assert_eq!(
            LineMeasurement::new(54, false),
            Font6x8::measure_line_nocr("somereallylongword", 54)
        );
        assert_eq!(
            (54, "somereall"),
            Font6x8::max_str_width("somereallylongword", 54)
        );
        assert_eq!(
            (54, "somereall"),
            Font6x8::max_str_width_nocr("somereallylongword", 54)
        );
    }

    #[test]
    fn test_max_fitting_long() {
        assert_eq!(
            LineMeasurement::new(54, false),
            Font6x8::measure_line("somereallylongword", 55)
        );
        assert_eq!(
            LineMeasurement::new(54, false),
            Font6x8::measure_line_nocr("somereallylongword", 55)
        );
        assert_eq!(
            (54, "somereall"),
            Font6x8::max_str_width("somereallylongword", 55)
        );
        assert_eq!(
            (54, "somereall"),
            Font6x8::max_str_width_nocr("somereallylongword", 55)
        );
    }

    #[test]
    fn test_cr() {
        assert_eq!(
            LineMeasurement::new(48, true),
            Font6x8::measure_line("somereal\rlylong", 55)
        );
        assert_eq!(
            (48, "somereal\rlylong"),
            Font6x8::max_str_width("somereal\rlylong", 55)
        );
    }

    #[test]
    fn test_max_space_width() {
        assert_eq!((0, 0), Font6x8::max_space_width(0, 36));
        assert_eq!((36, 6), Font6x8::max_space_width(6, 36));
        assert_eq!((36, 6), Font6x8::max_space_width(6, 36));
        assert_eq!((36, 6), Font6x8::max_space_width(6, 38));
        assert_eq!((36, 6), Font6x8::max_space_width(7, 36));
    }
}
