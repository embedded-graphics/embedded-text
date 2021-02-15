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
//! To apply a style, call [`TextBox::into_styled`].
//!
//! In-band text styling using ANSI escape codes
//! ============================================
//!
//! Sometimes you need more flexibility than what a single style object can provide, like changing
//! MonoFont color for a specific word in the text. `embedded-text` supports this use case by using a
//! subset of the standard [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code).
//! These are special character sequences you can use *in the text* to change the MonoFont style of the
//! text itself. This documentation does not aim to provide a full specification of all the ANSI
//! escape codes, only describes the supported subset.
//!
//! > *Note:* if `embedded-text` fails to parse an escape sequence, it will ignore the `\x1b` character
//! and display the rest as normal text.
//!
//! All escape sequences start with the `\x1b[` sequence, where `\x1b` is the ASCII `escape`
//! character. `embedded-text` supports a subset of the `SGR` parameters, which are numeric codes
//! with specific functions, followed by a number of parameters and end with the `m` character.
//!
//! Currently, `embedded-text` supports changing the text and background colors. To do this, you
//! have the following options:
//!
//! Standard color codes
//! --------------------
//!
//! <style>
//! .ansi_color {
//!     display: block;
//!     text-align: center;
//!     color: white;
//! }
//! </style>
//!
//! The standard color codes option is the simplest, and least flexible way to set color.
//!
//! | Color name          | Text color | Background color | RGB888                                                                                          |
//! |---------------------|------------|------------------|-------------------------------------------------------------------------------------------------|
//! | Black               | `\x1b[30m` | `\x1b[40m`       | <span class="ansi_color" style="background: rgb(12,12,12);"> 12,12,12 </span>                     |
//! | Red                 | `\x1b[31m` | `\x1b[41m`       | <span class="ansi_color" style="background: rgb(197,15,31);"> 197,15,31 </span>                   |
//! | Green               | `\x1b[32m` | `\x1b[42m`       | <span class="ansi_color" style="background: rgb(19,161,14);"> 19,161,14 </span>                   |
//! | Yellow              | `\x1b[33m` | `\x1b[43m`       | <span class="ansi_color" style="background: rgb(193,156,0);"> 193,156,0 </span>                   |
//! | Blue                | `\x1b[34m` | `\x1b[44m`       | <span class="ansi_color" style="background: rgb(0,55,218);"> 0,55,218 </span>                     |
//! | Magenta             | `\x1b[35m` | `\x1b[45m`       | <span class="ansi_color" style="background: rgb(136,23,152);"> 136,23,152 </span>                 |
//! | Cyan                | `\x1b[36m` | `\x1b[46m`       | <span class="ansi_color" style="background: rgb(58,150,221);"> 58,150,221 </span>                 |
//! | White               | `\x1b[37m` | `\x1b[47m`       | <span class="ansi_color" style="background: rgb(204,204,204); color: black;"> 204,204,204 </span> |
//! | Gray (Bright Black) | `\x1b[90m` | `\x1b[100m`      | <span class="ansi_color" style="background: rgb(118,118,118); color: black;"> 118,118,118 </span> |
//! | Bright Red          | `\x1b[91m` | `\x1b[101m`      | <span class="ansi_color" style="background: rgb(231,72,86);"> 231,72,86 </span>                   |
//! | Bright Green        | `\x1b[92m` | `\x1b[102m`      | <span class="ansi_color" style="background: rgb(22,198,12); color: black;"> 22,198,12 </span>     |
//! | Bright Yellow       | `\x1b[93m` | `\x1b[103m`      | <span class="ansi_color" style="background: rgb(249,241,165); color: black;"> 249,241,165 </span> |
//! | Bright Blue         | `\x1b[94m` | `\x1b[104m`      | <span class="ansi_color" style="background: rgb(59,120,255);"> 59,120,255 </span>                 |
//! | Bright Magenta      | `\x1b[95m` | `\x1b[105m`      | <span class="ansi_color" style="background: rgb(180,0,158);"> 180,0,158 </span>                   |
//! | Bright Cyan         | `\x1b[96m` | `\x1b[106m`      | <span class="ansi_color" style="background: rgb(97,214,214); color: black;"> 97,214,214 </span>   |
//! | Bright White        | `\x1b[97m` | `\x1b[107m`      | <span class="ansi_color" style="background: rgb(242,242,242); color: black;"> 242,242,242 </span> |
//!
//! 8 bit colors
//! ------------
//!
//! 8 bit colors are in the form of either `\x1b[38;5;<n>m` (text color) or `\x1b[48;5;<n>m`
//! (background color) sequence. Here, `<n>` marks a parameter that determines the color. `<n>` can
//! have the following values:
//!
//! * 0-15: standard colors in the order of the above table.
//!   For example, `\x1b[38;5;12m` is the `Bright Blue` color.
//! * 16-231: 6 × 6 × 6 cube (216 colors): `16 + 36 × r + 6 × g + b (0 ≤ r, g, b ≤ 5)`
//! * 232-255: grayscale from black to white
//!
//! 24 bit colors
//! -------------
//!
//! 8 bit colors are in the form of either `\x1b[38;2;<r>;<g>;<b>m` (text color) or
//! `\x1b[48;2;<r>;<g>;<b>m` (background color) sequence. Here, `<r>`, `<g>` and `<b>` can take any
//! value between `0` and `255`.
//!
//! Color values on color spaces other than `Rgb888`
//! ------------------------------------------------
//!
//! By default, `embedded-text` uses the following color types provided by `embedded-graphics`:
//!
//!  * `Rgb888`
//!  * `Rgb565`
//!  * `Rgb555`
//!  * `BinaryColor`
//!
//! Internally, all ANSI color sequences are turned into the [`Rgb`] type, which can be converted
//! to the above types. The resulting color will be the closest match to what you specify.
//!
//! If you wish to use a different color type, you'll need to implement `From<Rgb>` for your color
//! type and write the conversion yourself.
//!
//! Color values on monochrome displays
//! -----------------------------------
//!
//! Monochrome displays use the `BinaryColor` color which can have two values: `On` or `Off`.
//! You can still use the ANSI colors with the following considerations:
//!
//!  * If the value of all three color channels are greater than `127`, the resulting color in `On`
//!  * Otherwise, the color is converted to `Off`.
//!
//! Other text styling options
//! --------------------------
//!
//! The following SGR sequences are supported:
//!
//!  * `\x1b[0m`: Reset everything
//!  * `\x1b[4m`: Underlined text
//!  * `\x1b[24m`: Turn off text underline
//!  * `\x1b[9m`: Crossed out/strikethrough text
//!  * `\x1b[29m`: Turn off strikethrough
//!  * `\x1b[39m`: Reset text color
//!  * `\x1b[49m`: Reset background color
//!
//! Reset style options to default
//! ------------------------------
//!
//! `embedded-text` supports the `Reset all` (`\x1b[0m`), `Default text color` (`\x1b[39m`) and
//! `Default background color` (`\x1b[49m`) codes. These codes can be used to reset colors to
//! *transparent* (i.e. no pixels drawn for text or background).
//!
//! In addition, `Reset all` turns off the underlined and crossed out styles.
//!
//! Other supported ANSI escape codes
//! ---------------------------------
//!
//! Besides changing text style, you can also move the cursor using ANSI escape codes!
//! You have the following options:
//!
//!  - Move the cursor forward `<n>` characters: `\x1b[<n>C`. This command will stop at the end of
//!    line, so you can use it to simulate a highlighted line, for example.
//!    *Note:* Moving the cursor *forward* fills the line with the background color. If you want to
//!    avoid this, make sure to reset the background color before moving the cursor!
//!  - Move the cursor backward `<n>` characters: `\x1b[<n>D`. This command will stop at the start
//!    of line.
//!
//! [`Rgb`]: ./color/struct.Rgb.html
//! [`TextBox`]: ../struct.TextBox.html
//! [`TextBoxStyle`]: struct.TextBoxStyle.html
//! [`TextBoxStyleBuilder`]: builder/struct.TextBoxStyleBuilder.html
//! [`TextBoxStyleBuilder::new`]: builder/struct.TextBoxStyleBuilder.html#method.new
//! [`TextBox::into_styled`]: ../struct.TextBox.html#method.into_styled

pub mod builder;
pub mod color;
pub mod height_mode;
pub mod vertical_overdraw;

use crate::{
    alignment::HorizontalTextAlignment,
    parser::{Parser, Token, SPEC_CHAR_NBSP},
    rendering::{
        cursor::Cursor,
        line_iter::{LineElementParser, RenderElement},
        space_config::UniformSpaceConfig,
    },
    utils::str_width,
};
use color::Rgb;
use embedded_graphics::{
    geometry::{Point, Size},
    primitives::Rectangle,
    text::{CharacterStyle, TextRenderer},
};

pub use self::builder::TextBoxStyleBuilder;

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

impl Default for TabSize {
    #[inline]
    fn default() -> Self {
        Self::Spaces(4)
    }
}

impl TabSize {
    /// Calculate the rendered with of the next tab
    #[inline]
    pub fn into_pixels(self, renderer: &impl TextRenderer) -> u32 {
        match self {
            TabSize::Pixels(px) => px as u32,
            TabSize::Spaces(n) => n as u32 * str_width(renderer, " "),
        }
    }
}

/// Placeholder character style that needs to be changed in order to draw a `StyledTextBox`.
#[derive(Copy, Clone, Debug)]
pub struct UndefinedCharacterStyle;

/// Styling options of a [`TextBox`].
///
/// `TextBoxStyle` contains the font, foreground and background `PixelColor`, line spacing,
/// [`HeightMode`], [`HorizontalTextAlignment`] and [`VerticalTextAlignment`] information necessary
/// to draw a [`TextBox`].
///
/// To construct a new `TextBoxStyle` object, use the [`new`] or [`from_text_style`] methods or
/// the [`TextBoxStyleBuilder`] object.
///
/// [`TextBox`]: ../struct.TextBox.html
/// [`HeightMode`]: ./height_mode/trait.HeightMode.html
/// [`HorizontalTextAlignment`]: ../alignment/trait.HorizontalTextAlignment.html
/// [`VerticalTextAlignment`]: ../alignment/trait.VerticalTextAlignment.html
/// [`TextBoxStyleBuilder`]: builder/struct.TextBoxStyleBuilder.html
/// [`new`]: #method.new
/// [`from_text_style`]: #method.from_text_style
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TextBoxStyle<F, A, V, H> {
    /// Character style used to measure and draw text.
    pub character_style: F,

    /// Horizontal text alignment.
    pub alignment: A,

    /// Vertical text alignment.
    pub vertical_alignment: V,

    /// The height behaviour
    pub height_mode: H,

    /// Desired space between lines, in pixels
    pub line_spacing: i32,

    /// Desired column width for tabs
    pub tab_size: TabSize,
}

/// Information about a line.
pub struct LineMeasurement {
    /// Width in pixels, using the default space width returned by the text renderer.
    pub width: u32,

    /// The number of space characters in the line.
    pub space_count: u32,

    /// Whether this line is the last line of a paragraph.
    pub last_line: bool,
}

impl<F, A, V, H> TextBoxStyle<F, A, V, H>
where
    F: TextRenderer + CharacterStyle,
    <F as CharacterStyle>::Color: From<Rgb>,
    A: HorizontalTextAlignment,
{
    /// Measure the width and count spaces in a single line of text.
    ///
    /// Returns (width, rendered space count, carried token)
    ///
    /// Instead of peeking ahead when processing tokens, this function advances the parser before
    /// processing a token. If a token opens a new line, it will be returned as the carried token.
    /// If the carried token is `None`, the parser has finished processing the text.
    #[inline]
    #[must_use]
    pub(crate) fn measure_line<'a>(
        &self,
        parser: &mut Parser<'a>,
        carried_token: &mut Option<Token<'a>>,
        max_line_width: u32,
    ) -> LineMeasurement {
        let cursor: Cursor = Cursor::new(
            Rectangle::new(
                Point::zero(),
                Size::new(max_line_width, self.character_style.line_height()),
            ),
            self.character_style.line_height(),
            self.line_spacing,
            self.tab_size.into_pixels(&self.character_style),
        );

        let iter = LineElementParser::<'_, '_, _, _, A>::new(
            parser,
            cursor,
            UniformSpaceConfig::new(&self.character_style),
            carried_token,
            |s| str_width(&self.character_style, s),
        );

        let mut current_width = 0;
        let mut last_spaces = 0;
        let mut last_spaces_width = 0;
        let mut total_spaces = 0;

        for token in iter {
            match token {
                RenderElement::Space(width, count) => {
                    if A::ENDING_SPACES {
                        // only track width if spaces are rendered at the end of a line
                        current_width += width;

                        // in this case, count all spaces
                        total_spaces += count;
                    } else {
                        // ... otherwise save the number of spaces and it will be tracked with
                        // the next printed character, or it will be discarded
                        last_spaces = total_spaces + count;
                        last_spaces_width = width;
                    }
                }

                RenderElement::PrintedCharacters(s) => {
                    // must not rely on cursor position because it can get reset to 0 at line breaks
                    current_width += str_width(&self.character_style, s);

                    if !A::ENDING_SPACES {
                        // if ENDING_SPACES is true, spaces have already been counted and
                        // last_spaces is 0
                        current_width += last_spaces_width;
                        last_spaces_width = 0;
                        total_spaces = last_spaces;
                    }
                    total_spaces += s.chars().filter(|c| *c == SPEC_CHAR_NBSP).count() as u32;
                }

                #[cfg(feature = "ansi")]
                RenderElement::MoveCursor(delta) => {
                    current_width = (current_width as i32 + delta)
                        .max(0)
                        .min(max_line_width as i32) as u32
                }

                // Ignore color changes
                #[cfg(feature = "ansi")]
                RenderElement::Sgr(_) => {}
            }
        }

        LineMeasurement {
            width: current_width as u32,
            space_count: total_spaces,
            last_line: carried_token.is_none() || *carried_token == Some(Token::NewLine),
        }
    }

    /// Measures text height when rendered using a given width.
    ///
    /// # Example: measure height of text when rendered using a 6x8 MonoFont and 72px width.
    ///
    /// ```rust
    /// # use embedded_text::style::builder::TextBoxStyleBuilder;
    /// # use embedded_graphics::{
    /// #     mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
    /// #     pixelcolor::BinaryColor,
    /// # };
    /// #
    /// let character_style = MonoTextStyleBuilder::new()
    ///     .font(Font6x9)
    ///     .text_color(BinaryColor::On)
    ///     .build();
    /// let style = TextBoxStyleBuilder::new()
    ///     .character_style(character_style)
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
    /// assert_eq!(7 * 9, height);
    /// ```
    #[inline]
    #[must_use]
    pub fn measure_text_height(&self, text: &str, max_width: u32) -> u32 {
        let mut n_lines = 0_i32;
        let mut parser = Parser::parse(text);
        let mut carry = None;
        let line_height = self.character_style.line_height() as i32;

        // while let (w, _, Some(t)) = ...
        loop {
            let mut t = carry.clone();
            let lm = self.measure_line(&mut parser, &mut t, max_width);

            if (lm.width != 0 || t.is_some()) && carry != Some(Token::CarriageReturn) {
                // something was in this line, increment height
                // if last carried token was a carriage return, we already counted the height
                n_lines += 1;
            }

            if t.is_none() {
                return (n_lines * line_height + n_lines.saturating_sub(1) * self.line_spacing)
                    as u32;
            }

            carry = t;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{alignment::*, parser::Parser, style::builder::TextBoxStyleBuilder};
    use embedded_graphics::{
        mono_font::{ascii::Font6x9, MonoFont, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        text::TextRenderer,
    };

    #[test]
    fn no_infinite_loop() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let _ = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .build()
            .measure_text_height("a", 5);
    }

    #[test]
    fn test_measure_height() {
        let data = [
            ("", 0, 0),
            (" ", 0, 1),
            (" ", 5, 1),
            (" ", 6, 1),
            ("\n", 6, 1),
            ("\n ", 6, 2),
            ("word", 4 * 6, 1), // exact fit into 1 line
            ("word", 4 * 6 - 1, 2),
            ("word", 2 * 6, 2),      // exact fit into 2 lines
            ("word word", 4 * 6, 2), // exact fit into 2 lines
            ("word\n", 2 * 6, 2),
            ("word\nnext", 50, 2),
            ("word\n\nnext", 50, 3),
            ("word\n  \nnext", 50, 3),
            ("verylongword", 50, 2),
            ("some verylongword", 50, 3),
            ("1 23456 12345 61234 561", 36, 5),
            ("    Word      ", 36, 3),
            ("\rcr", 36, 1),
            ("Longer\r", 36, 1),
            ("Longer\rnowrap", 36, 1),
        ];

        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .build();

        for (i, (text, width, expected_n_lines)) in data.iter().enumerate() {
            let height = style.measure_text_height(text, *width);
            let expected_height = *expected_n_lines * character_style.line_height();
            assert_eq!(
                height, expected_height,
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
            ("\n ", 6, 1),
            ("word\n", 2 * 6, 2),
            ("word\n  \nnext", 50, 3),
            ("    Word      ", 36, 1),
        ];

        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(CenterAligned)
            .build();

        for (i, (text, width, expected_n_lines)) in data.iter().enumerate() {
            let height = style.measure_text_height(text, *width);
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
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(CenterAligned)
            .build();

        let mut text = Parser::parse("123 45 67");

        let lm = style.measure_line(&mut text, &mut None, 6 * Font6x9::CHARACTER_SIZE.width);
        assert_eq!(lm.width, 6 * Font6x9::CHARACTER_SIZE.width);
        assert_eq!(lm.space_count, 1);
    }

    #[test]
    fn test_measure_line_counts_nbsp() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(CenterAligned)
            .build();

        let mut text = Parser::parse("123\u{A0}45");

        let lm = style.measure_line(&mut text, &mut None, 5 * Font6x9::CHARACTER_SIZE.width);
        assert_eq!(lm.width, 5 * Font6x9::CHARACTER_SIZE.width);
        assert_eq!(lm.space_count, 1);
    }

    #[test]
    fn test_measure_height_nbsp() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(CenterAligned)
            .build();
        let text = "123\u{A0}45 123";

        let height = style.measure_text_height(text, 5 * Font6x9::CHARACTER_SIZE.width);
        assert_eq!(height, 2 * character_style.line_height());

        // bug discovered while using the interactive example
        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(LeftAligned)
            .build();

        let text = "embedded-text also\u{A0}supports non-breaking spaces.";

        let height = style.measure_text_height(text, 79);
        assert_eq!(height, 4 * character_style.line_height());
    }

    #[test]
    fn height_with_line_spacing() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .line_spacing(2)
            .build();

        let height = style.measure_text_height(
            "Lorem Ipsum is simply dummy text of the printing and typesetting industry.",
            72,
        );

        assert_eq!(height, 7 * character_style.line_height() + 6 * 2);
    }

    #[test]
    fn soft_hyphenated_line_width_includes_hyphen_width() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .line_spacing(2)
            .build();

        let lm = style.measure_line(&mut Parser::parse("soft\u{AD}hyphen"), &mut None, 50);

        assert_eq!(lm.width, 30);
    }
}
