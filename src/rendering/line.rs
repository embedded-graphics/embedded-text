//! Line rendering.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        ansi::Sgr,
        character::CharacterIterator,
        cursor::Cursor,
        line_iter::{LineElementIterator, RenderElement},
        modified_whitespace::ModifiedEmptySpaceIterator,
        space_config::*,
        whitespace::EmptySpaceIterator,
    },
    style::{color::Rgb, height_mode::HeightMode, TextBoxStyle},
};
use core::ops::Range;
use embedded_graphics::prelude::*;

/// Internal state used to render a line.
#[derive(Debug)]
enum State<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Fetch next render element.
    FetchNext,

    /// Render a character.
    Char(CharacterIterator<C, F>),

    /// Render a block of whitespace.
    Space(EmptySpaceIterator<C, F>),

    /// Render a block of whitespace with underlined or strikethrough effect.
    ModifiedSpace(ModifiedEmptySpaceIterator<C, F>),
}

/// Pixel iterator to render a single line of styled text.
#[derive(Debug)]
pub struct StyledLinePixelIterator<'a, C, F, SP, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    SP: SpaceConfig<Font = F>,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    state: State<C, F>,
    pub(crate) style: TextBoxStyle<C, F, A, V, H>,
    display_range: Range<i32>,
    inner: LineElementIterator<'a, F, SP, A>,
}

impl<'a, C, F, SP, A, V, H> StyledLinePixelIterator<'a, C, F, SP, A, V, H>
where
    C: PixelColor + From<Rgb>,
    F: Font + Copy,
    SP: SpaceConfig<Font = F>,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    /// Creates a new pixel iterator to draw the given character.
    #[inline]
    #[must_use]
    pub fn new(
        parser: Parser<'a>,
        cursor: Cursor<F>,
        config: SP,
        style: TextBoxStyle<C, F, A, V, H>,
        carried_token: Option<Token<'a>>,
    ) -> Self {
        Self {
            state: State::FetchNext,
            style,
            display_range: H::calculate_displayed_row_range(&cursor),
            inner: LineElementIterator::new(parser, cursor, config, carried_token, style.tab_size),
        }
    }

    /// When finished, this method returns the last partially processed [`Token`], or
    /// `None` if everything was rendered.
    ///
    /// [`Token`]: ../../parser/enum.Token.html
    #[must_use]
    #[inline]
    pub fn remaining_token(&self) -> Option<Token<'a>> {
        self.inner.remaining_token()
    }

    /// When finished, this method returns the text parser object.
    #[must_use]
    #[inline]
    pub fn parser(&self) -> Parser<'a> {
        self.inner.parser.clone()
    }

    /// When finished, this method returns the cursor object.
    #[must_use]
    #[inline]
    pub fn cursor(&self) -> Cursor<F> {
        self.inner.cursor
    }

    fn is_anything_displayed(&self) -> bool {
        self.display_range.start < self.display_range.end
    }
}

impl<C, F, SP, A, V, H> Iterator for StyledLinePixelIterator<'_, C, F, SP, A, V, H>
where
    C: PixelColor + From<Rgb>,
    F: Font + Copy,
    SP: SpaceConfig<Font = F>,
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
                        self.inner.cursor.position.y < self.inner.cursor.bounds.bottom_right.y
                    } else {
                        false
                    };

                    match self.inner.next() {
                        Some(RenderElement::PrintedCharacter(c)) => {
                            if self.is_anything_displayed() {
                                self.state = State::Char(CharacterIterator::new(
                                    c,
                                    self.inner.pos,
                                    self.style.text_style,
                                    self.display_range.clone(),
                                    underlined,
                                    self.style.strikethrough,
                                ));
                            } else {
                                self.state = State::FetchNext;
                            }
                        }

                        Some(RenderElement::Space(space_width, _)) => {
                            if self.is_anything_displayed() {
                                self.state = if self.style.underlined || self.style.strikethrough {
                                    State::ModifiedSpace(ModifiedEmptySpaceIterator::new(
                                        space_width,
                                        self.inner.pos,
                                        self.style.text_style,
                                        self.display_range.clone(),
                                        underlined,
                                        self.style.strikethrough,
                                    ))
                                } else {
                                    State::Space(EmptySpaceIterator::new(
                                        space_width,
                                        self.inner.pos,
                                        self.style.text_style,
                                        self.display_range.clone(),
                                    ))
                                };
                            } else {
                                self.state = State::FetchNext;
                            }
                        }

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

                State::Char(ref mut iter) => {
                    if let pixel @ Some(_) = iter.next() {
                        break pixel;
                    }

                    self.state = State::FetchNext;
                }

                State::Space(ref mut iter) => {
                    if let pixel @ Some(_) = iter.next() {
                        break pixel;
                    }

                    self.state = State::FetchNext;
                }

                State::ModifiedSpace(ref mut iter) => {
                    if let pixel @ Some(_) = iter.next() {
                        break pixel;
                    }

                    self.state = State::FetchNext;
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
        primitives::Rectangle,
    };

    fn test_rendered_text<'a, C, F, A, V, H>(
        text: &'a str,
        bounds: Rectangle,
        style: TextBoxStyle<C, F, A, V, H>,
        pattern: &[&str],
    ) -> StyledLinePixelIterator<'a, C, F, UniformSpaceConfig<F>, A, V, H>
    where
        C: PixelColor + From<Rgb> + embedded_graphics::mock_display::ColorMapping<C>,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        let parser = Parser::parse(text);
        let config = UniformSpaceConfig::default();

        let cursor = Cursor::new(bounds, style.line_spacing);
        let mut iter = StyledLinePixelIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(display, MockDisplay::from_pattern(pattern));

        iter
    }

    #[test]
    fn simple_render() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut iter = test_rendered_text(
            " Some sample text",
            Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 8)),
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
        assert_eq!(Some(Token::Break(None)), iter.remaining_token());
        assert_eq!(Some(Token::Word("sample")), iter.inner.parser.next());
    }

    #[test]
    fn render_before_area() {
        let parser = Parser::parse(" Some sample text");
        let config = UniformSpaceConfig::default();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut cursor = Cursor::new(
            Rectangle::new(Point::new(0, 8), Point::new(6 * 7 - 1, 16)),
            style.line_spacing,
        );
        cursor.position.y -= 8;

        let mut iter = StyledLinePixelIterator::new(parser, cursor, config, style, None);

        assert!(
            iter.next().is_none(),
            "Drawing is not allowed outside the bounding area"
        );

        // even though nothing was drawn, the text should be consumed
        assert_eq!(Some(Token::Break(None)), iter.remaining_token());
    }

    #[test]
    fn simple_render_nbsp() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let iter = test_rendered_text(
            "Some\u{A0}sample text",
            Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 8)),
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
        assert_eq!(Some(Token::Word("mple")), iter.remaining_token());
    }

    #[test]
    fn simple_render_first_word_not_wrapped() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let iter = test_rendered_text(
            "Some sample text",
            Rectangle::new(Point::zero(), Point::new(6 * 2 - 1, 7)),
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
        assert_eq!(Some(Token::Word("me")), iter.remaining_token());
    }

    #[test]
    fn newline_stops_render() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        test_rendered_text(
            "Some \nsample text",
            Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 7)),
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
    fn ansi_cursor_backwards() {
        let parser = Parser::parse("foo\x1b[2Dsample");
        let config = UniformSpaceConfig::default();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 7)), 0);
        let mut iter = StyledLinePixelIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "..##...........................##.........",
                ".#..#...........................#.........",
                ".#.....####..###..##.#..####....#....###..",
                "###...#.........#.#.#.#.#...#...#...#...#.",
                ".#.....###...####.#...#.#...#...#...#####.",
                ".#........#.#...#.#...#.####....#...#.....",
                ".#....####...####.#...#.#......###...###..",
                "........................#.................",
            ])
        );
    }

    #[test]
    fn carried_over_spaces() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let parser = Parser::parse("Some  sample text");
        let config = UniformSpaceConfig::default();
        let bounds = Rectangle::new(Point::zero(), Point::new(6 * 5 - 1, 7));
        let cursor = Cursor::new(bounds, style.line_spacing);
        let mut iter = StyledLinePixelIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        // eat one space, so one is rendered at the end of line and nothing in the next
        assert_eq!(Some(Token::Break(None)), iter.remaining_token());

        let mut iter = StyledLinePixelIterator::new(
            iter.parser(),
            cursor,
            config,
            style,
            iter.remaining_token(),
        );
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                ".........................##...",
                "..........................#...",
                ".####..###..##.#..####....#...",
                "#.........#.#.#.#.#...#...#...",
                ".###...####.#...#.#...#...#...",
                "....#.#...#.#...#.####....#...",
                "####...####.#...#.#......###..",
                "..................#...........",
            ])
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
            Rectangle::new(Point::zero(), Point::new(6 - 1, 7)),
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
