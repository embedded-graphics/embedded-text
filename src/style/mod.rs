//! `TextBox` styling.
//!
//! Style objects and why you need them
//! ===================================
//!
//! By itself, a [`TextBox`] does not contain the information necessary to draw it on a display.
//! This information is called "style" and it is contained in [`TextBoxStyle`] objects.
//!
//! The recommended (and most flexible) way of constructing a style object is using the
//! [`TextBoxStyleBuilder`] builder object. The least amount of information necessary to create a
//! text style is the `MonoFont` used to render the text, so you'll need to specify this when you call
//! [`TextBoxStyleBuilder::new`].
//! You can then chain together various builder methods to customize MonoFont rendering.
//!
//! See the [`TextBoxStyleBuilder`] for more information on what styling options you have.
//!
//! [`TextBox`]: crate::TextBox

mod builder;
mod height_mode;
mod vertical_overdraw;

use core::convert::Infallible;

use crate::{
    alignment::{HorizontalAlignment, VerticalAlignment},
    parser::Parser,
    plugin::{NoPlugin, PluginMarker as Plugin, PluginWrapper, ProcessingState},
    rendering::{
        cursor::LineCursor,
        line_iter::{ElementHandler, LineElementParser, LineEndType},
        space_config::SpaceConfig,
    },
    utils::str_width,
};
use az::SaturatingAs;
use embedded_graphics::{
    pixelcolor::Rgb888,
    text::{renderer::TextRenderer, LineHeight},
};

pub use self::{
    builder::TextBoxStyleBuilder, height_mode::HeightMode, vertical_overdraw::VerticalOverdraw,
};

/// Tab size helper
///
/// This type makes it more obvious what unit is used to define the width of tabs.
/// The default tab size is 4 spaces.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TabSize {
    /// Tab width as a number of pixels.
    Pixels(u16),

    /// Tab width as a number of space characters.
    Spaces(u16),
}

impl TabSize {
    /// Returns the default tab size, which is 4 spaces.
    #[inline]
    pub const fn default() -> Self {
        Self::Spaces(4)
    }

    /// Calculate the rendered with of the next tab
    #[inline]
    pub(crate) fn into_pixels(self, renderer: &impl TextRenderer) -> u32 {
        match self {
            TabSize::Pixels(px) => px as u32,
            TabSize::Spaces(n) => n as u32 * str_width(renderer, " "),
        }
    }
}

/// Styling options of a [`TextBox`].
///
/// `TextBoxStyle` contains the font, foreground and background `PixelColor`, line spacing,
/// [`HeightMode`], [`HorizontalAlignment`] and [`VerticalAlignment`] information necessary
/// to draw a [`TextBox`].
///
/// To construct a new `TextBoxStyle` object, use the [`TextBoxStyle::default`] method or
/// the [`TextBoxStyleBuilder`] object.
///
/// [`TextBox`]: crate::TextBox
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
#[must_use]
pub struct TextBoxStyle {
    /// Horizontal text alignment.
    pub alignment: HorizontalAlignment,

    /// Vertical text alignment.
    pub vertical_alignment: VerticalAlignment,

    /// The height behaviour.
    pub height_mode: HeightMode,

    /// Line height.
    pub line_height: LineHeight,

    /// Paragraph spacing.
    pub paragraph_spacing: u32,

    /// Desired column width for tabs
    pub tab_size: TabSize,

    /// True to render leading spaces
    pub leading_spaces: bool,

    /// True to render trailing spaces
    pub trailing_spaces: bool,
}

impl TextBoxStyle {
    /// Creates a new text box style object with default settings.
    #[inline]
    pub const fn default() -> Self {
        TextBoxStyleBuilder::new().build()
    }

    /// Creates a new text box style with the given alignment.
    #[inline]
    pub const fn with_alignment(alignment: HorizontalAlignment) -> TextBoxStyle {
        TextBoxStyleBuilder::new().alignment(alignment).build()
    }

    /// Creates a new text box style with the given vertical alignment.
    #[inline]
    pub const fn with_vertical_alignment(alignment: VerticalAlignment) -> TextBoxStyle {
        TextBoxStyleBuilder::new()
            .vertical_alignment(alignment)
            .build()
    }
}

/// Information about a line.
#[derive(Debug, Copy, Clone)]
#[must_use]
pub(crate) struct LineMeasurement {
    /// Maximum line width in pixels.
    pub max_line_width: u32,

    /// Width in pixels, using the default space width returned by the text renderer.
    pub width: u32,

    /// Whether this line is the last line of a paragraph.
    pub last_line: bool,

    /// Whether this line ended with a \r.
    pub line_end_type: LineEndType,

    /// Number of spaces in the current line.
    pub space_count: u32,
}

struct MeasureLineElementHandler<'a, S> {
    style: &'a S,
    right: u32,
    max_line_width: u32,
    pos: u32,
    space_count: u32,
    partial_space_count: u32,
    trailing_spaces: bool,
}

impl<'a, S: TextRenderer> ElementHandler for MeasureLineElementHandler<'a, S> {
    type Error = Infallible;
    type Color = S::Color;

    fn measure(&self, st: &str) -> u32 {
        str_width(self.style, st)
    }

    fn whitespace(&mut self, _st: &str, count: u32, width: u32) -> Result<(), Self::Error> {
        self.pos += width;
        self.partial_space_count += count;

        if self.trailing_spaces {
            self.space_count = self.partial_space_count;
            self.right = self.pos;
        }

        Ok(())
    }

    fn printed_characters(&mut self, _: &str, width: u32) -> Result<(), Self::Error> {
        self.right = self.right.max(self.pos + width);
        self.pos += width;
        self.space_count = self.partial_space_count;
        Ok(())
    }

    fn move_cursor(&mut self, by: i32) -> Result<(), Self::Error> {
        self.pos = (self.pos.saturating_as::<i32>() + by)
            .max(0)
            .min(self.max_line_width.saturating_as()) as u32;

        Ok(())
    }
}

impl TextBoxStyle {
    /// Measure the width and count spaces in a single line of text.
    ///
    /// Returns (width, rendered space count, carried token)
    ///
    /// Instead of peeking ahead when processing tokens, this function advances the parser before
    /// processing a token. If a token opens a new line, it will be returned as the carried token.
    /// If the carried token is `None`, the parser has finished processing the text.
    #[inline]
    pub(crate) fn measure_line<'a, S, M>(
        &self,
        plugin: &PluginWrapper<'a, M, S::Color>,
        character_style: &S,
        parser: &mut Parser<'a, S::Color>,
        max_line_width: u32,
    ) -> LineMeasurement
    where
        S: TextRenderer,
        M: Plugin<'a, S::Color>,
        S::Color: From<Rgb888>,
    {
        let cursor = LineCursor::new(max_line_width, self.tab_size.into_pixels(character_style));

        let mut iter = LineElementParser::new(
            parser,
            plugin,
            cursor,
            SpaceConfig::new(str_width(character_style, " "), None),
            self,
        );

        let mut handler = MeasureLineElementHandler {
            style: character_style,
            right: 0,
            pos: 0,
            max_line_width,
            space_count: 0,
            partial_space_count: 0,
            trailing_spaces: self.trailing_spaces,
        };
        let last_token = iter.process(&mut handler).unwrap();

        LineMeasurement {
            max_line_width,
            width: handler.right,
            space_count: handler.space_count,
            last_line: matches!(last_token, LineEndType::NewLine | LineEndType::EndOfText),
            line_end_type: last_token,
        }
    }

    /// Measures text height when rendered using a given width.
    ///
    /// # Example: measure height of text when rendered using a 6x8 MonoFont and 72px width.
    ///
    /// ```rust
    /// # use embedded_text::style::TextBoxStyleBuilder;
    /// # use embedded_graphics::{
    /// #     mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    /// #     pixelcolor::BinaryColor,
    /// # };
    /// #
    /// let character_style = MonoTextStyleBuilder::new()
    ///     .font(&FONT_6X9)
    ///     .text_color(BinaryColor::On)
    ///     .build();
    /// let style = TextBoxStyleBuilder::new().build();
    ///
    /// let height = style.measure_text_height(
    ///     &character_style,
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
    /// assert_eq!(7 * 9, height);
    /// ```
    #[inline]
    #[must_use]
    pub fn measure_text_height<S>(&self, character_style: &S, text: &str, max_width: u32) -> u32
    where
        S: TextRenderer,
        S::Color: From<Rgb888>,
    {
        let plugin = PluginWrapper::new(NoPlugin::new());
        self.measure_text_height_impl(plugin, character_style, text, max_width)
    }

    pub(crate) fn measure_text_height_impl<'a, S, M>(
        &self,
        plugin: PluginWrapper<'a, M, S::Color>,
        character_style: &S,
        text: &'a str,
        max_width: u32,
    ) -> u32
    where
        S: TextRenderer,
        M: Plugin<'a, S::Color>,
        S::Color: From<Rgb888>,
    {
        let mut parser = Parser::parse(text);
        let mut closed_paragraphs: u32 = 0;
        let line_height = self.line_height.to_absolute(character_style.line_height());
        let last_line_height = character_style.line_height();
        let mut height = last_line_height;
        let mut paragraph_ended = false;

        plugin.set_state(ProcessingState::Measure);

        let mut prev_end = LineEndType::EndOfText;

        loop {
            plugin.new_line();
            let lm = self.measure_line(&plugin, character_style, &mut parser, max_width);

            if paragraph_ended {
                closed_paragraphs += 1;
            }
            paragraph_ended = lm.last_line;

            if prev_end == LineEndType::LineBreak && lm.width != 0 {
                height += line_height;
            }

            match lm.line_end_type {
                LineEndType::CarriageReturn => {}
                LineEndType::LineBreak => {}
                LineEndType::NewLine => {
                    height += line_height;
                }
                LineEndType::EndOfText => {
                    return height + closed_paragraphs * self.paragraph_spacing;
                }
            }
            prev_end = lm.line_end_type;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        alignment::*,
        parser::Parser,
        plugin::{NoPlugin, PluginWrapper},
        style::{builder::TextBoxStyleBuilder, TextBoxStyle},
    };
    use embedded_graphics::{
        mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        text::{renderer::TextRenderer, LineHeight},
    };

    #[test]
    fn no_infinite_loop() {
        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let _ = TextBoxStyleBuilder::new()
            .build()
            .measure_text_height(&character_style, "a", 5);
    }

    #[test]
    fn test_measure_height() {
        let data = [
            // (text; max width in characters; number of expected lines)
            ("", 0, 1),
            (" ", 6, 1),
            ("\r", 6, 1),
            ("\n", 6, 2),
            ("\n ", 6, 2),
            ("word", 4 * 6, 1),   // exact fit into 1 line
            ("word\n", 4 * 6, 2), // newline
            ("word", 4 * 6 - 1, 2),
            ("word", 2 * 6, 2),      // exact fit into 2 lines
            ("word word", 4 * 6, 2), // exact fit into 2 lines
            ("word\n", 2 * 6, 3),
            ("word\nnext", 50, 2),
            ("word\n\nnext", 50, 3),
            ("word\n  \nnext", 50, 3),
            ("verylongword", 50, 2),
            ("some verylongword", 50, 3),
            ("1 23456 12345 61234 561", 36, 5),
            ("    Word      ", 36, 2),
            ("\rcr", 36, 1),
            ("cr\r", 36, 1),
            ("cr\rcr", 36, 1),
            ("Longer\r", 36, 1),
            ("Longer\rnowrap", 36, 1),
        ];

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyle::default();

        for (i, (text, width, expected_n_lines)) in data.iter().enumerate() {
            let height = style.measure_text_height(&character_style, text, *width);
            let expected_height = *expected_n_lines * character_style.line_height();
            assert_eq!(
                height,
                expected_height,
                r#"#{}: Height of "{}" is {} but is expected to be {}"#,
                i,
                text.replace('\r', "\\r").replace('\n', "\\n"),
                height,
                expected_height
            );
        }
    }

    #[test]
    fn test_measure_height_ignored_spaces() {
        let data = [
            ("", 0, 1),
            (" ", 0, 1),
            (" ", 6, 1),
            ("\n ", 6, 2),
            ("word\n  \nnext", 50, 3),
            ("    Word      ", 36, 1),
        ];

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Center)
            .build();

        for (i, (text, width, expected_n_lines)) in data.iter().enumerate() {
            let height = style.measure_text_height(&character_style, text, *width);
            let expected_height = *expected_n_lines * character_style.line_height();
            assert_eq!(
                height, expected_height,
                r#"#{}: Height of "{}" is {} but is expected to be {}"#,
                i, text, height, expected_height
            );
        }
    }

    #[test]
    fn test_measure_line() {
        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Center)
            .build();

        let mut text = Parser::parse("123 45 67");

        let mut plugin = PluginWrapper::new(NoPlugin::new());
        let lm = style.measure_line(
            &mut plugin,
            &character_style,
            &mut text,
            6 * FONT_6X9.character_size.width,
        );
        assert_eq!(lm.width, 6 * FONT_6X9.character_size.width);
    }

    #[test]
    fn test_measure_line_counts_nbsp() {
        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Center)
            .build();

        let mut text = Parser::parse("123\u{A0}45");

        let mut plugin = PluginWrapper::new(NoPlugin::new());
        let lm = style.measure_line(
            &mut plugin,
            &character_style,
            &mut text,
            5 * FONT_6X9.character_size.width,
        );
        assert_eq!(lm.width, 5 * FONT_6X9.character_size.width);
    }

    #[test]
    fn test_measure_height_nbsp() {
        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Center)
            .build();
        let text = "123\u{A0}45 123";

        let height =
            style.measure_text_height(&character_style, text, 5 * FONT_6X9.character_size.width);
        assert_eq!(height, 2 * character_style.line_height());

        // bug discovered while using the interactive example
        let style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Left)
            .build();

        let text = "embedded-text also\u{A0}supports non-breaking spaces.";

        let height = style.measure_text_height(&character_style, text, 79);
        assert_eq!(height, 4 * character_style.line_height());
    }

    #[test]
    fn height_with_line_spacing() {
        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .line_height(LineHeight::Pixels(11))
            .build();

        let height = style.measure_text_height(
            &character_style,
            "Lorem Ipsum is simply dummy text of the printing and typesetting industry.",
            72,
        );

        assert_eq!(height, 6 * 11 + 9);
    }

    #[test]
    fn soft_hyphenated_line_width_includes_hyphen_width() {
        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .line_height(LineHeight::Pixels(11))
            .build();

        let mut plugin = PluginWrapper::new(NoPlugin::new());
        let lm = style.measure_line(
            &mut plugin,
            &character_style,
            &mut Parser::parse("soft\u{AD}hyphen"),
            50,
        );

        assert_eq!(lm.width, 30);
    }
}
