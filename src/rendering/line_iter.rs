//! Line iterator.
//!
//! Turns a token stream into a number of events. A single `LineElementParser` object operates on
//! a single line and is responsible for handling word wrapping, eating leading/trailing whitespace,
//! handling tab characters, soft wrapping characters, non-breaking spaces, etc.
use crate::{
    alignment::HorizontalAlignment,
    parser::{Parser, Token, SPEC_CHAR_NBSP},
    rendering::{cursor::LineCursor, space_config::SpaceConfig},
};
use az::{SaturatingAs, SaturatingCast};

#[cfg(feature = "ansi")]
use super::ansi::{try_parse_sgr, Sgr};
#[cfg(feature = "ansi")]
use ansi_parser::AnsiSequence;
#[cfg(feature = "ansi")]
use as_slice::AsSlice;

/// Parser to break down a line into primitive elements used by measurement and rendering.
#[derive(Debug)]
#[must_use]
pub struct LineElementParser<'a, 'b> {
    /// Position information.
    cursor: LineCursor,

    /// The text to draw.
    parser: &'b mut Parser<'a>,

    first_token: Option<Token<'a>>,
    spaces: SpaceConfig,
    alignment: HorizontalAlignment,
    empty: bool,
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
    fn move_cursor(&mut self, _by: i32) -> Result<(), Self::Error> {
        Ok(())
    }

    /// A Select Graphic Rendition code.
    #[cfg(feature = "ansi")]
    fn sgr(&mut self, _sgr: Sgr) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, 'b> LineElementParser<'a, 'b> {
    /// Creates a new element parser.
    #[inline]
    pub fn new(
        parser: &'b mut Parser<'a>,
        cursor: LineCursor,
        spaces: SpaceConfig,
        carried_token: Option<Token<'a>>,
        alignment: HorizontalAlignment,
    ) -> Self {
        let first_token = carried_token
            .filter(|t| ![Token::NewLine, Token::CarriageReturn, Token::Break(None)].contains(t));

        Self {
            parser,
            first_token,
            spaces,
            cursor,
            alignment,
            empty: true,
        }
    }
}

impl<'a> LineElementParser<'a, '_> {
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

    fn move_cursor(&mut self, by: i32) -> Result<i32, i32> {
        self.cursor.move_cursor(by)
    }

    fn longest_fitting_substr<E: ElementHandler>(
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

    fn next_word_fits<E: ElementHandler>(&self, handler: &mut E) -> bool {
        let mut lookahead = self.parser.clone();
        let mut cursor = self.cursor.clone();
        let mut spaces = self.spaces;

        let mut exit = false;
        while !exit {
            let width = match lookahead.next() {
                Some(Token::Word(w)) => {
                    exit = true;
                    handler.measure(w).saturating_as()
                }
                Some(Token::Break(Some(w))) => {
                    exit = true;
                    handler.measure(w).saturating_as()
                }

                Some(Token::Whitespace(n)) => spaces.consume(n).saturating_as(),
                Some(Token::Tab) => cursor.next_tab_width().saturating_as(),

                #[cfg(feature = "ansi")]
                Some(Token::EscapeSequence(AnsiSequence::CursorForward(by))) => by.saturating_as(),

                #[cfg(feature = "ansi")]
                Some(Token::EscapeSequence(AnsiSequence::CursorBackward(by))) => {
                    -by.saturating_as::<i32>()
                }

                #[cfg(feature = "ansi")]
                Some(Token::EscapeSequence(_)) => continue,

                _ => return false,
            };

            if cursor.move_cursor(width).is_err() {
                return false;
            }
        }

        true
    }

    fn draw_whitespace<E: ElementHandler>(
        &mut self,
        handler: &mut E,
        space_width: u32,
    ) -> Result<Option<Token<'a>>, E::Error> {
        if self.empty && self.alignment.ignores_leading_spaces() {
            return Ok(None);
        }

        match self.move_cursor(space_width.saturating_cast()) {
            Ok(moved) if self.empty => handler.whitespace(moved.saturating_as())?,
            Ok(moved) if self.next_word_fits(handler) => {
                handler.whitespace(moved.saturating_as())?
            }

            Ok(moved) | Err(moved) => {
                handler.move_cursor(moved)?;
                return Ok(Some(Token::Break(None)));
            }
        }
        Ok(None)
    }

    #[inline]
    pub fn process<E: ElementHandler>(
        &mut self,
        handler: &mut E,
    ) -> Result<Option<Token<'a>>, E::Error> {
        while let Some(token) = self.first_token.take().or_else(|| self.parser.next()) {
            match token {
                Token::Whitespace(n) => {
                    let space_width = self.spaces.consume(n);
                    if let Some(token) = self.draw_whitespace(handler, space_width)? {
                        return Ok(Some(token));
                    }
                }

                Token::Tab => {
                    let space_width = self.cursor.next_tab_width();
                    if let Some(token) = self.draw_whitespace(handler, space_width)? {
                        return Ok(Some(token));
                    }
                }

                Token::Break(c) => {
                    if let Some(word_width) = self.next_word_width(handler) {
                        if !self.cursor.fits_in_line(word_width) {
                            // this line is done, decide how to end
                            let token = if let Some(c) = c {
                                // If a Break contains a character, display it if the next
                                // Word token does not fit the line.
                                let width = handler.measure(c);
                                if self.move_cursor(width.saturating_as()).is_ok() {
                                    handler.printed_characters(c, width)?;
                                    Token::Break(None)
                                } else {
                                    Token::Word(c)
                                }
                            } else {
                                Token::Break(None)
                            };

                            return Ok(Some(token));
                        }
                    } else {
                        // Next token is not a Word, consume Break and continue
                    }
                }

                Token::Word(w) => {
                    let width = handler.measure(w);
                    let (word, remainder) = if self.move_cursor(width.saturating_as()).is_ok() {
                        // We can move the cursor here since `process_word()`
                        // doesn't depend on it.
                        (w, None)
                    } else if self.empty {
                        // This word does not fit into an empty line. Find longest part
                        // that fits and push the rest to the next line.
                        match self.longest_fitting_substr(handler, w) {
                            ("", _) => {
                                // Weird case where width doesn't permit drawing anything.
                                // End here to prevent infinite looping.
                                return Ok(None);
                            }
                            (word, remainder) => (word, remainder),
                        }
                    } else {
                        // word wrapping - push this word to the next line
                        return Ok(Some(token));
                    };

                    self.empty = false;
                    self.process_word(handler, word)?;

                    if let Some(remainder) = remainder {
                        return Ok(Some(Token::Word(remainder)));
                    }
                }

                #[cfg(feature = "ansi")]
                Token::EscapeSequence(seq) => {
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
                            let delta = (n * handler.measure(" ")).saturating_as();
                            match self.move_cursor(delta) {
                                Ok(delta) | Err(delta) => {
                                    handler.whitespace(delta.saturating_as())?;
                                }
                            }
                        }

                        AnsiSequence::CursorBackward(n) => {
                            // The above poses an issue with variable-width fonts.
                            // If cursor movement ignores the variable width, the cursor
                            // will be placed in positions other than glyph boundaries.
                            let delta = -(n * handler.measure(" ")).saturating_as::<i32>();
                            match self.move_cursor(delta) {
                                Ok(delta) | Err(delta) => {
                                    handler.move_cursor(delta)?;
                                    handler.whitespace(delta.abs().saturating_as())?;
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

        Ok(None)
    }

    fn process_word<E: ElementHandler>(
        &mut self,
        handler: &mut E,
        w: &str,
    ) -> Result<(), E::Error> {
        match w.char_indices().find(|(_, c)| *c == SPEC_CHAR_NBSP) {
            Some((space_pos, _)) => {
                // If we have anything before the space...
                if space_pos != 0 {
                    let word = unsafe {
                        // Safety: space_pos must be a character boundary
                        w.get_unchecked(0..space_pos)
                    };
                    handler.printed_characters(word, handler.measure(word))?;
                }

                handler.whitespace(self.spaces.consume(1))?;

                // If we have anything after the space...
                if let Some(word) = w.get(space_pos + SPEC_CHAR_NBSP.len_utf8()..) {
                    return self.process_word(handler, word);
                }
            }

            None => {
                handler.printed_characters(w, handler.measure(w))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::convert::Infallible;

    use super::*;
    use crate::{
        rendering::{cursor::Cursor, space_config::SpaceConfig},
        style::TabSize,
        utils::{str_width, test::size_for},
    };
    use embedded_graphics::{
        geometry::{Point, Size},
        mono_font::{ascii::FONT_6X9, MonoTextStyle},
        pixelcolor::BinaryColor,
        primitives::Rectangle,
        text::{renderer::TextRenderer, LineHeight},
    };

    #[derive(PartialEq, Eq, Debug)]
    pub(super) enum RenderElement {
        Space(u32),
        String(String, u32),
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
        let style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);

        let config = SpaceConfig::new_from_renderer(&style);
        let cursor = Cursor::new(
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, max_chars, 1)),
            style.line_height(),
            LineHeight::Percent(100),
            TabSize::Spaces(4).into_pixels(&style),
        )
        .line();

        let mut handler = TestElementHandler::new(style);
        let mut line1 = LineElementParser::new(
            parser,
            cursor,
            config,
            carried.clone(),
            HorizontalAlignment::Left,
        );

        *carried = line1.process(&mut handler).unwrap();

        assert_eq!(handler.elements, elements);
    }

    #[test]
    fn insufficient_width_no_looping() {
        let mut parser = Parser::parse("foobar");

        let style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);

        let config = SpaceConfig::new_from_renderer(&style);
        let cursor = Cursor::new(
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, 1, 1) - Size::new(1, 0)),
            style.line_height(),
            LineHeight::Percent(100),
            TabSize::Spaces(4).into_pixels(&style),
        )
        .line();

        let mut handler = TestElementHandler::new(style);
        let mut line1 =
            LineElementParser::new(&mut parser, cursor, config, None, HorizontalAlignment::Left);

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
                RenderElement::MoveCursor(6),
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
                RenderElement::MoveCursor(0),
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
                RenderElement::MoveCursor(6),
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

    use embedded_graphics::pixelcolor::Rgb888;

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
                RenderElement::Sgr(Sgr::ChangeTextColor(Rgb888::new(22, 198, 12))),
                RenderElement::string("Ipsum", 30),
            ],
        );
    }

    #[test]
    fn ansi_code_does_not_break_word() {
        let mut parser = Parser::parse("Lorem foo\x1b[92mbarum");

        let mut carried = None;
        assert_line_elements(
            &mut parser,
            &mut carried,
            8,
            &[
                RenderElement::string("Lorem", 30),
                RenderElement::MoveCursor(6),
            ],
        );

        assert_line_elements(
            &mut parser,
            &mut carried,
            8,
            &[
                RenderElement::string("foo", 18),
                RenderElement::Sgr(Sgr::ChangeTextColor(Rgb888::new(22, 198, 12))),
                RenderElement::string("barum", 30),
            ],
        );
    }
}
