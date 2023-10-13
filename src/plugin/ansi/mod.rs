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
//! Supported color types
//! ---------------------
//!
//! `embedded-text` supports all color types that are included in `embedded-graphics`.
//!
//! If you wish to use a different color type, the types needs to implement `From<Rgb888>`.
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

use ansi_parser::AnsiSequence;
use embedded_graphics::{pixelcolor::Rgb888, prelude::PixelColor};

use crate::{
    parser::Token,
    plugin::{ansi::utils::try_parse_sgr, Plugin},
};

mod utils;

/// Ansi sequence parser plugin.
#[derive(Clone)]
pub struct Ansi<'a, C: PixelColor> {
    carry: Option<Token<'a, C>>,
}

impl<C: PixelColor> Ansi<'_, C> {
    /// Returns a new plugin object.
    #[inline]
    pub const fn new() -> Self {
        Self { carry: None }
    }
}

impl<'a, C: PixelColor + From<Rgb888>> Plugin<'a, C> for Ansi<'a, C> {
    fn next_token(
        &mut self,
        mut next_token: impl FnMut() -> Option<Token<'a, C>>,
    ) -> Option<Token<'a, C>> {
        let token = if let Some(token) = self.carry.take() {
            Some(token)
        } else {
            next_token()
        };

        if let Some(Token::Word(text)) = token {
            let mut chars = text.char_indices();

            match chars.find(|(_, c)| *c == '\u{1b}') {
                Some((0, _)) => match ansi_parser::parse_escape(text) {
                    Ok((string, output)) => {
                        let new_token = match output {
                            AnsiSequence::CursorForward(chars) => {
                                self.carry = Some(Token::Word(string));
                                Token::MoveCursor {
                                    chars: chars as i32,
                                    draw_background: true,
                                }
                            }
                            AnsiSequence::CursorBackward(chars) => {
                                self.carry = Some(Token::Word(string));
                                Token::MoveCursor {
                                    chars: -(chars as i32),
                                    draw_background: true,
                                }
                            }
                            AnsiSequence::SetGraphicsMode(sgr) => try_parse_sgr(&sgr)
                                .map(|sgr| {
                                    self.carry = Some(Token::Word(string));
                                    Token::ChangeTextStyle(sgr.into())
                                })
                                .or_else(|| self.next_token(next_token))?,

                            _ => self.next_token(next_token)?,
                        };

                        Some(new_token)
                    }
                    Err(_) => {
                        self.carry = Some(Token::Word(chars.as_str()));
                        Some(Token::Word("\u{1b}"))
                    }
                },

                Some((idx, _)) => {
                    // Escape character is not the first one. Strip and return the prefix.
                    let (pre, rem) = text.split_at(idx);

                    self.carry = Some(Token::Word(rem));
                    Some(Token::Word(pre))
                }

                None => {
                    // No escape character.
                    Some(Token::Word(text))
                }
            }
        } else {
            // Not a word token.
            token
        }
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        mock_display::MockDisplay,
        mono_font::{
            ascii::{FONT_6X10, FONT_6X9},
            MonoTextStyle, MonoTextStyleBuilder,
        },
        pixelcolor::{BinaryColor, Rgb888},
        prelude::{Point, Size},
        primitives::Rectangle,
        Drawable,
    };

    use crate::{
        alignment::{HorizontalAlignment, VerticalAlignment},
        parser::{ChangeTextStyle, Parser},
        plugin::{ansi::Ansi, PluginWrapper},
        rendering::{
            cursor::LineCursor,
            line::{LineRenderState, StyledLineRenderer},
            line_iter::{
                test::{assert_line_elements, RenderElement},
                LineEndType,
            },
        },
        style::{HeightMode, TabSize, TextBoxStyleBuilder},
        utils::test::size_for,
        TextBox,
    };

    #[test]

    fn test_measure_line_cursor_back() {
        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Center)
            .build();

        let mut text = Parser::parse("123\x1b[2D");

        let mut plugin = PluginWrapper::new(Ansi::new());
        let lm = style.measure_line(
            &mut plugin,
            &character_style,
            &mut text,
            5 * FONT_6X9.character_size.width,
        );
        assert_eq!(lm.width, 3 * FONT_6X9.character_size.width);

        // Now a case where the string itself without rewind is wider than the line and the
        // continuation after rewind extends the line.
        let mut text = Parser::parse("123\x1b[2D456");

        let mut plugin = PluginWrapper::new(Ansi::new());
        let lm = style.measure_line(
            &mut plugin,
            &character_style,
            &mut text,
            5 * FONT_6X9.character_size.width,
        );
        assert_eq!(lm.width, 4 * FONT_6X9.character_size.width);
    }

    #[test]
    fn colors() {
        let mut parser = Parser::parse("Lorem \x1b[92mIpsum");
        let mw = PluginWrapper::new(Ansi::<Rgb888>::new());

        assert_line_elements(
            &mut parser,
            100,
            &[
                RenderElement::string("Lorem", 30),
                RenderElement::Space(1, true),
                RenderElement::ChangeTextStyle(ChangeTextStyle::TextColor(Some(Rgb888::new(
                    22, 198, 12,
                )))),
                RenderElement::string("Ipsum", 30),
            ],
            &mw,
        );
    }

    #[test]
    fn ansi_code_does_not_break_word() {
        let mut parser = Parser::parse("Lorem foo\x1b[92mbarum");
        let mw = PluginWrapper::new(Ansi::<Rgb888>::new());

        assert_line_elements(
            &mut parser,
            8,
            &[
                RenderElement::string("Lorem", 30),
                RenderElement::Space(1, false),
            ],
            &mw,
        );

        assert_line_elements(
            &mut parser,
            8,
            &[
                RenderElement::string("foo", 18),
                RenderElement::ChangeTextStyle(ChangeTextStyle::TextColor(Some(Rgb888::new(
                    22, 198, 12,
                )))),
                RenderElement::string("barum", 30),
            ],
            &mw,
        );
    }

    #[test]
    fn ansi_cursor_backwards() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let parser = Parser::parse("foo\x1b[2Dsample");

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new().build();

        let cursor = LineCursor::new(
            size_for(&FONT_6X9, 7, 1).width,
            TabSize::Spaces(4).into_pixels(&character_style),
        );

        let plugin = PluginWrapper::new(Ansi::new());
        let state = LineRenderState {
            parser,
            character_style,
            style: &style,
            end_type: LineEndType::EndOfText,
            plugin: &plugin,
        };
        StyledLineRenderer::new(cursor, state)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "..........................................",
            "...#...........................##.........",
            "..#.#...........................#.........",
            "..#.....###...###.##.#...###....#.....##..",
            ".###...##....#..#.#.#.#..#..#...#....#.##.",
            "..#......##..#..#.#.#.#..#..#...#....##...",
            "..#....###....###.#...#..###...###....###.",
            ".........................#................",
            ".........................#................",
        ]);
    }

    #[test]
    fn ansi_style_measure() {
        let text = "Some \x1b[4mstylish\x1b[24m multiline text that expands the widget vertically";

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Middle)
            .height_mode(HeightMode::FitToText)
            .build();

        let tb = TextBox::with_textbox_style(
            text,
            Rectangle::new(Point::zero(), Size::new(150, 240)),
            character_style,
            style,
        )
        .add_plugin(Ansi::new());

        assert_eq!(3 * 9, tb.bounds.size.height);
    }

    #[test]
    fn no_panic_when_word_is_broken() {
        let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let bounding_box = Rectangle::new(Point::zero(), Size::new(50, 20));

        TextBox::new("\x1b[4munderlined", bounding_box, character_style)
            .add_plugin(Ansi::new())
            .fit_height();
    }

    #[test]
    fn broken_underlned_token() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let bounding_box = Rectangle::new(Point::zero(), Size::new(50, 20));

        TextBox::new("\x1b[4munderlined", bounding_box, character_style)
            .add_plugin(Ansi::new())
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "                                                ",
            "                #              ##     #         ",
            "                #               #               ",
            "#   # # ##   ## #  ###  # ##    #    ##   # ##  ",
            "#   # ##  # #  ## #   # ##  #   #     #   ##  # ",
            "#   # #   # #   # ##### #       #     #   #   # ",
            "#  ## #   # #  ## #     #       #     #   #   # ",
            " ## # #   #  ## #  ###  #      ###   ###  #   # ",
            "                                                ",
            "################################################",
            "                                                ",
            "          #                                     ",
            "          #                                     ",
            " ###   ## #                                     ",
            "#   # #  ##                                     ",
            "##### #   #                                     ",
            "#     #  ##                                     ",
            " ###   ## #                                     ",
            "                                                ",
            "############                                    ",
        ]);
    }
}
