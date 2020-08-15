//! Line rendering.
use crate::{
    alignment::HorizontalTextAlignment,
    parser::{Parser, Token},
    rendering::{
        character::StyledCharacterIterator, cursor::Cursor, whitespace::EmptySpaceIterator,
    },
    utils::font_ext::FontExt,
};
use core::{marker::PhantomData, str::Chars};
use embedded_graphics::{prelude::*, style::TextStyle};

/// Internal state used to render a line.
#[derive(Debug)]
pub enum State<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Fetch next token.
    FetchNext,

    /// Decide what to do next.
    ProcessToken(Token<'a>),

    /// Render a character in a word.
    WordChar(Chars<'a>, StyledCharacterIterator<C, F>),

    /// Render a printed space in a word.
    WordSpace(Chars<'a>, EmptySpaceIterator<C, F>),

    /// Render whitespace.
    Whitespace(u32, EmptySpaceIterator<C, F>),

    /// Signal that the renderer has finished, store the token that was consumed but not rendered.
    Done(Option<Token<'a>>),
}

/// Retrieves size of space characters.
pub trait SpaceConfig: Copy {
    /// Look at the size of next n spaces, without advancing.
    fn peek_next_width(&self, n: u32) -> u32;

    /// Advance the internal state
    fn consume(&mut self, n: u32) -> u32;
}

/// Contains the fixed width of a space character.
#[derive(Copy, Clone, Debug)]
pub struct UniformSpaceConfig {
    /// Space width.
    pub space_width: u32,
}

impl SpaceConfig for UniformSpaceConfig {
    #[inline]
    fn peek_next_width(&self, n: u32) -> u32 {
        n * self.space_width
    }

    #[inline]
    fn consume(&mut self, n: u32) -> u32 {
        self.peek_next_width(n)
    }
}

/// Pixel iterator to render a single line of styled text.
#[derive(Debug)]
pub struct StyledLineIterator<'a, C, F, SP, A>
where
    C: PixelColor,
    F: Font + Copy,
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
{
    /// Position information.
    pub cursor: Cursor<F>,

    /// The text to draw.
    pub parser: Parser<'a>,

    current_token: State<'a, C, F>,
    config: SP,
    style: TextStyle<C, F>,
    first_word: bool,
    alignment: PhantomData<A>,
}

impl<'a, C, F, SP, A> StyledLineIterator<'a, C, F, SP, A>
where
    C: PixelColor,
    F: Font + Copy,
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
{
    /// Creates a new pixel iterator to draw the given character.
    #[inline]
    #[must_use]
    pub fn new(
        parser: Parser<'a>,
        cursor: Cursor<F>,
        config: SP,
        style: TextStyle<C, F>,
        carried_token: Option<Token<'a>>,
    ) -> Self {
        Self {
            parser,
            current_token: carried_token.map_or(State::FetchNext, State::ProcessToken),
            config,
            cursor,
            style,
            first_word: true,
            alignment: PhantomData,
        }
    }

    /// When finished, this method returns the last partially processed [`Token`], or
    /// `None` if everything was rendered.
    ///
    /// [`Token`]: ../../parser/enum.Token.html
    #[must_use]
    #[inline]
    pub fn remaining_token(&self) -> Option<Token<'a>> {
        match self.current_token {
            State::Done(ref t) => t.clone(),
            _ => None,
        }
    }

    fn fits_in_line(&self, width: u32) -> bool {
        self.cursor.fits_in_line(width)
    }

    fn try_draw_next_character(&mut self, word: &'a str) -> State<'a, C, F> {
        let mut lookahead = word.chars();
        let pos = self.cursor.position;
        lookahead.next().map_or(State::FetchNext, |c| {
            if c == '\u{A0}' {
                // nbsp
                let sp_width = self.config.peek_next_width(1);

                if self.cursor.advance(sp_width) {
                    self.config.consume(1); // we have peeked the value, consume it
                    return if self.cursor.in_display_area() {
                        State::WordSpace(
                            lookahead,
                            EmptySpaceIterator::new(sp_width, pos, self.style),
                        )
                    } else {
                        State::ProcessToken(Token::Word(lookahead.as_str()))
                    };
                }
            } else {
                // character done, move to the next one
                let char_width = F::total_char_width(c);

                if self.cursor.advance(char_width) {
                    return if self.cursor.in_display_area() {
                        State::WordChar(lookahead, StyledCharacterIterator::new(c, pos, self.style))
                    } else {
                        State::ProcessToken(Token::Word(lookahead.as_str()))
                    };
                }
            }

            // word wrapping, this line is done
            Self::finish(&mut self.cursor, Token::Word(word))
        })
    }

    fn finish_draw_whitespace(cursor: &mut Cursor<F>, carried: u32) -> State<'a, C, F> {
        if carried == 0 {
            State::FetchNext
        } else {
            // n > 0 only if not every space was rendered
            Self::finish(cursor, Token::Whitespace(carried))
        }
    }

    fn finish(cursor: &mut Cursor<F>, t: Token<'a>) -> State<'a, C, F> {
        let carried_token = match t {
            Token::NewLine => {
                cursor.new_line();
                cursor.carriage_return();
                None
            }

            Token::CarriageReturn => {
                cursor.carriage_return();
                None
            }

            c => {
                cursor.new_line();
                cursor.carriage_return();
                Some(c)
            }
        };

        State::Done(carried_token)
    }
}

impl<C, F, SP, A> Iterator for StyledLineIterator<'_, C, F, SP, A>
where
    C: PixelColor,
    F: Font + Copy,
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current_token {
                // No token being processed, get next one
                State::FetchNext => {
                    self.current_token = self.parser.next().map_or_else(
                        || Self::finish(&mut self.cursor, Token::NewLine),
                        State::ProcessToken,
                    );
                }

                State::ProcessToken(ref token) => {
                    let token = token.clone();
                    self.current_token = match token {
                        Token::Whitespace(n) => {
                            let mut would_wrap = false;
                            let render_whitespace = if self.first_word {
                                A::STARTING_SPACES
                            } else if A::ENDING_SPACES {
                                true
                            } else if let Some(Token::Word(w)) = self.parser.peek() {
                                // Check if space + w fits in line, otherwise it's up to config
                                let space_width = self.config.peek_next_width(n);
                                let word_width = F::str_width_nocr(w);

                                let fits = self.fits_in_line(space_width + word_width);

                                would_wrap = !fits;

                                fits
                            } else {
                                false
                            };

                            if render_whitespace {
                                // take as many spaces as possible and save the rest in state

                                // we could also binary search but I don't think it's worth it
                                let mut spaces_to_render = 0;
                                let available = self.cursor.space();
                                while spaces_to_render < n
                                    && self.config.peek_next_width(spaces_to_render + 1) < available
                                {
                                    spaces_to_render += 1;
                                }

                                if spaces_to_render > 0 {
                                    let pos = self.cursor.position;
                                    let space_width = self.config.consume(spaces_to_render);
                                    self.cursor.advance_unchecked(space_width);
                                    if self.cursor.in_display_area() {
                                        State::Whitespace(
                                            n - spaces_to_render,
                                            EmptySpaceIterator::new(space_width, pos, self.style),
                                        )
                                    } else {
                                        Self::finish_draw_whitespace(
                                            &mut self.cursor,
                                            n - spaces_to_render,
                                        )
                                    }
                                } else {
                                    // there are spaces to render but none fit the line
                                    // eat one as a newline and stop
                                    Self::finish(
                                        &mut self.cursor,
                                        if n > 1 {
                                            Token::Whitespace(n - 1)
                                        } else {
                                            Token::NewLine
                                        },
                                    )
                                }
                            } else if would_wrap {
                                Self::finish(&mut self.cursor, Token::NewLine)
                            } else {
                                // nothing, process next token
                                State::FetchNext
                            }
                        }

                        Token::Word(w) => {
                            if self.first_word {
                                self.first_word = false;
                            } else if !self.fits_in_line(F::str_width_nocr(w)) {
                                self.current_token = Self::finish(&mut self.cursor, token);
                                break None;
                            }

                            // FIXME: this isn't exactly optimal when outside of the display area
                            self.try_draw_next_character(w)
                        }

                        Token::NewLine | Token::CarriageReturn => {
                            // we're done
                            Self::finish(&mut self.cursor, token)
                        }
                    }
                }

                State::Whitespace(ref n, ref mut iter) => {
                    if let pixel @ Some(_) = iter.next() {
                        break pixel;
                    }

                    self.current_token = Self::finish_draw_whitespace(&mut self.cursor, *n);
                }

                State::WordChar(ref chars, ref mut iter) => {
                    if let pixel @ Some(_) = iter.next() {
                        break pixel;
                    }

                    let word = chars.as_str();
                    self.current_token = self.try_draw_next_character(word);
                }

                State::WordSpace(ref chars, ref mut iter) => {
                    if let pixel @ Some(_) = iter.next() {
                        break pixel;
                    }

                    let word = chars.as_str();
                    self.current_token = self.try_draw_next_character(word);
                }

                State::Done(_) => {
                    break None;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        alignment::HorizontalTextAlignment,
        parser::{Parser, Token},
        rendering::{
            cursor::Cursor,
            line::{StyledLineIterator, UniformSpaceConfig},
        },
    };
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        primitives::Rectangle, style::TextStyleBuilder,
    };

    #[derive(Copy, Clone)]
    pub struct AllSpaces;
    impl HorizontalTextAlignment for AllSpaces {
        const STARTING_SPACES: bool = true;
        const ENDING_SPACES: bool = true;
    }
    #[derive(Copy, Clone)]
    pub struct StartingSpaces;
    impl HorizontalTextAlignment for StartingSpaces {
        const STARTING_SPACES: bool = true;
        const ENDING_SPACES: bool = false;
    }
    #[derive(Copy, Clone)]
    pub struct EndingSpaces;
    impl HorizontalTextAlignment for EndingSpaces {
        const STARTING_SPACES: bool = false;
        const ENDING_SPACES: bool = true;
    }

    #[test]
    fn simple_render() {
        let parser = Parser::parse(" Some sample text");
        let config = UniformSpaceConfig { space_width: 6 };
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 8)));
        let mut iter: StyledLineIterator<_, _, _, AllSpaces> =
            StyledLineIterator::new(parser, cursor, config, style, None);
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
        let config = UniformSpaceConfig { space_width: 6 };
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut cursor = Cursor::new(Rectangle::new(Point::new(0, 8), Point::new(6 * 7 - 1, 16)));
        cursor.position.y -= 8;

        let mut iter: StyledLineIterator<_, _, _, AllSpaces> =
            StyledLineIterator::new(parser, cursor, config, style, None);

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
        let config = UniformSpaceConfig { space_width: 6 };
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 8)));
        let mut iter: StyledLineIterator<_, _, _, AllSpaces> =
            StyledLineIterator::new(parser, cursor, config, style, None);
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
        let config = UniformSpaceConfig { space_width: 6 };
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 3 - 1, 7)));
        let mut iter: StyledLineIterator<_, _, _, AllSpaces> =
            StyledLineIterator::new(parser, cursor, config, style, None);
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
        let config = UniformSpaceConfig { space_width: 6 };
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 7)));
        let mut iter: StyledLineIterator<_, _, _, AllSpaces> =
            StyledLineIterator::new(parser, cursor, config, style, None);
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
    fn first_spaces_not_rendered() {
        let parser = Parser::parse("  Some sample text");
        let config = UniformSpaceConfig { space_width: 6 };
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 3 - 1, 7)));
        let mut iter: StyledLineIterator<_, _, _, EndingSpaces> =
            StyledLineIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                ".###..............",
                "#...#.............",
                "#......###..##.#..",
                ".###..#...#.#.#.#.",
                "....#.#...#.#...#.",
                "#...#.#...#.#...#.",
                ".###...###..#...#.",
                "..................",
            ])
        );
    }

    #[test]
    fn last_spaces() {
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let parser = Parser::parse("Some  sample text");
        let config = UniformSpaceConfig { space_width: 6 };

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 7)));
        let mut iter: StyledLineIterator<_, _, _, StartingSpaces> =
            StyledLineIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                ".###....................",
                "#...#...................",
                "#......###..##.#...###..",
                ".###..#...#.#.#.#.#...#.",
                "....#.#...#.#...#.#####.",
                "#...#.#...#.#...#.#.....",
                ".###...###..#...#..###..",
                "........................",
            ])
        );

        let parser = Parser::parse("Some  sample text");
        let config = UniformSpaceConfig { space_width: 6 };

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 7 - 1, 7)));
        let mut iter: StyledLineIterator<_, _, _, AllSpaces> =
            StyledLineIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                ".###................................",
                "#...#...............................",
                "#......###..##.#...###..............",
                ".###..#...#.#.#.#.#...#.............",
                "....#.#...#.#...#.#####.............",
                "#...#.#...#.#...#.#.................",
                ".###...###..#...#..###..............",
                "....................................",
            ])
        );
    }

    #[test]
    fn carried_over_spaces() {
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let parser = Parser::parse("Some  sample text");
        let config = UniformSpaceConfig { space_width: 6 };

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 5 - 1, 7)));
        let mut iter: StyledLineIterator<_, _, _, AllSpaces> =
            StyledLineIterator::new(parser, cursor, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(Some(Token::Whitespace(1)), iter.remaining_token());

        let mut iter: StyledLineIterator<_, _, _, AllSpaces> = StyledLineIterator::new(
            iter.parser.clone(),
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
