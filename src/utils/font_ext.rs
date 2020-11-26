//! Font helper extensions.
//!
//! Extends font types with some helper methods.
use embedded_graphics::fonts::Font;

/// `Font` extensions
pub trait FontExt {
    /// Returns the total width of the character plus the character spacing.
    fn total_char_width(c: char) -> u32;

    /// This function is identical to [`str_width`] except it does **not** handle carriage
    /// return characters.
    ///
    /// [`str_width`]: #method.str_width
    fn str_width_nocr(s: &str) -> u32;

    /// Measures a sequence of spaces in a line with a determinate maximum width.
    ///
    /// Returns the width of the spaces that fit into the given space and the number of spaces that
    /// fit.
    fn max_space_width(n: u32, max_width: u32) -> (u32, u32);

    /// Returns the y offset for the strikethrough line.
    fn strikethrough_pos() -> u32;
}

impl<F> FontExt for F
where
    F: Font,
{
    #[inline]
    fn total_char_width(c: char) -> u32 {
        if c == '\u{A0}' {
            // A non-breaking space is as wide as a regular one
            return F::char_width(' ') + F::CHARACTER_SPACING;
        }
        F::char_width(c) + F::CHARACTER_SPACING
    }

    #[inline]
    fn str_width_nocr(s: &str) -> u32 {
        let mut current_width = 0;
        for c in s.chars() {
            current_width += F::total_char_width(c);
        }

        current_width
    }

    #[inline]
    #[must_use]
    fn max_space_width(n: u32, max_width: u32) -> (u32, u32) {
        let space_width = F::total_char_width(' ');
        let num_spaces = (max_width / space_width).min(n);

        (num_spaces * space_width, num_spaces)
    }

    #[inline]
    fn strikethrough_pos() -> u32 {
        F::CHARACTER_SIZE.height / 2
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
    fn test_max_space_width() {
        assert_eq!((0, 0), Font6x8::max_space_width(0, 36));
        assert_eq!((36, 6), Font6x8::max_space_width(6, 36));
        assert_eq!((36, 6), Font6x8::max_space_width(6, 36));
        assert_eq!((36, 6), Font6x8::max_space_width(6, 38));
        assert_eq!((36, 6), Font6x8::max_space_width(7, 36));
    }
}
