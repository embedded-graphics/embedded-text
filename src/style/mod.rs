//! `TextBox` styling.
//!
//! By itself, a [`TextBox`] does not contain the information necessary to draw it on a display.
//! This information is called "style" and it is contained in [`TextBoxStyle`] objects.
//!
//! To create a [`TextBoxStyle`], you can use the [`TextBoxStyle::new`] and
//! [`TextBoxStyle::from_text_style`] constructors, or the [`TextBoxStyleBuilder`] builder object.
//!
//! To apply a style, call [`TextBox::into_styled`].
//!
//! [`TextBox`]: ../struct.TextBox.html
//! [`TextBoxStyle::new`]: struct.TextBoxStyle.html#method.new
//! [`TextBoxStyle::from_text_style`]: struct.TextBoxStyle.html#method.from_text_style
//! [`TextBoxStyleBuilder`]: builder/struct.TextBoxStyleBuilder.html
//! [`TextBox::into_styled`]: ../struct.TextBox.html#method.into_styled
use embedded_graphics::{prelude::*, style::TextStyle};

pub mod builder;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    utils::font_ext::FontExt,
    StyledTextBox,
};
pub use builder::TextBoxStyleBuilder;

/// Specifies how the [`TextBox`]'s height is adjusted when it's turned into a [`StyledTextBox`].
pub trait HeightMode: Copy {
    /// Apply the height mode to the textbox
    fn apply<C, F, A, V, H>(text_box: &mut StyledTextBox<'_, C, F, A, V, H>)
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode;
}

/// Keep the original height
#[derive(Copy, Clone, Debug)]
pub struct Exact;

impl HeightMode for Exact {
    #[inline]
    fn apply<C, F, A, V, H>(_text_box: &mut StyledTextBox<'_, C, F, A, V, H>)
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
    }
}

/// Change the height to exactly fit the text
#[derive(Copy, Clone, Debug)]
pub struct FitToText;

impl HeightMode for FitToText {
    #[inline]
    fn apply<C, F, A, V, H>(text_box: &mut StyledTextBox<'_, C, F, A, V, H>)
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        text_box.fit_height();
    }
}

/// If the text doesn't fill the original height, shrink the [`StyledTextBox`] to be as tall as the
/// text.
#[derive(Copy, Clone, Debug)]
pub struct ShrinkToText;

impl HeightMode for ShrinkToText {
    #[inline]
    fn apply<C, F, A, V, H>(text_box: &mut StyledTextBox<'_, C, F, A, V, H>)
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        text_box.fit_height_limited(text_box.size().height);
    }
}

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
pub struct TextBoxStyle<C, F, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    /// Style properties for text.
    pub text_style: TextStyle<C, F>,

    /// Horizontal text alignment.
    pub alignment: A,

    /// Vertical text alignment.
    pub vertical_alignment: V,

    /// The height behaviour
    pub height_mode: H,
}

impl<C, F, A, V, H> TextBoxStyle<C, F, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    /// Creates a `TextBoxStyle` object with transparent background.
    #[inline]
    pub fn new(
        font: F,
        text_color: C,
        alignment: A,
        vertical_alignment: V,
        height_mode: H,
    ) -> Self {
        Self {
            text_style: TextStyle::new(font, text_color),
            alignment,
            vertical_alignment,
            height_mode,
        }
    }

    /// Creates a `TextBoxStyle` object from the given text style and alignment.
    #[inline]
    pub fn from_text_style(
        text_style: TextStyle<C, F>,
        alignment: A,
        vertical_alignment: V,
        height_mode: H,
    ) -> Self {
        Self {
            text_style,
            alignment,
            vertical_alignment,
            height_mode,
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

    /// Counts the number of printed whitespaces in a word.
    ///
    /// This function only counts whitespaces that the `Parser` can include in a `Token::Word`.
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
                r#"#{}: Height of "{}" is {} but is expected to be {}"#,
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
                r#"#{}: Height of "{}" is {} but is expected to be {}"#,
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
