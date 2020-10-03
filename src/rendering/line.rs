//! Line rendering.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        character::StyledCharacterIterator,
        cursor::Cursor,
        line_iter::{LineElementIterator, RenderElement},
        space_config::*,
        whitespace::EmptySpaceIterator,
    },
    style::{height_mode::HeightMode, TextBoxStyle},
};
use core::ops::Range;
use embedded_graphics::{prelude::*, style::TextStyle};

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
    Char(StyledCharacterIterator<C, F>),

    /// Render a block of whitespace.
    Space(EmptySpaceIterator<C, F>),
}

/// Pixel iterator to render a single line of styled text.
#[derive(Debug)]
pub struct StyledLinePixelIterator<'a, C, F, SP, A>
where
    C: PixelColor,
    F: Font + Copy,
    SP: SpaceConfig<Font = F>,
    A: HorizontalTextAlignment,
{
    state: State<C, F>,
    style: TextStyle<C, F>,
    display_range: Range<i32>,
    inner: LineElementIterator<'a, F, SP, A>,
}

impl<'a, C, F, SP, A> StyledLinePixelIterator<'a, C, F, SP, A>
where
    C: PixelColor,
    F: Font + Copy,
    SP: SpaceConfig<Font = F>,
    A: HorizontalTextAlignment,
{
    /// Creates a new pixel iterator to draw the given character.
    #[inline]
    #[must_use]
    pub fn new<V, H>(
        parser: Parser<'a>,
        cursor: Cursor<F>,
        config: SP,
        style: TextBoxStyle<C, F, A, V, H>,
        carried_token: Option<Token<'a>>,
    ) -> Self
    where
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        Self {
            state: State::FetchNext,
            style: style.text_style,
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

impl<C, F, SP, A> Iterator for StyledLinePixelIterator<'_, C, F, SP, A>
where
    C: PixelColor,
    F: Font + Copy,
    SP: SpaceConfig<Font = F>,
    A: HorizontalTextAlignment,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                // No token being processed, get next one
                State::FetchNext => {
                    match self.inner.next() {
                        Some(RenderElement::PrintedCharacter(c)) => {
                            if self.is_anything_displayed() {
                                self.state = State::Char(StyledCharacterIterator::new(
                                    c,
                                    self.inner.pos,
                                    self.style,
                                    self.display_range.clone(),
                                ));
                            } else {
                                self.state = State::FetchNext;
                            }
                        }

                        Some(RenderElement::Space(space_width, _)) => {
                            if self.is_anything_displayed() {
                                self.state = State::Space(EmptySpaceIterator::new(
                                    space_width,
                                    self.inner.pos,
                                    self.style,
                                ));
                            } else {
                                self.state = State::FetchNext;
                            }
                        }

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
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        parser::{Parser, Token},
        rendering::{
            cursor::Cursor,
            line::{StyledLinePixelIterator, UniformSpaceConfig},
        },
        style::TextBoxStyleBuilder,
    };
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        primitives::Rectangle,
    };

    #[test]
    fn simple_render() {
        let parser = Parser::parse(" Some sample text");
        let config = UniformSpaceConfig::default();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 8)), 0);
        let mut iter = StyledLinePixelIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                ".......###..........................",
                "......#...#.........................",
                "......#......###..##.#...###........",
                ".......###..#...#.#.#.#.#...#.......",
                "..........#.#...#.#...#.#####.......",
                "......#...#.#...#.#...#.#...........",
                ".......###...###..#...#..###........",
                "....................................",
            ])
        );
        assert_eq!(Some(Token::Word("sample")), iter.remaining_token());
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
            0,
        );
        cursor.position.y -= 8;

        let mut iter = StyledLinePixelIterator::new(parser, cursor, config, style, None);

        assert!(
            iter.next().is_none(),
            "Drawing is not allowed outside the bounding area"
        );

        // even though nothing was drawn, the text should be consumed
        assert_eq!(Some(Token::Word("sample")), iter.remaining_token());
    }

    #[test]
    fn simple_render_nbsp() {
        let parser = Parser::parse("Some\u{A0}sample text");
        let config = UniformSpaceConfig::default();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 8)), 0);
        let mut iter = StyledLinePixelIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                ".###......................................",
                "#...#.....................................",
                "#......###..##.#...###.........####..###..",
                ".###..#...#.#.#.#.#...#.......#.........#.",
                "....#.#...#.#...#.#####........###...####.",
                "#...#.#...#.#...#.#...............#.#...#.",
                ".###...###..#...#..###........####...####.",
                "..........................................",
            ])
        );
        assert_eq!(Some(Token::Word("mple")), iter.remaining_token());
    }

    #[test]
    fn simple_render_first_word_not_wrapped() {
        let parser = Parser::parse(" Some sample text");
        let config = UniformSpaceConfig::default();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 3 - 1, 7)), 0);
        let mut iter = StyledLinePixelIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                ".......###........",
                "......#...#.......",
                "......#......###..",
                ".......###..#...#.",
                "..........#.#...#.",
                "......#...#.#...#.",
                ".......###...###..",
                "..................",
            ])
        );
        assert_eq!(Some(Token::Word("me")), iter.remaining_token());
    }

    #[test]
    fn newline_stops_render() {
        let parser = Parser::parse("Some \nsample text");
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
                ".###..........................",
                "#...#.........................",
                "#......###..##.#...###........",
                ".###..#...#.#.#.#.#...#.......",
                "....#.#...#.#...#.#####.......",
                "#...#.#...#.#...#.#...........",
                ".###...###..#...#..###........",
                "..............................",
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

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 5 - 1, 7)), 0);
        let mut iter = StyledLinePixelIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(Some(Token::Whitespace(1)), iter.remaining_token());

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
                "..............................",
                "..............................",
                ".......####..###..##.#..####..",
                "......#.........#.#.#.#.#...#.",
                ".......###...####.#...#.#...#.",
                "..........#.#...#.#...#.####..",
                "......####...####.#...#.#.....",
                "........................#.....",
            ])
        );
    }
}
