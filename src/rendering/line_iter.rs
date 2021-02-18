//! Line iterator.
//!
//! Provide elements (spaces or characters) to render as long as they fit in the current line
use crate::{
    alignment::HorizontalTextAlignment,
    parser::{Parser, Token, SPEC_CHAR_NBSP},
    rendering::{cursor::LineCursor, space_config::SpaceConfig},
};
use core::marker::PhantomData;
use embedded_graphics::geometry::Point;

#[cfg(feature = "ansi")]
use super::ansi::{try_parse_sgr, Sgr};
#[cfg(feature = "ansi")]
use ansi_parser::AnsiSequence;
#[cfg(feature = "ansi")]
use as_slice::AsSlice;

/// Internal state used to render a line.
#[derive(Debug)]
enum State<'a> {
    /// Decide what to do next.
    ProcessToken(Token<'a>),

    FirstWord(&'a str),
    Word(&'a str),

    /// Signal that the renderer has finished.
    Done,
}

/// What to draw
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RenderElement<'a> {
    /// Render a whitespace block with the given width
    Space(u32),

    /// Render the given character
    PrintedCharacters(&'a str, u32),

    /// Move the cursor
    #[cfg(feature = "ansi")]
    MoveCursor(i32),

    /// A Select Graphic Rendition code
    #[cfg(feature = "ansi")]
    Sgr(Sgr),
}

/// Parser to break down a line into primitive elements used by measurement and rendering.
#[derive(Debug)]
pub struct LineElementParser<'a, 'b, M, SP, A> {
    /// Position information.
    pub cursor: LineCursor,

    /// The text to draw.
    pub parser: &'b mut Parser<'a>,

    pub(crate) pos: Point,
    current_token: State<'a>,
    config: SP,
    first_word: bool,
    alignment: PhantomData<A>,
    carried_token: &'b mut Option<Token<'a>>,
    measure: M,
}

impl<'a, 'b, M, SP, A> LineElementParser<'a, 'b, M, SP, A>
where
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
    M: Fn(&str) -> u32,
{
    /// Creates a new element parser.
    #[inline]
    #[must_use]
    pub fn new(
        parser: &'b mut Parser<'a>,
        cursor: LineCursor,
        config: SP,
        carried_token: &'b mut Option<Token<'a>>,
        measure: M,
    ) -> Self {
        let current_token = carried_token
            .take() // forget the old carried token
            .filter(|t| ![Token::NewLine, Token::CarriageReturn, Token::Break(None)].contains(t))
            .or_else(|| parser.next())
            .map_or(State::Done, State::ProcessToken);

        Self {
            parser,
            current_token,
            config,
            cursor,
            first_word: true,
            alignment: PhantomData,
            pos: Point::zero(),
            measure,
            carried_token,
        }
    }

    pub fn iter(&mut self) -> LineElementParserIterator<'a, 'b, '_, M, SP, A> {
        LineElementParserIterator { parser: self }
    }
}

pub struct LineElementParserIterator<'a, 'b, 'c, M, SP, A> {
    parser: &'c mut LineElementParser<'a, 'b, M, SP, A>,
}

impl<'a, M, SP, A> LineElementParserIterator<'a, '_, '_, M, SP, A>
where
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
    M: Fn(&str) -> u32,
{
    fn next_token(&mut self) {
        match self.parser.parser.next() {
            None => self.finish_end_of_string(),
            Some(t) => self.parser.current_token = State::ProcessToken(t),
        }
    }

    fn finish_end_of_string(&mut self) {
        self.parser.current_token = State::Done;
    }

    fn finish_wrapped(&mut self) {
        self.finish(Token::Break(None));
    }

    fn finish(&mut self, t: Token<'a>) {
        self.parser.carried_token.replace(t);
        self.parser.current_token = State::Done;
    }

    fn next_word_width(&mut self) -> Option<u32> {
        let mut width = None;
        let mut lookahead = self.parser.parser.clone();

        'lookahead: loop {
            match lookahead.next() {
                Some(Token::Word(w)) => {
                    let w = self.str_width(w);

                    width = width.map_or(Some(w), |acc| Some(acc + w));
                }

                Some(Token::Break(Some(c))) => {
                    let w = self.str_width(c);
                    width = width.map_or(Some(w), |acc| Some(acc + w));
                    break 'lookahead;
                }

                #[cfg(feature = "ansi")]
                Some(Token::EscapeSequence(_)) => {}

                _ => break 'lookahead,
            }
        }

        width
    }

    fn str_width(&self, s: &str) -> u32 {
        let measure = &self.parser.measure;
        measure(s)
    }

    fn count_widest_space_seq(&self, n: u32) -> u32 {
        // we could also binary search but I don't think it's worth it
        let mut spaces_to_render = 0;
        let available = self.parser.cursor.space();
        while spaces_to_render < n
            && self.parser.config.peek_next_width(spaces_to_render + 1) < available
        {
            spaces_to_render += 1;
        }

        spaces_to_render
    }

    fn move_cursor(&mut self, by: i32) -> Result<i32, i32> {
        self.parser.cursor.move_cursor(by as i32)
    }
}

impl<'a, M, SP, A> Iterator for LineElementParserIterator<'a, '_, '_, M, SP, A>
where
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
    M: Fn(&str) -> u32,
{
    type Item = RenderElement<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.parser.pos = self.parser.cursor.pos();
            match core::mem::replace(&mut self.parser.current_token, State::Done) {
                // No token being processed, get next one
                State::ProcessToken(ref token) => {
                    let token = token.clone();
                    match token {
                        Token::Whitespace(n) => {
                            // This mess decides if we want to render whitespace at all.
                            // The current horizontal alignment can ignore spaces at the beginning
                            // and end of a line.
                            let mut would_wrap = false;
                            let render_whitespace = if self.parser.first_word {
                                if A::STARTING_SPACES {
                                    self.parser.first_word = false;
                                }
                                A::STARTING_SPACES
                            } else if let Some(word_width) = self.next_word_width() {
                                // Check if space + w fits in line, otherwise it's up to config
                                let space_width = self.parser.config.peek_next_width(n);
                                let fits =
                                    self.parser.cursor.fits_in_line(space_width + word_width);

                                would_wrap = !fits;

                                A::ENDING_SPACES || fits
                            } else {
                                A::ENDING_SPACES
                            };

                            if render_whitespace {
                                // take as many spaces as possible and save the rest in state
                                let n = if would_wrap { n.saturating_sub(1) } else { n };
                                let spaces_to_render = self.count_widest_space_seq(n);

                                if spaces_to_render > 0 {
                                    let space_width = self.parser.config.consume(spaces_to_render);
                                    let _ = self.move_cursor(space_width as i32);
                                    let carried = n - spaces_to_render;

                                    if carried == 0 {
                                        self.next_token();
                                    } else {
                                        // n > 0 only if not every space was rendered
                                        self.finish(Token::Whitespace(carried));
                                    }

                                    return Some(RenderElement::Space(space_width));
                                } else {
                                    // there are spaces to render but none fit the line
                                    // eat one as a newline and stop
                                    if n > 1 {
                                        self.finish(Token::Whitespace(n - 1));
                                    } else {
                                        self.finish_wrapped();
                                    }
                                }
                            } else if would_wrap {
                                self.finish_wrapped();
                            } else {
                                // nothing, process next token
                                self.next_token();
                            }
                        }

                        Token::Break(c) => {
                            let fits = if let Some(word_width) = self.next_word_width() {
                                self.parser.cursor.fits_in_line(word_width)
                            } else {
                                // Next token is not a Word, consume Break and continue
                                true
                            };

                            if fits {
                                self.next_token();
                            } else if let Some(c) = c {
                                // If a Break contains a character, display it if the next
                                // Word token does not fit the line.
                                let width = self.str_width(c);
                                if self.move_cursor(width as i32).is_ok() {
                                    self.finish_wrapped();
                                    return Some(RenderElement::PrintedCharacters(c, width));
                                } else {
                                    // this line is done
                                    self.finish(Token::Word(c));
                                }
                            } else {
                                // this line is done
                                self.finish_wrapped();
                            }
                        }

                        Token::Word(w) => {
                            let width = self.str_width(w);
                            if self.move_cursor(width as i32).is_ok() {
                                // we can move the cursor here since Word doesn't depend on it
                                self.parser.first_word = false;
                                self.parser.current_token = State::Word(w);
                            } else if self.parser.first_word {
                                self.parser.first_word = false;
                                self.parser.current_token = State::FirstWord(w);
                            } else {
                                self.finish(token);
                            }
                        }

                        Token::Tab => {
                            let sp_width = self.parser.cursor.next_tab_width();

                            let tab_width = match self.move_cursor(sp_width as i32) {
                                Ok(width) => {
                                    self.next_token();
                                    width
                                }
                                Err(width) => {
                                    // If we can't render the whole tab since we don't fit in the line,
                                    // render it using all the available space - it will be < tab size.
                                    self.finish_wrapped();
                                    width
                                }
                            };

                            // don't count tabs as spaces
                            return Some(RenderElement::Space(tab_width as u32));
                        }

                        #[cfg(feature = "ansi")]
                        Token::EscapeSequence(seq) => {
                            self.next_token();
                            match seq {
                                AnsiSequence::SetGraphicsMode(vec) => {
                                    if let Some(sgr) = try_parse_sgr(vec.as_slice()) {
                                        return Some(RenderElement::Sgr(sgr));
                                    }
                                }

                                AnsiSequence::CursorForward(n) => {
                                    // Cursor movement can't rely on the text, as it's permitted
                                    // to move the cursor outside of the current line.
                                    // Example:
                                    // (| denotes the cursor, [ and ] are the limits of the line):
                                    // [Some text|    ]
                                    // Cursor forward 2 characters
                                    // [Some text  |  ]
                                    let delta = (n * self.str_width(" ")) as i32;
                                    match self.move_cursor(delta) {
                                        Ok(delta) | Err(delta) => {
                                            return Some(RenderElement::MoveCursor(delta));
                                        }
                                    }
                                }

                                AnsiSequence::CursorBackward(n) => {
                                    // The above poses an issue with variable-width fonts.
                                    // If cursor movement ignores the variable width, the cursor
                                    // will be placed in positions other than glyph boundaries.
                                    let delta = -((n * self.str_width(" ")) as i32);
                                    match self.move_cursor(delta) {
                                        Ok(delta) | Err(delta) => {
                                            return Some(RenderElement::MoveCursor(delta));
                                        }
                                    }
                                }

                                _ => {
                                    // ignore for now
                                }
                            }
                        }

                        Token::NewLine | Token::CarriageReturn => {
                            // we're done
                            self.finish(token);
                        }
                    }
                }

                State::Word(w) => {
                    // need to update the space config
                    if let Some((space_pos, _)) =
                        w.char_indices().find(|(_, c)| *c == SPEC_CHAR_NBSP)
                    {
                        if space_pos == 0 {
                            if let Some(word) = w.get(SPEC_CHAR_NBSP.len_utf8()..) {
                                self.parser.current_token = State::Word(word);
                            } else {
                                self.next_token();
                            }
                            let sp_width = self.parser.config.consume(1);

                            return Some(RenderElement::Space(sp_width));
                        } else {
                            let word = unsafe { w.get_unchecked(0..space_pos) };
                            self.parser.current_token =
                                State::Word(unsafe { w.get_unchecked(space_pos..) });

                            return Some(RenderElement::PrintedCharacters(
                                word,
                                self.str_width(word),
                            ));
                        }
                    } else {
                        self.next_token();

                        // FIXME: Maybe this state should hold on to the total word width
                        return Some(RenderElement::PrintedCharacters(w, self.str_width(w)));
                    }
                }

                State::FirstWord(w) => {
                    let mut start_idx = 0;
                    let mut width = 0;
                    for c in w.chars() {
                        let end_idx = start_idx + c.len_utf8();

                        let char_width = if c == SPEC_CHAR_NBSP {
                            self.parser.config.peek_next_width(1)
                        } else {
                            let c_str = unsafe { w.get_unchecked(start_idx..end_idx) };
                            self.str_width(c_str)
                        };

                        if self.parser.cursor.fits_in_line(width + char_width) {
                            // We return the non-breaking space as a different render element
                            if c == SPEC_CHAR_NBSP {
                                return if start_idx == 0 {
                                    // we have peeked the space width, now consume it
                                    self.parser.config.consume(1);

                                    // here, width == 0 so don't need to add
                                    let _ = self.move_cursor(char_width as i32);

                                    if let Some(word) = w.get(SPEC_CHAR_NBSP.len_utf8()..) {
                                        self.parser.current_token = State::FirstWord(word);
                                    } else {
                                        self.next_token();
                                    }

                                    Some(RenderElement::Space(char_width))
                                } else {
                                    // we know the previous characters fit in the line
                                    let _ = self.move_cursor(width as i32);

                                    // New state starts with the current space
                                    self.parser.current_token =
                                        State::FirstWord(unsafe { w.get_unchecked(start_idx..) });

                                    Some(RenderElement::PrintedCharacters(
                                        unsafe { w.get_unchecked(..start_idx) },
                                        width,
                                    ))
                                };
                            }
                            width += char_width;
                        } else {
                            // `word` does not fit into the space - this can happen for first words
                            // in this case, we return the widest we can and carry the rest

                            return if start_idx == 0 {
                                // Weird case where width doesn't permit drawing anything.
                                // Consume token to avoid infinite loop.
                                self.finish_end_of_string();
                                None
                            } else {
                                // This can happen because words can be longer than the line itself.parser.
                                let _ = self.move_cursor(width as i32);
                                // `start_idx` is actually the end of the substring that fits
                                self.finish(Token::Word(unsafe { w.get_unchecked(start_idx..) }));
                                Some(RenderElement::PrintedCharacters(
                                    unsafe { w.get_unchecked(..start_idx) },
                                    width,
                                ))
                            };
                        }

                        start_idx = end_idx;
                    }

                    self.next_token();
                    let _ = self.move_cursor(width as i32);
                    return Some(RenderElement::PrintedCharacters(w, width));
                }

                State::Done => return None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        alignment::LeftAligned,
        rendering::{cursor::Cursor, space_config::UniformSpaceConfig},
        style::TabSize,
        utils::{str_width, test::size_for},
    };
    use embedded_graphics::{
        mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        primitives::Rectangle,
        text::TextRenderer,
    };

    pub fn assert_line_elements<'a>(
        parser: &mut Parser<'a>,
        carried: &mut Option<Token<'a>>,
        max_chars: u32,
        elements: &[RenderElement],
    ) {
        let style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let config = UniformSpaceConfig::new(&style);
        let cursor = Cursor::new(
            Rectangle::new(Point::zero(), size_for(Font6x9, max_chars, 1)),
            style.line_height(),
            0,
            TabSize::Spaces(4).into_pixels(&style),
        )
        .line();

        let mut line1: LineElementParser<'_, '_, _, _, LeftAligned> =
            LineElementParser::new(parser, cursor, config, carried, |s| str_width(&style, s));

        assert_eq!(line1.iter().collect::<Vec<_>>(), elements);
    }

    #[test]
    fn soft_hyphen_no_wrapping() {
        let mut parser = Parser::parse("sam\u{00AD}ple");
        let mut carried = None;

        assert_line_elements(
            &mut parser,
            &mut carried,
            6,
            &[
                RenderElement::PrintedCharacters("sam", 18),
                RenderElement::PrintedCharacters("ple", 18),
            ],
        );
    }

    #[test]
    fn soft_hyphen() {
        let mut parser = Parser::parse("sam\u{00AD}ple");
        let mut carried = None;

        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[
                RenderElement::PrintedCharacters("sam", 18),
                RenderElement::PrintedCharacters("-", 6),
            ],
        );
        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[RenderElement::PrintedCharacters("ple", 18)],
        );
    }

    #[test]
    fn nbsp_issue() {
        let mut parser = Parser::parse("a b c\u{a0}d e f");
        let mut carried = None;

        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[
                RenderElement::PrintedCharacters("a", 6),
                RenderElement::Space(6),
                RenderElement::PrintedCharacters("b", 6),
            ],
        );
        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[
                RenderElement::PrintedCharacters("c", 6),
                RenderElement::Space(6),
                RenderElement::PrintedCharacters("d", 6),
                RenderElement::Space(6),
                RenderElement::PrintedCharacters("e", 6),
            ],
        );
        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[RenderElement::PrintedCharacters("f", 6)],
        );
    }

    #[test]
    fn soft_hyphen_issue_42() {
        let mut parser =
            Parser::parse("super\u{AD}cali\u{AD}fragi\u{AD}listic\u{AD}espeali\u{AD}docious");
        let mut carried = None;

        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[RenderElement::PrintedCharacters("super", 30)],
        );
        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[
                RenderElement::PrintedCharacters("-", 6),
                RenderElement::PrintedCharacters("cali", 24),
            ],
        );
    }

    #[test]
    fn nbsp_is_rendered_as_space() {
        let mut parser = Parser::parse("glued\u{a0}words");

        assert_line_elements(
            &mut parser,
            &mut None,
            50,
            &[
                RenderElement::PrintedCharacters("glued", 30),
                RenderElement::Space(6),
                RenderElement::PrintedCharacters("words", 30),
            ],
        );
    }

    #[test]
    fn tabs() {
        let mut parser = Parser::parse("a\tword\nand\t\tanother\t");
        let mut carried = None;

        assert_line_elements(
            &mut parser,
            &mut carried,
            16,
            &[
                RenderElement::PrintedCharacters("a", 6),
                RenderElement::Space(6 * 3),
                RenderElement::PrintedCharacters("word", 24),
            ],
        );
        assert_line_elements(
            &mut parser,
            &mut carried,
            16,
            &[
                RenderElement::PrintedCharacters("and", 18),
                RenderElement::Space(6),
                RenderElement::Space(6 * 4),
                RenderElement::PrintedCharacters("another", 42),
                RenderElement::Space(6),
            ],
        );
    }

    #[test]
    fn cursor_limit() {
        let mut parser = Parser::parse("Some sample text");

        assert_line_elements(
            &mut parser,
            &mut None,
            2,
            &[RenderElement::PrintedCharacters("So", 12)],
        );
    }
}

#[cfg(all(test, feature = "ansi"))]
mod ansi_parser_tests {
    use super::{test::assert_line_elements, *};
    use crate::style::color::Rgb;

    #[test]
    fn colors() {
        let mut parser = Parser::parse("Lorem \x1b[92mIpsum");

        assert_line_elements(
            &mut parser,
            &mut None,
            100,
            &[
                RenderElement::PrintedCharacters("Lorem", 30),
                RenderElement::Space(6),
                RenderElement::Sgr(Sgr::ChangeTextColor(Rgb::new(22, 198, 12))),
                RenderElement::PrintedCharacters("Ipsum", 30),
            ],
        );
    }

    #[test]
    fn ansi_code_does_not_break_word() {
        let mut parser = Parser::parse("Lorem foo\x1b[92mbarum");

        assert_line_elements(
            &mut parser,
            &mut None,
            8,
            &[RenderElement::PrintedCharacters("Lorem", 30)],
        );

        assert_line_elements(
            &mut parser,
            &mut None,
            8,
            &[
                RenderElement::PrintedCharacters("foo", 18),
                RenderElement::Sgr(Sgr::ChangeTextColor(Rgb::new(22, 198, 12))),
                RenderElement::PrintedCharacters("barum", 30),
            ],
        );
    }
}
