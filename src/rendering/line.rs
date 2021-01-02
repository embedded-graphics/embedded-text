//! Line rendering.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        character::CharacterIterator,
        cursor::Cursor,
        line_iter::{LineElementIterator, RenderElement},
        modified_whitespace::ModifiedEmptySpaceIterator,
        space_config::*,
    },
    style::{color::Rgb, height_mode::HeightMode, TextBoxStyle},
};
use core::ops::Range;
use embedded_graphics::{fonts::MonoFont, pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_core::primitives::{rectangle, Rectangle};

#[cfg(feature = "ansi")]
use crate::rendering::ansi::Sgr;

/// Internal state used to render a line.
#[derive(Debug)]
enum State<C, F>
where
    C: PixelColor,
    F: MonoFont,
{
    /// Fetch next render element.
    FetchNext,

    /// Render a character at the given position.
    Char(Point, CharacterIterator<F>),

    /// Render a block of whitespace at the given position.
    Space(rectangle::Points, C),

    /// Render a block of whitespace at the given position with underlined or strikethrough effect.
    ModifiedSpace(Point, ModifiedEmptySpaceIterator<F>),
}

/// Pixel iterator to render a single line of styled text.
#[derive(Debug)]
pub struct StyledLinePixelIterator<'a, 'b, C, F, SP, A, V, H>
where
    C: PixelColor,
    F: MonoFont,
{
    state: State<C, F>,
    style: &'b mut TextBoxStyle<C, F, A, V, H>,
    display_range: Range<i32>,
    inner: LineElementIterator<'a, 'b, F, SP, A>,
}

impl<'a, 'b, C, F, SP, A, V, H> StyledLinePixelIterator<'a, 'b, C, F, SP, A, V, H>
where
    C: PixelColor + From<Rgb>,
    F: MonoFont,
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    /// Creates a new pixel iterator to draw the given character.
    #[inline]
    #[must_use]
    pub fn new(
        parser: &'b mut Parser<'a>,
        cursor: &'b mut Cursor<F>,
        config: SP,
        style: &'b mut TextBoxStyle<C, F, A, V, H>,
        carried_token: &'b mut Option<Token<'a>>,
    ) -> Self {
        let tab_size = style.tab_size;
        Self {
            state: State::FetchNext,
            style,
            display_range: H::calculate_displayed_row_range(&cursor),
            inner: LineElementIterator::new(parser, cursor, config, carried_token, tab_size),
        }
    }

    fn is_anything_displayed(&self) -> bool {
        self.display_range.start < self.display_range.end
    }
}

impl<C, F, SP, A, V, H> Iterator for StyledLinePixelIterator<'_, '_, C, F, SP, A, V, H>
where
    C: PixelColor + From<Rgb>,
    F: MonoFont,
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                // No token being processed, get next one
                State::FetchNext => {
                    // HACK: avoid drawing the underline outside of the text box
                    let underlined = if self.style.underlined {
                        self.inner.cursor.position.y + self.display_range.end - 1
                            < self.inner.cursor.bottom_right().y
                    } else {
                        false
                    };

                    match self.inner.next() {
                        Some(RenderElement::PrintedCharacter(c)) => {
                            if self.is_anything_displayed() {
                                self.state = State::Char(
                                    self.inner.pos,
                                    CharacterIterator::new(
                                        c,
                                        self.display_range.clone(),
                                        underlined,
                                        self.style.strikethrough,
                                    ),
                                );
                            }
                        }

                        Some(RenderElement::Space(space_width, _)) => {
                            if self.is_anything_displayed() {
                                let row_range = self.display_range.clone();
                                self.state = if underlined || self.style.strikethrough {
                                    State::ModifiedSpace(
                                        self.inner.pos,
                                        ModifiedEmptySpaceIterator::new(
                                            space_width,
                                            row_range,
                                            underlined,
                                            self.style.strikethrough,
                                        ),
                                    )
                                } else if let Some(color) = self.style.text_style.background_color {
                                    let start = row_range.start;
                                    let rows = row_range.count() as u32;
                                    State::Space(
                                        Rectangle::new(
                                            self.inner.pos + Point::new(0, start),
                                            Size::new(space_width, rows),
                                        )
                                        .points(),
                                        color,
                                    )
                                } else {
                                    State::FetchNext
                                };
                            }
                        }

                        #[cfg(feature = "ansi")]
                        Some(RenderElement::Sgr(sgr)) => match sgr {
                            Sgr::Reset => {
                                self.style.text_style.text_color = None;
                                self.style.text_style.background_color = None;
                                self.style.underlined = false;
                                self.style.strikethrough = false;
                            }
                            Sgr::ChangeTextColor(color) => {
                                self.style.text_style.text_color = Some(color.into());
                            }
                            Sgr::DefaultTextColor => {
                                self.style.text_style.text_color = None;
                            }
                            Sgr::ChangeBackgroundColor(color) => {
                                self.style.text_style.background_color = Some(color.into());
                            }
                            Sgr::DefaultBackgroundColor => {
                                self.style.text_style.background_color = None;
                            }
                            Sgr::Underline => {
                                self.style.underlined = true;
                            }
                            Sgr::UnderlineOff => {
                                self.style.underlined = false;
                            }
                            Sgr::CrossedOut => {
                                self.style.strikethrough = true;
                            }
                            Sgr::NotCrossedOut => {
                                self.style.strikethrough = false;
                            }
                        },

                        None => break None,
                    };
                }

                State::Char(ref pos, ref mut iter) => {
                    if let Some(Pixel(position, color)) = iter.next() {
                        let color = match color {
                            BinaryColor::Off => self.style.text_style.background_color,
                            BinaryColor::On => self.style.text_style.text_color,
                        };
                        if let Some(color) = color {
                            break Some(Pixel(position + *pos, color));
                        }
                    } else {
                        self.state = State::FetchNext;
                    }
                }

                State::Space(ref mut iter, color) => {
                    if let Some(position) = iter.next() {
                        break Some(Pixel(position, color));
                    }

                    self.state = State::FetchNext;
                }

                State::ModifiedSpace(ref pos, ref mut iter) => {
                    if let Some(Pixel(position, color)) = iter.next() {
                        let color = match color {
                            BinaryColor::Off => self.style.text_style.background_color,
                            BinaryColor::On => self.style.text_style.text_color,
                        };
                        if let Some(color) = color {
                            break Some(Pixel(position + *pos, color));
                        }
                    } else {
                        self.state = State::FetchNext;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        alignment::{HorizontalTextAlignment, VerticalTextAlignment},
        parser::{Parser, Token},
        rendering::{
            cursor::Cursor,
            line::{StyledLinePixelIterator, UniformSpaceConfig},
        },
        style::{color::Rgb, height_mode::HeightMode, TextBoxStyle, TextBoxStyleBuilder},
    };
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    };
    use embedded_graphics_core::primitives::Rectangle;

    fn test_rendered_text<'a, C, F, A, V, H>(
        text: &'a str,
        bounds: Rectangle,
        mut style: TextBoxStyle<C, F, A, V, H>,
        pattern: &[&str],
    ) where
        C: PixelColor + From<Rgb> + embedded_graphics::mock_display::ColorMapping,
        F: MonoFont,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        let config = UniformSpaceConfig::new(F::CHARACTER_SIZE.width + F::CHARACTER_SPACING);
        let mut parser = Parser::parse(text);
        let mut cursor = Cursor::new(bounds, style.line_spacing);
        let mut carried = None;

        let iter = StyledLinePixelIterator::new(
            &mut parser,
            &mut cursor,
            config,
            &mut style,
            &mut carried,
        );
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        display.assert_pattern(pattern);
    }

    #[test]
    fn simple_render() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        test_rendered_text(
            " Some sample text",
            Rectangle::new(Point::zero(), Size::new(6 * 7, 8)),
            style,
            &[
                ".......###....................",
                "......#...#...................",
                "......#......###..##.#...###..",
                ".......###..#...#.#.#.#.#...#.",
                "..........#.#...#.#...#.#####.",
                "......#...#.#...#.#...#.#.....",
                ".......###...###..#...#..###..",
                "..............................",
            ],
        );
    }

    #[test]
    fn render_before_area() {
        let config = UniformSpaceConfig::new(Font6x8::CHARACTER_SIZE.width);
        let mut parser = Parser::parse(" Some sample text");
        let mut style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut cursor = Cursor::new(
            Rectangle::new(Point::new(0, 8), Size::new(6 * 7, 16)),
            style.line_spacing,
        );
        cursor.position.y -= 8;

        let mut carried = None;
        let mut iter = StyledLinePixelIterator::new(
            &mut parser,
            &mut cursor,
            config,
            &mut style,
            &mut carried,
        );

        assert!(
            iter.next().is_none(),
            "Drawing is not allowed outside the bounding area"
        );

        // even though nothing was drawn, the text should be consumed
        assert_eq!(Some(Token::Break(None)), carried);
    }

    #[test]
    fn simple_render_nbsp() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        test_rendered_text(
            "Some\u{A0}sample text",
            Rectangle::new(Point::zero(), Size::new(6 * 7, 8)),
            style,
            &[
                ".###......................................",
                "#...#.....................................",
                "#......###..##.#...###.........####..###..",
                ".###..#...#.#.#.#.#...#.......#.........#.",
                "....#.#...#.#...#.#####........###...####.",
                "#...#.#...#.#...#.#...............#.#...#.",
                ".###...###..#...#..###........####...####.",
                "..........................................",
            ],
        );
    }

    #[test]
    fn simple_render_first_word_not_wrapped() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        test_rendered_text(
            "Some sample text",
            Rectangle::new(Point::zero(), Size::new(6 * 2, 8)),
            style,
            &[
                ".###........",
                "#...#.......",
                "#......###..",
                ".###..#...#.",
                "....#.#...#.",
                "#...#.#...#.",
                ".###...###..",
                "............",
            ],
        );
    }

    #[test]
    fn newline_stops_render() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        test_rendered_text(
            "Some \nsample text",
            Rectangle::new(Point::zero(), Size::new(6 * 7, 8)),
            style,
            &[
                ".###..........................",
                "#...#.........................",
                "#......###..##.#...###........",
                ".###..#...#.#.#.#.#...#.......",
                "....#.#...#.#...#.#####.......",
                "#...#.#...#.#...#.#...........",
                ".###...###..#...#..###........",
                "..............................",
            ],
        );
    }

    #[test]
    fn underline_just_inside_of_textbox() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .underlined(true)
            .build();

        test_rendered_text(
            "s",
            Rectangle::new(Point::zero(), Size::new(6, 9)),
            style,
            &[
                "......             ",
                "......             ",
                ".####.             ",
                "#.....             ",
                ".###..             ",
                "....#.             ",
                "####..             ",
                "......             ",
                "######             ",
            ],
        );
    }

    #[test]
    fn underline_outside_of_textbox() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .underlined(true)
            .build();

        test_rendered_text(
            "s",
            Rectangle::new(Point::zero(), Size::new(6, 8)),
            style,
            &[
                "......             ",
                "......             ",
                ".####.             ",
                "#.....             ",
                ".###..             ",
                "....#.             ",
                "####..             ",
                "......             ",
            ],
        );
    }
}

#[cfg(all(test, feature = "ansi"))]
mod ansi_parser_tests {
    use crate::{
        parser::Parser,
        rendering::{
            cursor::Cursor,
            line::{StyledLinePixelIterator, UniformSpaceConfig},
        },
        style::TextBoxStyleBuilder,
    };
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    };
    use embedded_graphics_core::primitives::Rectangle;

    #[test]
    fn ansi_cursor_backwards() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);
        let config = UniformSpaceConfig::new(Font6x8::CHARACTER_SIZE.width);

        let mut parser = Parser::parse("foo\x1b[2Dsample");
        let mut style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();
        let mut cursor = Cursor::new(Rectangle::new(Point::zero(), Size::new(6 * 7, 8)), 0);
        let mut carried = None;
        let iter = StyledLinePixelIterator::new(
            &mut parser,
            &mut cursor,
            config,
            &mut style,
            &mut carried,
        );

        iter.draw(&mut display).unwrap();

        display.assert_pattern(&[
            "..##...........................##.........",
            ".#..#...........................#.........",
            ".#.....####..###..##.#..####....#....###..",
            "###...#.........#.#.#.#.#...#...#...#...#.",
            ".#.....###...####.#...#.#...#...#...#####.",
            ".#........#.#...#.#...#.####....#...#.....",
            ".#....####...####.#...#.#......###...###..",
            "........................#.................",
        ]);
    }
}
