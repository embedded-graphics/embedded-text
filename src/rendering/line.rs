//! Line rendering
use crate::{
    parser::{Parser, Token},
    rendering::{character::StyledCharacterIterator, whitespace::EmptySpaceIterator},
    utils::font_ext::FontExt,
};
use core::str::Chars;
use embedded_graphics::{prelude::*, style::TextStyle};

#[derive(Debug)]
pub enum LineState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    ProcessToken(Token<'a>),
    Word(Chars<'a>, StyledCharacterIterator<C, F>),
    Whitespace(u32, EmptySpaceIterator<C, F>),
    Done(Option<Token<'a>>),
}

pub trait SpaceConfig: Copy {
    fn peek_next_width(&self, n: u32) -> u32;
    fn next_space_width(&mut self) -> u32;
}

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

#[derive(Copy, Clone, Debug)]
pub struct LineConfiguration<SP: SpaceConfig> {
    pub starting_spaces: bool,
    pub ending_spaces: bool,
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
    current_token: Option<LineState<'a, C, F>>,
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
            current_token: carried_token.map(LineState::ProcessToken),
            config,
            style,
            pos,
            max_x: pos.x + width as i32 - 1,
            first_word: true,
        }
    }

    /// When finished, this method returns the last partially processed token, or
    /// None if everything was rendered.
    pub fn remaining_token(&self) -> Option<Token<'a>> {
        match self.current_token {
            Some(LineState::Done(ref t)) => t.clone(),
            _ => None,
        }
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
                None => {
                    if let Some(token) = self.parser.next() {
                        self.current_token = Some(LineState::ProcessToken(token));
                    } else {
                        // we're done
                        self.current_token = Some(LineState::Done(None));
                        break None;
                    }
                }

                Some(LineState::ProcessToken(ref token)) => {
                    // No token being processed, get next one
                    match token.clone() {
                        Token::Whitespace(n) => {
                            let render_whitespace = if self.first_word {
                                self.config.starting_spaces
                            } else {
                                let lookahead = self.parser.peek();

                                if self.config.ending_spaces {
                                    true
                                } else if let Some(Token::Word(w)) = lookahead {
                                    // Check if space + w fits in line, otherwise it's up to config
                                    let space_width = self.config.space_config.peek_next_width(n);
                                    let word_width = F::str_width(w);

                                    let width = (space_width + word_width) as i32;

                                    self.pos.x + width <= self.max_x
                                } else {
                                    false
                                }
                            };

                            if render_whitespace {
                                // take as many spaces as possible and save the rest in state

                                let pos = self.pos;
                                let mut space_width = 0;

                                let mut spaces = n;
                                while spaces > 0
                                    && pos.x
                                        + (space_width
                                            + self.config.space_config.peek_next_width(0))
                                            as i32
                                        <= self.max_x
                                {
                                    spaces -= 1;
                                    space_width += self.config.space_config.next_space_width();
                                }

                                self.pos.x += space_width as i32;
                                self.current_token = Some(LineState::Whitespace(
                                    spaces,
                                    EmptySpaceIterator::new(space_width, pos, self.style),
                                ));
                            } else {
                                // nothing, process next token
                                self.current_token = None;
                            }
                        }

                        Token::Word(w) => {
                            if self.first_word {
                                self.first_word = false;
                            } else {
                                let word_width = F::str_width(w) as i32;
                                if self.pos.x + word_width > self.max_x {
                                    self.current_token =
                                        Some(LineState::Done(Some(Token::Word(w))));
                                    break None;
                                }
                            }

                            // - always draw first word, Word state should handle wrapping
                            let mut chars = w.chars();

                            // unwrap is safe here, parser doesn't emit empty words
                            let c = chars.next().unwrap();

                            let pos = self.pos;
                            self.pos.x += F::total_char_width(c) as i32;

                            self.current_token = Some(LineState::Word(
                                chars,
                                StyledCharacterIterator::new(c, pos, self.style),
                            ));
                        }

                        _ => {
                            // we're done
                            self.current_token = Some(LineState::Done(None));
                            break None;
                        }
                    }
                }

                Some(LineState::Whitespace(ref n, ref mut iter)) => {
                    if let pixel @ Some(_) = iter.next() {
                        break pixel;
                    }

                    if *n == 0 {
                        self.current_token = None;
                    } else {
                        // n > 0 only if not every space was rendered
                        self.current_token = Some(LineState::Done(Some(Token::Whitespace(*n))));
                        break None;
                    }
                }

                Some(LineState::Word(ref mut chars, ref mut iter)) => {
                    if let pixel @ Some(_) = iter.next() {
                        break pixel;
                    }

                    let mut lookahead = chars.clone();
                    if let Some(c) = lookahead.next() {
                        // character done, move to the next one
                        let pos = self.pos;
                        self.pos.x += F::total_char_width(c) as i32;

                        if self.pos.x > self.max_x + 1 {
                            // word wrapping, this line is done
                            self.current_token =
                                Some(LineState::Done(Some(Token::Word(chars.as_str()))));
                            break None;
                        }

                        self.current_token = Some(LineState::Word(
                            lookahead,
                            StyledCharacterIterator::new(c, pos, self.style),
                        ));
                    } else {
                        // process token
                        self.current_token = None;
                    }
                }

                Some(LineState::Done(_)) => {
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
