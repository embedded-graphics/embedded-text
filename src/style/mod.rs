//! Textbox styling.
use embedded_graphics::{prelude::*, style::TextStyle};

pub mod builder;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{StateFactory, StyledTextBoxIterator},
    utils::{font_ext::FontExt, rect_ext::RectExt},
    TextBox,
};
pub use builder::TextBoxStyleBuilder;

/// Styling options of a [`TextBox`].
///
/// `TextBoxStyle` contains the `Font`, foreground and background `PixelColor`,
/// [`HorizontalTextAlignment`] and [`VerticalTextAlignment`] information necessary to draw a
/// [`TextBox`].
///
/// To construct a new `TextBoxStyle` object, use the [`new`] or [`from_text_style`] methods or
/// the [`TextBoxStyleBuilder`] object.
///
/// [`TextBox`]: ../struct.TextBox.html
/// [`HorizontalTextAlignment`]: ../alignment/trait.HorizontalTextAlignment.html
/// [`VerticalTextAlignment`]: ../alignment/trait.VerticalTextAlignment.html
/// [`TextBoxStyleBuilder`]: builder/struct.TextBoxStyleBuilder.html
/// [`new`]: #method.new
/// [`from_text_style`]: #method.from_text_style
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct TextBoxStyle<C, F, A, V>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
{
    /// Style properties for text.
    pub text_style: TextStyle<C, F>,

    /// Horizontal text alignment.
    pub alignment: A,

    /// Vertical text alignment.
    pub vertical_alignment: V,
}

impl<C, F, A, V> TextBoxStyle<C, F, A, V>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
{
    /// Creates a `TextBoxStyle` object with transparent background.
    #[inline]
    pub fn new(font: F, text_color: C, alignment: A, vertical_alignment: V) -> Self {
        Self {
            text_style: TextStyle::new(font, text_color),
            alignment,
            vertical_alignment,
        }
    }

    /// Creates a `TextBoxStyle` object from the given text style and alignment.
    #[inline]
    pub fn from_text_style(
        text_style: TextStyle<C, F>,
        alignment: A,
        vertical_alignment: V,
    ) -> Self {
        Self {
            text_style,
            alignment,
            vertical_alignment,
        }
    }

    /// Returns the size of a token if it fits the line, or the max size that fits and the remaining
    /// unprocessed part.
    fn measure_word(w: &str, max_width: u32) -> (u32, Option<&str>) {
        let (width, consumed) = F::max_str_width_nocr(w, max_width);
        let carried = if consumed == w {
            None
        } else {
            Some(unsafe {
                // consumed is the first part of w, so it's length must be
                // on char boundary
                w.get_unchecked(consumed.len()..)
            })
        };

        (width, carried)
    }

    /// Counts the number of printed whitespaces in a word
    fn count_printed_spaces(w: &str) -> u32 {
        w.chars().filter(|&c| c == '\u{A0}').count() as u32
    }

    /// Measure the width and count spaces in a single line of text.
    ///
    /// Returns (width, rendered space count, carried token)
    ///
    /// Instead of peeking ahead when processing tokens, this function advances the parser before
    /// processing a token. If a token opens a new line, it will be returned as the carried token.
    /// If the carried token is `None`, the parser has finished processing the text.
    #[inline]
    #[must_use]
    pub fn measure_line<'a>(
        &self,
        parser: &mut Parser<'a>,
        carried_token: Option<Token<'a>>,
        max_line_width: u32,
    ) -> (u32, u32, Option<Token<'a>>) {
        let mut current_width = 0;
        let mut last_spaces = 0;
        let mut last_space_width = 0;
        let mut is_first_word = true;

        if let Some(t) = carried_token {
            match t {
                Token::Word(w) => {
                    let (width, carried) = Self::measure_word(w, max_line_width);

                    if let Some(w) = carried {
                        let spaces = Self::count_printed_spaces(w);
                        return (width, spaces, Some(Token::Word(w)));
                    }

                    is_first_word = false;
                    current_width = width;
                }

                Token::Whitespace(n) => {
                    if A::STARTING_SPACES {
                        let (width, consumed) = F::max_space_width(n, max_line_width);
                        let carried = n - consumed;

                        if carried != 0 {
                            let token = Some(Token::Whitespace(carried - 1));
                            return if A::ENDING_SPACES {
                                (width, consumed, token)
                            } else {
                                (0, 0, token)
                            };
                        }

                        last_spaces = n;
                        last_space_width = width;
                    }
                }

                Token::NewLine => {
                    // eat the newline
                }

                Token::CarriageReturn => {
                    // eat the \r since it's meaningless in the beginning of a line
                }
            }
        }

        let mut total_spaces = 0;
        let mut carried_token = None;
        for token in parser {
            let available_space = max_line_width - current_width - last_space_width;
            match token {
                Token::Word(w) => {
                    let (width, carried) = Self::measure_word(w, available_space);

                    if let Some(carried_w) = carried {
                        let carried_word = if is_first_word {
                            if width != 0 {
                                let spaces =
                                    Self::count_printed_spaces(&w[0..w.len() - carried_w.len()]);
                                current_width += last_space_width + width;
                                total_spaces += last_spaces + spaces;
                            }
                            // first word; break word into parts
                            carried_w
                        } else {
                            // carry the whole word
                            w
                        };
                        carried_token.replace(Token::Word(carried_word));
                        break;
                    }

                    let spaces = Self::count_printed_spaces(w);
                    // If there's no carried token, width != 0 assuming there are no empty words
                    current_width += last_space_width + width;
                    total_spaces += last_spaces + spaces;

                    is_first_word = false;
                    last_space_width = 0;
                    last_spaces = 0;
                }

                Token::Whitespace(n) => {
                    if A::STARTING_SPACES || !is_first_word {
                        let (width, mut consumed) = F::max_space_width(n, available_space);

                        // update before breaking, so that ENDING_SPACES can use data
                        last_spaces += n;
                        last_space_width += width;

                        if n != consumed {
                            if A::ENDING_SPACES {
                                consumed += 1;
                            }
                            carried_token.replace(Token::Whitespace(n - consumed));
                            break;
                        }
                    }
                }

                Token::NewLine | Token::CarriageReturn => {
                    carried_token.replace(token);
                    break;
                }
            }
        }
        if A::ENDING_SPACES {
            total_spaces += last_spaces;
            current_width += last_space_width;
        }
        (current_width, total_spaces, carried_token)
    }

    /// Measures text height when rendered using a given width.
    ///
    /// # Example: measure height of text when rendered using a 6x8 font and 72px width.
    ///
    /// ```rust
    /// # use embedded_text::style::builder::TextBoxStyleBuilder;
    /// # use embedded_graphics::fonts::Font6x8;
    /// # use embedded_graphics::pixelcolor::BinaryColor;
    /// #
    /// let style = TextBoxStyleBuilder::new(Font6x8)
    ///     .text_color(BinaryColor::On)
    ///     .build();
    ///
    /// let height = style.measure_text_height(
    ///     "Lorem Ipsum is simply dummy text of the printing and typesetting industry.",
    ///     72,
    /// );
    ///
    /// // Expect 7 lines of text, wrapped in something like the following:
    ///
    /// // |Lorem Ipsum |
    /// // |is simply   |
    /// // |dummy text  |
    /// // |of the      |
    /// // |printing and|
    /// // |typesetting |
    /// // |industry.   |
    ///
    /// assert_eq!(7 * 8, height);
    /// ```
    #[inline]
    #[must_use]
    pub fn measure_text_height(&self, text: &str, max_width: u32) -> u32 {
        let mut current_height = 0;
        let mut parser = Parser::parse(text);
        let mut carry = None;
        let mut bytes = parser.remaining();

        loop {
            let (w, _, t) = self.measure_line(&mut parser, carry.clone(), max_width);
            let remaining = parser.remaining();
            // exit condition:
            // - no more tokens to process
            // - the same token is encountered twice
            if t.is_none() || (t == carry && bytes == remaining) {
                if w != 0 {
                    current_height += F::CHARACTER_SIZE.height;
                }
                return current_height;
            }
            if t != Some(Token::CarriageReturn) {
                current_height += F::CHARACTER_SIZE.height;
            }
            bytes = remaining;
            carry = t;
        }
    }
}

/// A styled [`TextBox`] struct.
///
/// This structure is constructed by calling the [`into_styled`] method of a [`TextBox`] object.
/// Use the [`draw`] method to draw the textbox on a display.
///
/// [`TextBox`]: ../struct.TextBox.html
/// [`into_styled`]: ../struct.TextBox.html#method.into_styled
/// [`draw`]: #method.draw
pub struct StyledTextBox<'a, C, F, A, V>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
{
    /// A [`TextBox`] that has an associated [`TextBoxStyle`].
    ///
    /// [`TextBox`]: ../struct.TextBox.html
    /// [`TextBoxStyle`]: struct.TextBoxStyle.html
    pub text_box: TextBox<'a>,

    /// The style of the [`TextBox`].
    ///
    /// [`TextBox`]: ../struct.TextBox.html
    pub style: TextBoxStyle<C, F, A, V>,
}

impl<'a, C, F, A, V> Drawable<C> for &'a StyledTextBox<'a, C, F, A, V>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    StyledTextBoxIterator<'a, C, F, A, V>: Iterator<Item = Pixel<C>>,
    StyledTextBox<'a, C, F, A, V>: StateFactory<F>,
{
    #[inline]
    fn draw<D: DrawTarget<C>>(self, display: &mut D) -> Result<(), D::Error> {
        display.draw_iter(StyledTextBoxIterator::new(self))
    }
}

impl<C, F, A, V> Transform for StyledTextBox<'_, C, F, A, V>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
{
    #[inline]
    #[must_use]
    fn translate(&self, by: Point) -> Self {
        Self {
            text_box: self.text_box.translate(by),
            style: self.style,
        }
    }

    #[inline]
    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.text_box.bounds.translate_mut(by);

        self
    }
}

impl<C, F, A, V> Dimensions for StyledTextBox<'_, C, F, A, V>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
{
    #[inline]
    #[must_use]
    fn top_left(&self) -> Point {
        self.text_box.bounds.top_left
    }

    #[inline]
    #[must_use]
    fn bottom_right(&self) -> Point {
        self.text_box.bounds.bottom_right
    }

    #[inline]
    #[must_use]
    fn size(&self) -> Size {
        RectExt::size(self.text_box.bounds)
    }
}

#[cfg(test)]
mod test {
    use crate::{alignment::*, parser::Parser, style::builder::TextBoxStyleBuilder};
    use embedded_graphics::{
        fonts::{Font, Font6x8},
        pixelcolor::BinaryColor,
    };

    #[test]
    fn test_measure_height() {
        let data = [
            ("", 0, 0),
            (" ", 0, 8),
            (" ", 5, 8),
            (" ", 6, 8),
            ("\n", 6, 8),
            ("\n ", 6, 16),
            ("word", 4 * 6, 8), // exact fit into 1 line
            ("word", 4 * 6 - 1, 16),
            ("word", 2 * 6, 16),      // exact fit into 2 lines
            ("word word", 4 * 6, 16), // exact fit into 2 lines
            ("word\n", 2 * 6, 16),
            ("word\nnext", 50, 16),
            ("word\n\nnext", 50, 24),
            ("word\n  \nnext", 50, 24),
            ("verylongword", 50, 16),
            ("some verylongword", 50, 24),
            ("1 23456 12345 61234 561", 36, 40),
            ("    Word      ", 36, 24),
            ("Longer\rnowrap", 36, 8),
        ];
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .build();
        for (i, (text, width, expected_height)) in data.iter().enumerate() {
            let height = textbox_style.measure_text_height(text, *width);
            assert_eq!(
                height, *expected_height,
                "#{}: Height of \"{}\" is {} but is expected to be {}",
                i, text, height, expected_height
            );
        }
    }

    #[test]
    fn test_measure_height_ignored_spaces() {
        let data = [
            ("", 0, 0),
            (" ", 0, 0),
            (" ", 6, 0),
            ("\n ", 6, 8),
            ("word\n", 2 * 6, 16),
            ("word\n  \nnext", 50, 24),
            ("    Word      ", 36, 8),
        ];
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .build();
        for (i, (text, width, expected_height)) in data.iter().enumerate() {
            let height = textbox_style.measure_text_height(text, *width);
            assert_eq!(
                height, *expected_height,
                "#{}: Height of \"{}\" is {} but is expected to be {}",
                i, text, height, expected_height
            );
        }
    }

    #[test]
    fn test_measure_line_counts_nbsp() {
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .build();

        let mut text = Parser::parse("123\u{A0}45");

        let (w, s, _) =
            textbox_style.measure_line(&mut text, None, 5 * Font6x8::CHARACTER_SIZE.width);
        assert_eq!(w, 5 * Font6x8::CHARACTER_SIZE.width);
        assert_eq!(s, 1);
    }

    #[test]
    fn test_measure_height_nbsp() {
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .build();

        let text = "123\u{A0}45 123";

        let height = textbox_style.measure_text_height(text, 5 * Font6x8::CHARACTER_SIZE.width);
        assert_eq!(height, 16);

        // bug discovered while using the interactive example
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .build();

        let text = "embedded-text also\u{A0}supports non-breaking spaces.";

        let height = textbox_style.measure_text_height(text, 79);
        assert_eq!(height, 4 * Font6x8::CHARACTER_SIZE.height);
    }
}
