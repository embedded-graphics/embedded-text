//! Font helper extensions.
//!
//! Extends font types with some helper methods.
use embedded_graphics::fonts::Font;

/// `Font` extensions
pub trait FontExt {
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

    /// Returns the y offset for the strikethrough line.
    fn strikethrough_pos() -> u32;
}

fn str_width<F: Font>(s: &str, ignore_cr: bool) -> u32 {
    let mut width = 0;
    let mut current_width = 0;
    for c in s.chars() {
        if !ignore_cr && c == '\r' {
            width = current_width.max(width);
            current_width = 0;
        } else {
            current_width += F::total_char_width(c);
        }
    }

    current_width.max(width)
}

fn max_str_width<F: Font>(s: &str, max_width: u32, ignore_cr: bool) -> (u32, &str) {
    let mut width = 0;
    let mut current_width = 0;
    for (idx, c) in s.char_indices() {
        if !ignore_cr && c == '\r' {
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
    fn str_width(s: &str) -> u32 {
        str_width::<F>(s, false)
    }

    #[inline]
    fn str_width_nocr(s: &str) -> u32 {
        str_width::<F>(s, true)
    }

    #[inline]
    #[must_use]
    fn max_str_width(s: &str, max_width: u32) -> (u32, &str) {
        max_str_width::<F>(s, max_width, false)
    }

    #[inline]
    #[must_use]
    fn max_str_width_nocr(s: &str, max_width: u32) -> (u32, &str) {
        max_str_width::<F>(s, max_width, true)
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
    fn test_str_width() {
        let data: [(&str, u32); 4] = [("", 0), ("foo", 3), ("foo\rbar", 3), ("foo\rfoobar", 6)];
        for (word, chars) in data.iter() {
            assert_eq!(chars * 6, Font6x8::str_width(word));
        }
    }

    #[test]
    fn test_max_fitting_empty() {
        assert_eq!((0, ""), Font6x8::max_str_width("", 54));
        assert_eq!((0, ""), Font6x8::max_str_width_nocr("", 54));
    }

    #[test]
    fn test_max_fitting_exact() {
        assert_eq!((54, "somereall"), Font6x8::max_str_width("somereall", 54));
        assert_eq!(
            (54, "somereall"),
            Font6x8::max_str_width_nocr("somereall", 54)
        );
    }

    #[test]
    fn test_max_fitting_long_exact() {
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
