//! Line iterator.
//!
//! Provide elements (spaces or characters) to render as long as they fit in the current line
use crate::{
    alignment::HorizontalTextAlignment,
    parser::{Parser, Token, SPEC_CHAR_NBSP},
    rendering::{cursor::LineCursor, space_config::SpaceConfig},
};
use core::marker::PhantomData;

#[cfg(feature = "ansi")]
use super::ansi::{try_parse_sgr, Sgr};
#[cfg(feature = "ansi")]
use ansi_parser::AnsiSequence;
#[cfg(feature = "ansi")]
use as_slice::AsSlice;

/// Internal state used to render a line.
#[derive(Debug, Clone)]
enum State<'a> {
    /// Decide what to do next.
    ProcessToken(Token<'a>),

    /// Process a string of printable characters. If the sequence is longer than the line, this
    /// state also contains the remaining sequence that is pushed to the next line.
    Word(&'a str, Option<&'a str>),

    /// Signal that the renderer has finished.
    Done,
}

impl State<'_> {
    pub fn take(&mut self) -> Self {
        core::mem::replace(self, State::Done)
    }
}

/// Parser to break down a line into primitive elements used by measurement and rendering.
#[derive(Debug)]
pub struct LineElementParser<'a, 'b, SP, A> {
    /// Position information.
    cursor: LineCursor,

    /// The text to draw.
    parser: &'b mut Parser<'a>,

    current_token: State<'a>,
    spaces: SP,
    first_word: bool,
    alignment: PhantomData<A>,
}

pub trait ElementHandler {
    type Error;

    /// Returns the width of the given string in pixels.
    fn measure(&self, st: &str) -> u32;

    /// A whitespace block with the given width.
    fn whitespace(&mut self, _width: u32) -> Result<(), Self::Error> {
        Ok(())
    }

    /// A string of printable characters.
    fn printed_characters(&mut self, _st: &str, _width: u32) -> Result<(), Self::Error> {
        Ok(())
    }

    /// A cursor movement event.
    #[cfg(feature = "ansi")]
    fn move_cursor(&mut self, _by: i32) -> Result<(), Self::Error> {
        Ok(())
    }

    /// A Select Graphic Rendition code.
    #[cfg(feature = "ansi")]
    fn sgr(&mut self, _sgr: Sgr) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, 'b, SP, A> LineElementParser<'a, 'b, SP, A>
where
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
{
    /// Creates a new element parser.
    #[inline]
    #[must_use]
    pub fn new(
        parser: &'b mut Parser<'a>,
        cursor: LineCursor,
        spaces: SP,
        carried_token: Option<Token<'a>>,
    ) -> Self {
        let current_token = carried_token
            .filter(|t| ![Token::NewLine, Token::CarriageReturn, Token::Break(None)].contains(t))
            .or_else(|| parser.next())
            .map_or(State::Done, State::ProcessToken);

        Self {
            parser,
            current_token,
            spaces,
            cursor,
            first_word: true,
            alignment: PhantomData,
        }
    }
}

impl<'a, SP, A> LineElementParser<'a, '_, SP, A>
where
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
{
    fn next_token(&mut self) {
        self.current_token = match self.parser.next() {
            None => State::Done,
            Some(t) => State::ProcessToken(t),
        };
    }

    fn next_word_width<E: ElementHandler>(&mut self, handler: &E) -> Option<u32> {
        let mut width = None;
        let mut lookahead = self.parser.clone();

        'lookahead: loop {
            match lookahead.next() {
                Some(Token::Word(w)) => {
                    let w = handler.measure(w);

                    width = width.map_or(Some(w), |acc| Some(acc + w));
                }

                Some(Token::Break(Some(c))) => {
                    let w = handler.measure(c);
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

    fn count_widest_space_seq(&self, n: u32) -> u32 {
        // we could also binary search but I don't think it's worth it
        let mut spaces_to_render = 0;
        let available = self.cursor.space();
        while spaces_to_render < n && self.spaces.peek_next_width(spaces_to_render + 1) < available
        {
            spaces_to_render += 1;
        }

        spaces_to_render
    }

    fn move_cursor(&mut self, by: i32) -> Result<i32, i32> {
        self.cursor.move_cursor(by as i32)
    }

    fn longest_fitting_word<E: ElementHandler>(
        &mut self,
        handler: &E,
        w: &'a str,
    ) -> (&'a str, Option<&'a str>) {
        let mut width = 0;
        for (idx, c) in w.char_indices() {
            let char_width = handler.measure(unsafe {
                // SAFETY: we are working on character boundaries
                w.get_unchecked(idx..idx + c.len_utf8())
            });
            if !self.cursor.fits_in_line(width + char_width) {
                return (
                    unsafe {
                        // SAFETY: we are working on character boundaries
                        w.get_unchecked(0..idx)
                    },
                    w.get(idx..),
                );
            }
            width += char_width;
        }

        (w, None)
    }

    #[inline]
    pub fn process<E: ElementHandler>(
        &mut self,
        handler: &mut E,
    ) -> Result<Option<Token<'a>>, E::Error> {
        loop {
            match self.current_token.take() {
                State::ProcessToken(token) => {
                    match token {
                        Token::Whitespace(n) => {
                            // This mess decides if we want to render whitespace at all.
                            // The current horizontal alignment can ignore spaces at the beginning
                            // and end of a line.
                            let mut would_wrap = false;
                            let render_whitespace = if self.first_word {
                                if A::STARTING_SPACES {
                                    self.first_word = false;
                                }
                                A::STARTING_SPACES
                            } else if let Some(word_width) = self.next_word_width(handler) {
                                // Check if space + w fits in line, otherwise it's up to config
                                let space_width = self.spaces.peek_next_width(n);
                                let fits = self.cursor.fits_in_line(space_width + word_width);

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
                                    let space_width = self.spaces.consume(spaces_to_render);
                                    let _ = self.move_cursor(space_width as i32);
                                    let carried = n - spaces_to_render;

                                    handler.whitespace(space_width)?;

                                    if carried == 0 {
                                        self.next_token();
                                    } else {
                                        // n > 0 only if not every space was rendered
                                        return Ok(Some(Token::Whitespace(carried)));
                                    }
                                } else {
                                    // there are spaces to render but none fit the line
                                    // eat one as a newline and stop
                                    if n > 1 {
                                        return Ok(Some(Token::Whitespace(n - 1)));
                                    } else {
                                        return Ok(Some(Token::Break(None)));
                                    }
                                }
                            } else if would_wrap {
                                return Ok(Some(Token::Break(None)));
                            } else {
                                // nothing, process next token
                                self.next_token();
                            }
                        }

                        Token::Break(c) => {
                            let fits = if let Some(word_width) = self.next_word_width(handler) {
                                self.cursor.fits_in_line(word_width)
                            } else {
                                // Next token is not a Word, consume Break and continue
                                true
                            };

                            if !fits {
                                if let Some(c) = c {
                                    // If a Break contains a character, display it if the next
                                    // Word token does not fit the line.
                                    let width = handler.measure(c);
                                    if self.move_cursor(width as i32).is_ok() {
                                        handler.printed_characters(c, width)?;
                                        return Ok(Some(Token::Break(None)));
                                    } else {
                                        // this line is done
                                        return Ok(Some(Token::Word(c)));
                                    }
                                } else {
                                    // this line is done
                                    return Ok(Some(Token::Break(None)));
                                }
                            }

                            self.next_token();
                        }

                        Token::Word(w) => {
                            let width = handler.measure(w);
                            if self.move_cursor(width as i32).is_ok() {
                                // We can move the cursor here since Word doesn't depend on it.
                                self.current_token = State::Word(w, None);
                            } else if self.first_word {
                                // This word does not fit into an empty line. Find longest part
                                // that fits and push the rest to the next line.
                                match self.longest_fitting_word(handler, w) {
                                    ("", _) => {
                                        // Weird case where width doesn't permit drawing anything.
                                        // End here to prevent infinite looping.
                                        return Ok(None);
                                    }
                                    (word, remainder) => {
                                        self.current_token = State::Word(word, remainder);
                                    }
                                }
                            } else {
                                // word wrapping - push this word to the next line
                                return Ok(Some(token));
                            }
                            self.first_word = false;
                        }

                        Token::Tab => {
                            let sp_width = self.cursor.next_tab_width();

                            let (tab_width, wrapped) = match self.move_cursor(sp_width as i32) {
                                Ok(width) => (width, false),
                                Err(width) => {
                                    // If we can't render the whole tab since we don't fit in the line,
                                    // render it using all the available space - it will be < tab size.
                                    (width, true)
                                }
                            };

                            // don't count tabs as spaces
                            handler.whitespace(tab_width as u32)?;

                            if wrapped {
                                return Ok(Some(Token::Break(None)));
                            } else {
                                self.next_token();
                            }
                        }

                        #[cfg(feature = "ansi")]
                        Token::EscapeSequence(seq) => {
                            self.next_token();
                            match seq {
                                AnsiSequence::SetGraphicsMode(vec) => {
                                    if let Some(sgr) = try_parse_sgr(vec.as_slice()) {
                                        handler.sgr(sgr)?;
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
                                    let delta = (n * handler.measure(" ")) as i32;
                                    match self.move_cursor(delta) {
                                        Ok(delta) | Err(delta) => {
                                            handler.move_cursor(delta)?;
                                        }
                                    }
                                }

                                AnsiSequence::CursorBackward(n) => {
                                    // The above poses an issue with variable-width fonts.
                                    // If cursor movement ignores the variable width, the cursor
                                    // will be placed in positions other than glyph boundaries.
                                    let delta = -((n * handler.measure(" ")) as i32);
                                    match self.move_cursor(delta) {
                                        Ok(delta) | Err(delta) => {
                                            handler.move_cursor(delta)?;
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
                            return Ok(Some(token));
                        }
                    }
                }

                State::Word(w, remainder) => {
                    if let Some(space_pos) = w
                        .char_indices()
                        .find(|(_, c)| *c == SPEC_CHAR_NBSP)
                        .map(|(idx, _)| idx)
                    {
                        if space_pos == 0 {
                            let sp_width = self.spaces.consume(1);

                            handler.whitespace(sp_width)?;
                            if let Some(word) = w.get(SPEC_CHAR_NBSP.len_utf8()..) {
                                self.current_token = State::Word(word, remainder);
                            } else if let Some(remainder) = remainder {
                                return Ok(Some(Token::Word(remainder)));
                            } else {
                                self.next_token();
                            }
                        } else {
                            let (word, rest) = unsafe {
                                (w.get_unchecked(0..space_pos), w.get_unchecked(space_pos..))
                            };
                            self.current_token = State::Word(rest, remainder);

                            handler.printed_characters(word, handler.measure(word))?;
                        }
                    } else {
                        handler.printed_characters(w, handler.measure(w))?;
                        if let Some(remainder) = remainder {
                            return Ok(Some(Token::Word(remainder)));
                        } else {
                            self.next_token();
                        }
                    }
                }

                State::Done => return Ok(None),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::convert::Infallible;

    use super::*;
    use crate::{
        alignment::LeftAligned,
        rendering::{cursor::Cursor, space_config::UniformSpaceConfig},
        style::TabSize,
        utils::{str_width, test::size_for},
    };
    use embedded_graphics::{
        geometry::Point,
        mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        prelude::Size,
        primitives::Rectangle,
        text::TextRenderer,
    };

    #[derive(PartialEq, Eq, Debug)]
    pub(super) enum RenderElement {
        Space(u32),
        String(String, u32),
        #[cfg(feature = "ansi")]
        MoveCursor(i32),
        #[cfg(feature = "ansi")]
        Sgr(Sgr),
    }

    impl RenderElement {
        pub fn string(st: &str, width: u32) -> Self {
            Self::String(st.to_owned(), width)
        }
    }

    struct TestElementHandler<F> {
        elements: Vec<RenderElement>,
        style: F,
    }

    impl<F> TestElementHandler<F> {
        fn new(style: F) -> Self {
            Self {
                elements: vec![],
                style,
            }
        }
    }

    impl<'el, F: TextRenderer> ElementHandler for TestElementHandler<F> {
        type Error = Infallible;

        fn measure(&self, st: &str) -> u32 {
            str_width(&self.style, st)
        }

        fn whitespace(&mut self, width: u32) -> Result<(), Self::Error> {
            self.elements.push(RenderElement::Space(width));
            Ok(())
        }

        fn printed_characters(&mut self, st: &str, width: u32) -> Result<(), Self::Error> {
            self.elements
                .push(RenderElement::String(st.to_owned(), width));
            Ok(())
        }

        #[cfg(feature = "ansi")]
        fn move_cursor(&mut self, by: i32) -> Result<(), Self::Error> {
            self.elements.push(RenderElement::MoveCursor(by));
            Ok(())
        }

        #[cfg(feature = "ansi")]
        fn sgr(&mut self, sgr: Sgr) -> Result<(), Self::Error> {
            self.elements.push(RenderElement::Sgr(sgr));
            Ok(())
        }
    }

    pub(super) fn assert_line_elements<'a>(
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

        let mut handler = TestElementHandler::new(style);
        let mut line1: LineElementParser<'_, '_, _, LeftAligned> =
            LineElementParser::new(parser, cursor, config, carried.clone());

        *carried = line1.process(&mut handler).unwrap();

        assert_eq!(handler.elements, elements);
    }

    #[test]
    fn insufficient_width_no_looping() {
        let mut parser = Parser::parse("foobar");

        let style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let config = UniformSpaceConfig::new(&style);
        let cursor = Cursor::new(
            Rectangle::new(Point::zero(), size_for(Font6x9, 1, 1) - Size::new(1, 0)),
            style.line_height(),
            0,
            TabSize::Spaces(4).into_pixels(&style),
        )
        .line();

        let mut handler = TestElementHandler::new(style);
        let mut line1: LineElementParser<'_, '_, _, LeftAligned> =
            LineElementParser::new(&mut parser, cursor, config, None);

        line1.process(&mut handler).unwrap();

        assert_eq!(handler.elements, &[]);
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
                RenderElement::string("sam", 18),
                RenderElement::string("ple", 18),
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
                RenderElement::string("sam", 18),
                RenderElement::string("-", 6),
            ],
        );
        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[RenderElement::string("ple", 18)],
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
                RenderElement::string("a", 6),
                RenderElement::Space(6),
                RenderElement::string("b", 6),
            ],
        );
        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[
                RenderElement::string("c", 6),
                RenderElement::Space(6),
                RenderElement::string("d", 6),
                RenderElement::Space(6),
                RenderElement::string("e", 6),
            ],
        );
        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[RenderElement::string("f", 6)],
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
            &[RenderElement::string("super", 30)],
        );
        assert_line_elements(
            &mut parser,
            &mut carried,
            5,
            &[
                RenderElement::string("-", 6),
                RenderElement::string("cali", 24),
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
                RenderElement::string("glued", 30),
                RenderElement::Space(6),
                RenderElement::string("words", 30),
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
                RenderElement::string("a", 6),
                RenderElement::Space(6 * 3),
                RenderElement::string("word", 24),
            ],
        );
        assert_line_elements(
            &mut parser,
            &mut carried,
            16,
            &[
                RenderElement::string("and", 18),
                RenderElement::Space(6),
                RenderElement::Space(6 * 4),
                RenderElement::string("another", 42),
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
            &[RenderElement::string("So", 12)],
        );
    }
}

#[cfg(all(test, feature = "ansi"))]
mod ansi_parser_tests {
    use super::{
        test::{assert_line_elements, RenderElement},
        *,
    };
    use crate::style::color::Rgb;

    #[test]
    fn colors() {
        let mut parser = Parser::parse("Lorem \x1b[92mIpsum");

        assert_line_elements(
            &mut parser,
            &mut None,
            100,
            &[
                RenderElement::string("Lorem", 30),
                RenderElement::Space(6),
                RenderElement::Sgr(Sgr::ChangeTextColor(Rgb::new(22, 198, 12))),
                RenderElement::string("Ipsum", 30),
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
            &[RenderElement::string("Lorem", 30)],
        );

        assert_line_elements(
            &mut parser,
            &mut None,
            8,
            &[
                RenderElement::string("foo", 18),
                RenderElement::Sgr(Sgr::ChangeTextColor(Rgb::new(22, 198, 12))),
                RenderElement::string("barum", 30),
            ],
        );
    }
}
