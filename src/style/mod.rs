//! Textbox styling.
use embedded_graphics::{prelude::*, style::TextStyle};

pub mod builder;

use crate::{
    alignment::TextAlignment,
    parser::{Parser, Token},
    rendering::{StateFactory, StyledTextBoxIterator},
    utils::{font_ext::FontExt, rect_ext::RectExt},
    TextBox,
};
pub use builder::TextBoxStyleBuilder;

/// Styling options of a [`TextBox`].
///
/// `TextBoxStyle` contains the `Font`, foreground and background `PixelColor` and
/// [`TextAlignment`] information necessary to draw a [`TextBox`].
///
/// To construct a new `TextBoxStyle` object, use the [`new`] or [`from_text_style`] methods or
/// the [`TextBoxStyleBuilder`] object.
///
/// [`TextBox`]: ../struct.TextBox.html
/// [`TextAlignment`]: ../alignment/trait.TextAlignment.html
/// [`TextBoxStyleBuilder`]: builder/struct.TextBoxStyleBuilder.html
/// [`new`]: #method.new
/// [`from_text_style`]: #method.from_text_style
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct TextBoxStyle<C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    /// Style properties for text.
    pub text_style: TextStyle<C, F>,

    /// Horizontal text alignment.
    pub alignment: A,
}

impl<C, F, A> TextBoxStyle<C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    /// Creates a `TextBoxStyle` object with transparent background.
    #[inline]
    pub fn new(font: F, text_color: C, alignment: A) -> Self {
        Self {
            text_style: TextStyle::new(font, text_color),
            alignment,
        }
    }

    /// Creates a `TextBoxStyle` object from the given text style and alignment.
    #[inline]
    pub fn from_text_style(text_style: TextStyle<C, F>, alignment: A) -> Self {
        Self {
            text_style,
            alignment,
        }
    }

    // Returns the size of a token if it fits the line, or the max size that fits and the remaining
    // unprocessed part.
    fn measure_word(w: &str, max_width: u32) -> (u32, Option<&str>) {
        let (width, consumed) = F::max_str_width(w, max_width);
        if consumed == w {
            (width, None)
        } else {
            (
                width,
                Some(unsafe {
                    // consumed is the first part of w, so it's length must be
                    // on char boundary
                    w.get_unchecked(consumed.len()..)
                }),
            )
        }
    }

    /// Measure the width and count spaces in a single line of text.
    ///
    /// Returns (width, rendered space count, unprocessed token)
    #[inline]
    pub fn measure_line<'a>(
        &self,
        parser: &mut Parser<'a>,
        carried_token: Option<Token<'a>>,
        max_line_width: u32,
    ) -> (u32, u32, Option<Token<'a>>) {
        let mut current_width = 0;
        let mut total_spaces = 0;
        let mut last_spaces = 0;
        let mut last_space_width = 0;

        let mut first_word_processed = false;

        if let Some(t) = carried_token {
            match t {
                Token::Word(w) => {
                    let (width, carried) = Self::measure_word(w, max_line_width);

                    if let Some(w) = carried {
                        return (width, 0, Some(Token::Word(w)));
                    }

                    first_word_processed = true;
                    current_width = width;
                }

                Token::Whitespace(n) => {
                    let (width, carried) = if A::STARTING_SPACES {
                        let (w, consumed) = F::max_space_width(n, max_line_width);
                        (w, n - consumed)
                    } else {
                        (0, 0)
                    };

                    if carried != 0 {
                        let token = Some(Token::Whitespace(carried));
                        return if A::ENDING_SPACES {
                            (width, n - carried, token)
                        } else {
                            (0, 0, token)
                        };
                    }

                    last_spaces = n;
                    last_space_width = width;
                }

                Token::NewLine => {
                    // eat the newline, although it shoulnd't be carried
                    // todo remove this
                    unreachable!();
                }
            }
        }

        let mut carried_token: Option<Token<'_>> = None;
        for token in parser {
            match token {
                Token::Word(w) => {
                    let (width, carried) =
                        Self::measure_word(w, max_line_width - current_width - last_space_width);

                    if let Some(carried_w) = carried {
                        if first_word_processed {
                            // carry the whole word
                            carried_token.replace(Token::Word(w));
                        } else {
                            if width != 0 {
                                current_width += last_space_width + width;
                                total_spaces += last_spaces;
                            }
                            // first word; break word into parts
                            carried_token.replace(Token::Word(carried_w));
                        }
                        break;
                    }
                    if width != 0 {
                        current_width += last_space_width + width;
                        total_spaces += last_spaces;
                    }

                    first_word_processed = true;
                    last_space_width = 0;
                    last_spaces = 0;
                }

                Token::Whitespace(n) => {
                    let (width, carried) = if A::STARTING_SPACES || first_word_processed {
                        let (w, consumed) = F::max_space_width(
                            n,
                            max_line_width - current_width - last_space_width,
                        );
                        (w, n - consumed)
                    } else {
                        (0, 0)
                    };

                    // update before breaking, so that ENDING_SPACES can use data
                    last_spaces += n;
                    last_space_width += width;

                    if carried != 0 {
                        carried_token.replace(Token::Whitespace(carried));
                        break;
                    }
                }

                Token::NewLine => {
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
    #[inline]
    #[must_use]
    pub fn measure_text_height(&self, text: &str, max_width: u32) -> u32 {
        let line_count = text
            .lines()
            .map(|line| {
                let mut current_rows = 0;
                let mut parser = Parser::parse(line);
                let mut carry = None;

                loop {
                    let (w, _, t) = self.measure_line(&mut parser, carry.take(), max_width);
                    if w != 0 {
                        current_rows += 1;
                    }
                    if t.is_none() {
                        break;
                    }
                    carry = t;
                }

                current_rows
            })
            .sum::<u32>();
        line_count * F::CHARACTER_SIZE.height
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
pub struct StyledTextBox<'a, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    /// A [`TextBox`] that has an associated [`TextBoxStyle`].
    ///
    /// [`TextBox`]: ../struct.TextBox.html
    /// [`TextBoxStyle`]: struct.TextBoxStyle.html
    pub text_box: TextBox<'a>,

    /// The style of the [`TextBox`].
    ///
    /// [`TextBox`]: ../struct.TextBox.html
    pub style: TextBoxStyle<C, F, A>,
}

impl<'a, C, F, A> Drawable<C> for &'a StyledTextBox<'a, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
    StyledTextBoxIterator<'a, C, F, A>: Iterator<Item = Pixel<C>>,
    StyledTextBox<'a, C, F, A>: StateFactory,
{
    #[inline]
    fn draw<D: DrawTarget<C>>(self, display: &mut D) -> Result<(), D::Error> {
        display.draw_iter(StyledTextBoxIterator::new(self))
    }
}

impl<C, F, A> Transform for StyledTextBox<'_, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
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

impl<C, F, A> Dimensions for StyledTextBox<'_, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
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
    use crate::{alignment::*, style::builder::TextBoxStyleBuilder};
    use embedded_graphics::{
        fonts::{Font, Font6x8},
        pixelcolor::BinaryColor,
    };

    #[test]
    fn test_measure_height() {
        let data = [
            ("", 0, 0),
            ("word", 4 * 6, 8), // exact fit into 1 line
            ("word", 4 * 6 - 1, 16),
            ("word", 2 * 6, 16), // exact fit into 2 lines
            ("word\nnext", 50, 16),
            ("verylongword", 50, 16),
            ("some verylongword", 50, 24),
            ("1 23456 12345 61234 561", 36, 40),
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
    fn test_measure_height_of_left_aligned_counts_space() {
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .build();

        let text = "    Word      ";
        let width = 6 * 6;
        let expected_height = 3 * Font6x8::CHARACTER_SIZE.height;

        let height = textbox_style.measure_text_height(text, width);
        assert_eq!(
            height, expected_height,
            "Height of \"{}\" is {} but is expected to be {}",
            text, height, expected_height
        );
    }

    #[test]
    fn test_measure_height_of_center_aligned_ignores_space() {
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .build();

        let text = "    Word      ";
        let width = 6 * 6;
        #[allow(clippy::identity_op)]
        let expected_height = 1 * Font6x8::CHARACTER_SIZE.height;

        let height = textbox_style.measure_text_height(text, width);
        assert_eq!(
            height, expected_height,
            "Height of \"{}\" is {} but is expected to be {}",
            text, height, expected_height
        );
    }
}
