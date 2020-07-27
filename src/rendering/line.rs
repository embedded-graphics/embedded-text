//! Line rendering
use crate::{
    parser::{Parser, Token},
    rendering::{character::StyledCharacterIterator, whitespace::EmptySpaceIterator},
    utils::font_ext::FontExt,
};
use core::str::Chars;
use embedded_graphics::{prelude::*, style::TextStyle};

/// Internal state used to render a line
#[derive(Debug)]
pub enum LineState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Fetch next token
    FetchNext,

    /// Decide what to do next
    ProcessToken(Token<'a>),

    /// Render a word
    Word(Chars<'a>, StyledCharacterIterator<C, F>),

    /// Render whitespace
    Whitespace(u32, EmptySpaceIterator<C, F>),

    /// Signal that the renderer has finished, store the token that was consumed but not rendered
    Done(Option<Token<'a>>),
}

/// Retrieves size of space characters
pub trait SpaceConfig: Copy {
    /// Look at the size of next n spaces, without advancing
    fn peek_next_width(&self, n: u32) -> u32;

    /// Get the width of the next space and advance
    fn next_space_width(&mut self) -> u32;
}

/// Contains the fixed width of a space character
#[derive(Copy, Clone, Debug)]
pub struct UniformSpaceConfig(pub u32);
impl SpaceConfig for UniformSpaceConfig {
    #[inline]
    fn peek_next_width(&self, n: u32) -> u32 {
        n * self.0
    }

    #[inline]
    fn next_space_width(&mut self) -> u32 {
        self.0
    }
}

/// Renderer configuration options
#[derive(Copy, Clone, Debug)]
pub struct LineConfiguration<SP: SpaceConfig> {
    /// Render spaces at the start of a line
    pub starting_spaces: bool,

    /// Render spaces at the end of a line
    pub ending_spaces: bool,

    /// Space configuration
    pub space_config: SP,
}

/// Pixel iterator to render a styled character
#[derive(Debug)]
pub struct StyledLineIterator<'a, C, F, SP: SpaceConfig>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// The text to draw.
    pub parser: Parser<'a>,
    current_token: LineState<'a, C, F>,
    config: LineConfiguration<SP>,
    style: TextStyle<C, F>,
    pos: Point,
    max_x: i32,
    first_word: bool,
}

impl<'a, C, F, SP> StyledLineIterator<'a, C, F, SP>
where
    C: PixelColor,
    F: Font + Copy,
    SP: SpaceConfig,
{
    /// Creates a new pixel iterator to draw the given character.
    #[inline]
    #[must_use]
    pub fn new(
        parser: Parser<'a>,
        pos: Point,
        width: u32,
        config: LineConfiguration<SP>,
        style: TextStyle<C, F>,
        carried_token: Option<Token<'a>>,
    ) -> Self {
        Self {
            parser,
            current_token: carried_token
                .map(LineState::ProcessToken)
                .unwrap_or(LineState::FetchNext),
            config,
            style,
            pos,
            max_x: pos.x + width as i32 - 1,
            first_word: true,
        }
    }

    /// When finished, this method returns the last partially processed token, or
    /// None if everything was rendered.
    #[must_use]
    #[inline]
    pub fn remaining_token(&self) -> Option<Token<'a>> {
        match self.current_token {
            LineState::Done(ref t) => t.clone(),
            _ => None,
        }
    }

    fn fits_in_line(&self, width: u32) -> bool {
        self.pos.x + width as i32 - 1 <= self.max_x
    }
}

impl<C, F, SP> Iterator for StyledLineIterator<'_, C, F, SP>
where
    C: PixelColor,
    F: Font + Copy,
    SP: SpaceConfig,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current_token {
                LineState::FetchNext => {
                    self.current_token = if let Some(token) = self.parser.next() {
                        LineState::ProcessToken(token)
                    } else {
                        // we're done
                        LineState::Done(None)
                    }
                }

                LineState::ProcessToken(ref token) => {
                    // No token being processed, get next one
                    match token.clone() {
                        Token::Whitespace(n) => {
                            let render_whitespace = if self.first_word {
                                self.config.starting_spaces
                            } else if self.config.ending_spaces {
                                true
                            } else if let Some(Token::Word(w)) = self.parser.peek() {
                                // Check if space + w fits in line, otherwise it's up to config
                                let space_width = self.config.space_config.peek_next_width(n);
                                let word_width = F::str_width(w);

                                self.fits_in_line(space_width + word_width)
                            } else {
                                false
                            };

                            if render_whitespace {
                                // take as many spaces as possible and save the rest in state

                                let mut space_width = 0;
                                let mut spaces = n;

                                while spaces > 0
                                    && self.fits_in_line(
                                        space_width + self.config.space_config.peek_next_width(1),
                                    )
                                {
                                    spaces -= 1;
                                    space_width += self.config.space_config.next_space_width();
                                }

                                self.current_token = if space_width > 0 {
                                    let pos = self.pos;
                                    self.pos.x += space_width as i32;
                                    LineState::Whitespace(
                                        spaces,
                                        EmptySpaceIterator::new(space_width, pos, self.style),
                                    )
                                } else if spaces > 1 {
                                    // there are spaces to render but none fit the line
                                    // eat one as a newline and stop
                                    LineState::Done(Some(Token::Whitespace(
                                        spaces.saturating_sub(1),
                                    )))
                                } else {
                                    LineState::Done(None)
                                }
                            } else {
                                // nothing, process next token
                                self.current_token = LineState::FetchNext;
                            }
                        }

                        Token::Word(w) => {
                            if self.first_word {
                                self.first_word = false;
                            } else if !self.fits_in_line(F::str_width(w)) {
                                self.current_token = LineState::Done(Some(Token::Word(w)));
                                break None;
                            }

                            // - always draw first word, Word state should handle wrapping
                            let mut chars = w.chars();

                            // unwrap is safe here, parser doesn't emit empty words
                            let c = chars.next().unwrap();

                            let pos = self.pos;
                            self.pos.x += F::total_char_width(c) as i32;

                            self.current_token = LineState::Word(
                                chars,
                                StyledCharacterIterator::new(c, pos, self.style),
                            );
                        }

                        Token::NewLine => {
                            // we're done
                            self.current_token = LineState::Done(None);
                        }
                    }
                }

                LineState::Whitespace(ref n, ref mut iter) => {
                    if let pixel @ Some(_) = iter.next() {
                        break pixel;
                    }

                    self.current_token = if *n == 0 {
                        LineState::FetchNext
                    } else {
                        // n > 0 only if not every space was rendered
                        LineState::Done(Some(Token::Whitespace(*n)))
                    }
                }

                LineState::Word(ref chars, ref mut iter) => {
                    if let pixel @ Some(_) = iter.next() {
                        break pixel;
                    }

                    let mut lookahead = chars.clone();
                    self.current_token = if let Some(c) = lookahead.next() {
                        // character done, move to the next one
                        let char_width = F::total_char_width(c);

                        if !self.fits_in_line(char_width) {
                            // word wrapping, this line is done
                            LineState::Done(Some(Token::Word(chars.as_str())))
                        } else {
                            let pos = self.pos;
                            self.pos.x += char_width as i32;
                            LineState::Word(
                                lookahead,
                                StyledCharacterIterator::new(c, pos, self.style),
                            )
                        }
                    } else {
                        // process token
                        LineState::FetchNext
                    }
                }

                LineState::Done(_) => {
                    break None;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::parser::{Parser, Token};
    use crate::rendering::line::{LineConfiguration, StyledLineIterator, UniformSpaceConfig};
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        style::TextStyleBuilder,
    };

    #[test]
    fn simple_render() {
        let parser = Parser::parse(" Some sample text");
        let config = LineConfiguration {
            starting_spaces: true,
            ending_spaces: true,
            space_config: UniformSpaceConfig(6),
        };
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut iter = StyledLineIterator::new(parser, Point::zero(), 6 * 7, config, style, None);
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
    fn simple_render_first_word_not_wrapped() {
        let parser = Parser::parse(" Some sample text");
        let config = LineConfiguration {
            starting_spaces: true,
            ending_spaces: true,
            space_config: UniformSpaceConfig(6),
        };
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut iter = StyledLineIterator::new(parser, Point::zero(), 6 * 3, config, style, None);
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
        let config = LineConfiguration {
            starting_spaces: true,
            ending_spaces: true,
            space_config: UniformSpaceConfig(6),
        };
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut iter = StyledLineIterator::new(parser, Point::zero(), 6 * 7, config, style, None);
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
        let config = LineConfiguration {
            starting_spaces: false,
            ending_spaces: true,
            space_config: UniformSpaceConfig(6),
        };
        let style = TextStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut iter = StyledLineIterator::new(parser, Point::zero(), 6 * 3, config, style, None);
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
        let config = LineConfiguration {
            starting_spaces: true,
            ending_spaces: false,
            space_config: UniformSpaceConfig(6),
        };

        let mut iter = StyledLineIterator::new(parser, Point::zero(), 6 * 7, config, style, None);
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
        let config = LineConfiguration {
            starting_spaces: true,
            ending_spaces: true,
            space_config: UniformSpaceConfig(6),
        };

        let mut iter = StyledLineIterator::new(parser, Point::zero(), 6 * 7, config, style, None);
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
        let config = LineConfiguration {
            starting_spaces: true,
            ending_spaces: true,
            space_config: UniformSpaceConfig(6),
        };

        let mut iter = StyledLineIterator::new(parser, Point::zero(), 6 * 5, config, style, None);
        let mut display = MockDisplay::new();

        iter.draw(&mut display).unwrap();

        assert_eq!(Some(Token::Whitespace(1)), iter.remaining_token());

        let mut iter = StyledLineIterator::new(
            iter.parser.clone(),
            Point::zero(),
            6 * 5,
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
