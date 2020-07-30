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

    /// Measures text height when rendered using a given width.
    #[inline]
    #[must_use]
    pub fn measure_text_height(&self, text: &str, max_width: u32) -> u32 {
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
                                        total_width = 0;
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

                        Token::NewLine => {}
                    }
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
        for (text, width, expected_height) in data.iter() {
            let height = textbox_style.measure_text_height(text, *width);
            assert_eq!(
                height, *expected_height,
                "Height of \"{}\" is {} but is expected to be {}",
                text, height, expected_height
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
