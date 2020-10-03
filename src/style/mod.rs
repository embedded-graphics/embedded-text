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

pub mod builder;
pub mod height_mode;
pub mod vertical_overdraw;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        cursor::Cursor,
        line_iter::{LineElementIterator, RenderElement},
        space_config::UniformSpaceConfig,
    },
    style::height_mode::HeightMode,
    utils::font_ext::FontExt,
};
use core::marker::PhantomData;
use embedded_graphics::{prelude::*, primitives::Rectangle, style::TextStyle};

pub use builder::TextBoxStyleBuilder;

/// Tab size helper
///
/// This type makes it more obvious what unit is used to define the width of tabs.
/// The default tab size is 4 spaces.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TabSize<F: Font> {
    pub(crate) width: i32,
    _font: PhantomData<F>,
}

impl<F: Font> Default for TabSize<F> {
    #[inline]
    fn default() -> Self {
        Self::spaces(4)
    }
}

impl<F: Font> TabSize<F> {
    /// Calculate tab size from a number of spaces in the current font.
    #[inline]
    pub fn spaces(n: u32) -> Self {
        let space = F::total_char_width(' ') as i32;
        // make sure n is at least 1, and the multiplication doesn't overflow
        let size = (n.max(1) as i32).checked_mul(space).unwrap_or(4 * space);

        Self::pixels(size)
    }

    /// Define the tab size in pixels.
    #[inline]
    pub fn pixels(px: i32) -> Self {
        Self {
            width: px,
            _font: PhantomData,
        }
    }

    /// Calculate the rendered with of the next tab
    #[inline]
    pub fn next_width(self, pos: i32) -> u32 {
        let next_tab_pos = (pos / self.width + 1) * self.width;
        (next_tab_pos - pos) as u32
    }
}

/// Styling options of a [`TextBox`].
///
/// `TextBoxStyle` contains the `Font`, foreground and background `PixelColor`, line spacing,
/// [`HeightMode`], [`HorizontalTextAlignment`] and [`VerticalTextAlignment`] information necessary
/// to draw a [`TextBox`].
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

    /// Desired space between lines, in pixels
    pub line_spacing: i32,

    /// Desired column width for tabs
    pub tab_size: TabSize<F>,
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
            line_spacing: 0,
            tab_size: TabSize::default(),
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
            line_spacing: 0,
            tab_size: TabSize::default(),
        }
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
        let cursor: Cursor<F> = Cursor::new(
            Rectangle::new(
                Point::zero(),
                Point::new(
                    max_line_width.saturating_sub(1) as i32,
                    F::CHARACTER_SIZE.height.saturating_sub(1) as i32,
                ),
            ),
            self.line_spacing,
        );
        let mut iter: LineElementIterator<'_, F, _, A> = LineElementIterator::new(
            parser.clone(),
            cursor,
            UniformSpaceConfig::default(),
            carried_token.clone(),
            self.tab_size,
        );

        let mut current_width = 0;
        let mut last_spaces = 0;
        let mut total_spaces = 0;
        while let Some(token) = iter.next() {
            match token {
                RenderElement::Space(_, count) => {
                    if A::ENDING_SPACES {
                        // only track width if spaces are rendered at the end of a line
                        current_width = iter.cursor.position.x;

                        // in this case, count all spaces
                        total_spaces += count;
                    } else {
                        // ... otherwise save the number of spaces and it will be tracked with
                        // the next printed character, or it will be discarded
                        last_spaces = total_spaces + count;
                    }
                }

                RenderElement::PrintedCharacter(c) => {
                    // the current width is always the position where the cursor is (left is 0)
                    current_width = iter.cursor.position.x;

                    if c == '\u{A0}' {
                        total_spaces += 1;
                    } else if !A::ENDING_SPACES {
                        // if ENDING_SPACES is true, spaces have already been counted and
                        // last_spaces is 0
                        total_spaces = last_spaces;
                    }
                }

                // Ignore color changes
                _ => {}
            }
        }

        let carried = iter.remaining_token();
        *parser = iter.parser;
        (current_width as u32, total_spaces, carried)
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
        let mut n_lines = 0_i32;
        let mut parser = Parser::parse(text);
        let mut carry = None;

        loop {
            let (w, _, t) = self.measure_line(&mut parser, carry.clone(), max_width);

            if (w != 0 || t.is_some()) && carry != Some(Token::CarriageReturn) {
                // something was in this line, increment height
                // if last carried token was a carriage return, we already counted the height
                n_lines += 1;
            }

            if t.is_none() {
                return (n_lines * F::CHARACTER_SIZE.height as i32
                    + n_lines.saturating_sub(1) * self.line_spacing) as u32;
            }

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
    fn no_infinite_loop() {
        let _ = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .build()
            .measure_text_height("a", 5);
    }

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
    fn test_measure_line() {
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .build();

        let mut text = Parser::parse("123 45 67");

        let (w, s, _) =
            textbox_style.measure_line(&mut text, None, 6 * Font6x8::CHARACTER_SIZE.width);
        assert_eq!(w, 6 * Font6x8::CHARACTER_SIZE.width);
        assert_eq!(s, 1);
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

    #[test]
    fn height_with_line_spacing() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .line_spacing(2)
            .build();

        let height = style.measure_text_height(
            "Lorem Ipsum is simply dummy text of the printing and typesetting industry.",
            72,
        );

        assert_eq!(height, 7 * 8 + 6 * 2);
    }
}
