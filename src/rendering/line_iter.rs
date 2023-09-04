//! Line iterator.
//!
//! Turns a token stream into a number of events. A single `LineElementParser` object operates on
//! a single line and is responsible for handling word wrapping, eating leading/trailing whitespace,
//! handling tab characters, soft wrapping characters, non-breaking spaces, etc.
use crate::{
    parser::{ChangeTextStyle, Parser, Token, SPEC_CHAR_NBSP},
    plugin::{PluginMarker as Plugin, PluginWrapper},
    rendering::{cursor::LineCursor, space_config::SpaceConfig},
    style::TextBoxStyle,
};
use az::{SaturatingAs, SaturatingCast};
use embedded_graphics::{pixelcolor::Rgb888, prelude::PixelColor};

/// Parser to break down a line into primitive elements used by measurement and rendering.
#[derive(Debug)]
#[must_use]
pub(crate) struct LineElementParser<'a, 'b, M, C>
where
    C: PixelColor,
{
    /// Position information.
    pub cursor: LineCursor,

    parser: &'b mut Parser<'a, C>,

    spaces: SpaceConfig,
    empty: bool,
    plugin: &'b PluginWrapper<'a, M, C>,
    style: &'b TextBoxStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEndType {
    NewLine,
    CarriageReturn,
    EndOfText,
    LineBreak,
}

pub trait ElementHandler {
    type Error;
    type Color: PixelColor;

    /// Returns the width of the given string in pixels.
    fn measure(&self, st: &str) -> u32;

    /// A whitespace block with the given width.
    fn whitespace(&mut self, _st: &str, _space_count: u32, _width: u32) -> Result<(), Self::Error> {
        Ok(())
    }

    /// A string of printable characters.
    fn printed_characters(&mut self, _st: &str, _width: Option<u32>) -> Result<(), Self::Error> {
        Ok(())
    }

    /// A cursor movement event.
    fn move_cursor(&mut self, _by: i32) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Text style change
    fn change_text_style(
        &mut self,
        _change: ChangeTextStyle<Self::Color>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, 'b, M, C> LineElementParser<'a, 'b, M, C>
where
    C: PixelColor + From<Rgb888>,
    M: Plugin<'a, C>,
{
    /// Creates a new element parser.
    #[inline]
    pub fn new(
        parser: &'b mut Parser<'a, C>,
        plugin: &'b PluginWrapper<'a, M, C>,
        cursor: LineCursor,
        spaces: SpaceConfig,
        style: &'b TextBoxStyle,
    ) -> Self {
        Self {
            parser,
            spaces,
            cursor,
            empty: true,
            plugin,
            style,
        }
    }

    fn next_word_width<E: ElementHandler>(&mut self, handler: &E) -> Option<u32> {
        let mut width = None;

        // This looks extremely inefficient.
        let lookahead = self.plugin.clone();
        let mut lookahead_parser = self.parser.clone();

        // We don't want to count the current token.
        lookahead.consume_peeked_token();

        'lookahead: loop {
            match lookahead.peek_token(&mut lookahead_parser) {
                Some(Token::Word(w)) => {
                    *width.get_or_insert(0) += handler.measure(w);
                }

                Some(Token::Break(w, _original)) => {
                    *width.get_or_insert(0) += handler.measure(w);

                    break 'lookahead;
                }

                Some(Token::ChangeTextStyle(_)) | Some(Token::MoveCursor { .. }) => {}

                _ => break 'lookahead,
            }
            lookahead.consume_peeked_token();
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
                debug_assert!(w.is_char_boundary(idx));
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

    fn next_word_fits<E: ElementHandler>(&self, space_width: i32, handler: &E) -> bool {
        let mut cursor = self.cursor.clone();
        let mut spaces = self.spaces;

        let mut exit = false;

        // This looks extremely inefficient.
        let lookahead = self.plugin.clone();
        let mut lookahead_parser = self.parser.clone();

        // We don't want to count the current token.
        lookahead.consume_peeked_token();

        if cursor.move_cursor(space_width).is_err() {
            return false;
        }
        while !exit {
            let width = match lookahead.peek_token(&mut lookahead_parser) {
                Some(Token::Word(w)) | Some(Token::Break(w, _)) => {
                    exit = true;
                    handler.measure(w).saturating_as()
                }

                Some(Token::Whitespace(n, _)) => spaces.consume(n).saturating_as(),
                Some(Token::Tab) => cursor.next_tab_width().saturating_as(),

                Some(Token::MoveCursor { chars, .. }) => {
                    chars * handler.measure(" ").saturating_as::<i32>()
                }

                Some(Token::ChangeTextStyle(_)) => 0,

                _ => return false,
            };

            lookahead.consume_peeked_token();
            if cursor.move_cursor(width).is_err() {
                return false;
            }
        }

        true
    }

    fn render_trailing_spaces(&self) -> bool {
        self.style.trailing_spaces
    }

    fn render_leading_spaces(&self) -> bool {
        self.style.leading_spaces
    }

    fn draw_whitespace<E: ElementHandler>(
        &mut self,
        handler: &mut E,
        string: &'a str,
        space_count: u32,
        space_width: u32,
    ) -> Result<bool, E::Error> {
        if self.empty && !self.render_leading_spaces() {
            handler.whitespace(string, space_count, 0)?;
            return Ok(false);
        }
        let signed_width = space_width.saturating_as();
        let draw_whitespace = (self.empty && self.render_leading_spaces())
            || self.render_trailing_spaces()
            || self.next_word_fits(signed_width, handler);

        match self.move_cursor(signed_width) {
            Ok(moved) => {
                handler.whitespace(
                    string,
                    space_count,
                    moved.saturating_as::<u32>() * draw_whitespace as u32,
                )?;
            }

            Err(moved) => {
                let single = space_width / space_count;
                let consumed = moved as u32 / single;
                if consumed > 0 {
                    let (pos, _) = string.char_indices().nth(consumed as usize).unwrap();
                    let consumed_str = unsafe {
                        // SAFETY: Pos is a valid index, we just got it
                        string.get_unchecked(0..pos)
                    };
                    let consumed_width = consumed * single;

                    let _ = self.move_cursor(consumed_width.saturating_as());
                    handler.whitespace(
                        consumed_str,
                        consumed,
                        consumed_width * self.render_trailing_spaces() as u32,
                    )?;
                }

                self.plugin
                    .consume_partial((consumed + 1).min(space_count) as usize);
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn draw_tab<E: ElementHandler>(
        &mut self,
        handler: &mut E,
        space_width: u32,
    ) -> Result<(), E::Error> {
        if self.empty && !self.render_leading_spaces() {
            return Ok(());
        }

        let draw_whitespace = (self.empty && self.render_leading_spaces())
            || self.render_trailing_spaces()
            || self.next_word_fits(space_width.saturating_as(), handler);

        match self.move_cursor(space_width.saturating_cast()) {
            Ok(moved) if draw_whitespace => handler.whitespace("\t", 0, moved.saturating_as())?,

            Ok(moved) | Err(moved) => {
                handler.move_cursor(moved.saturating_as())?;
            }
        }
        Ok(())
    }

    fn peek_next_token(&mut self) -> Option<Token<'a, C>> {
        self.plugin.peek_token(self.parser)
    }

    fn consume_token(&mut self) {
        self.plugin.consume_peeked_token();
    }

    #[inline]
    pub fn process<E: ElementHandler<Color = C>>(
        &mut self,
        handler: &mut E,
    ) -> Result<LineEndType, E::Error> {
        while let Some(token) = self.peek_next_token() {
            match token {
                Token::Whitespace(n, seq) => {
                    let space_width = self.spaces.consume(n);
                    if self.draw_whitespace(handler, seq, n, space_width)? {
                        return Ok(LineEndType::LineBreak);
                    }
                }

                Token::Tab => {
                    let space_width = self.cursor.next_tab_width();
                    self.draw_tab(handler, space_width)?;
                }

                Token::Break(c, _original) => {
                    if let Some(word_width) = self.next_word_width(handler) {
                        if !self.cursor.fits_in_line(word_width) || self.empty {
                            // this line is done, decide how to end

                            // If the next Word token does not fit the line, display break character
                            let width = handler.measure(c);
                            if self.move_cursor(width.saturating_as()).is_ok() {
                                if let Some(Token::Break(c, _)) = self.plugin.render_token(token) {
                                    handler.printed_characters(c, Some(width))?;
                                }
                                self.consume_token();
                            }

                            if !self.empty {
                                return Ok(LineEndType::LineBreak);
                            }
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
                                self.consume_token();
                                return Ok(LineEndType::LineBreak);
                            }
                            other => other,
                        }
                    } else {
                        // word wrapping - push this word to the next line
                        return Ok(LineEndType::LineBreak);
                    };

                    self.empty = false;

                    if let Some(Token::Word(word)) = self.plugin.render_token(Token::Word(word)) {
                        self.process_word(handler, word)?;
                    }

                    if remainder.is_some() {
                        // Consume what was printed.
                        self.plugin.consume_partial(word.len());
                        return Ok(LineEndType::LineBreak);
                    }
                }

                // Cursor movement can't rely on the text, as it's permitted
                // to move the cursor outside of the current line.
                // Example:
                // (| denotes the cursor, [ and ] are the limits of the line):
                // [Some text|    ]
                // Cursor forward 2 characters
                // [Some text  |  ]
                Token::MoveCursor {
                    chars,
                    draw_background: true,
                } => {
                    let delta = chars * handler.measure(" ").saturating_as::<i32>();
                    match self.move_cursor(delta) {
                        Ok(delta) | Err(delta) => {
                            if chars > 0 {
                                handler.whitespace("", 1, delta.saturating_as())?;
                            } else {
                                handler.move_cursor(delta)?;
                                handler.whitespace("", 1, delta.abs().saturating_as())?;
                                handler.move_cursor(delta)?;
                            }
                        }
                    }
                }

                Token::MoveCursor {
                    chars,
                    draw_background: false,
                } => {
                    let delta = chars * handler.measure(" ").saturating_as::<i32>();
                    match self.move_cursor(delta) {
                        Ok(delta) | Err(delta) => {
                            handler.move_cursor(delta)?;
                        }
                    }
                }

                Token::ChangeTextStyle(change) => handler.change_text_style(change)?,

                Token::CarriageReturn => {
                    handler.whitespace("\r", 0, 0)?;
                    self.consume_token();
                    return Ok(LineEndType::CarriageReturn);
                }

                Token::NewLine => {
                    handler.whitespace("\n", 0, 0)?;
                    self.consume_token();
                    return Ok(LineEndType::NewLine);
                }
            }
            self.consume_token();
        }

        Ok(LineEndType::EndOfText)
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
                    handler.printed_characters(word, None)?;
                }

                handler.whitespace("\u{a0}", 1, self.spaces.consume(1))?;

                // If we have anything after the space...
                if let Some(word) = w.get(space_pos + SPEC_CHAR_NBSP.len_utf8()..) {
                    return self.process_word(handler, word);
                }
            }

            None => handler.printed_characters(w, None)?,
        }

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::convert::Infallible;

    use super::*;
    use crate::{
        plugin::{NoPlugin, PluginMarker as Plugin, PluginWrapper},
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
    pub enum RenderElement<C: PixelColor> {
        Space(u32, bool),
        String(String, u32),
        MoveCursor(i32),
        ChangeTextStyle(ChangeTextStyle<C>),
    }

    impl<C: PixelColor> RenderElement<C> {
        pub fn string(st: &str, width: u32) -> Self {
            Self::String(st.to_owned(), width)
        }
    }

    struct TestElementHandler<F>
    where
        F: TextRenderer,
    {
        elements: Vec<RenderElement<F::Color>>,
        style: F,
    }

    impl<F> TestElementHandler<F>
    where
        F: TextRenderer,
    {
        fn new(style: F) -> Self {
            Self {
                elements: vec![],
                style,
            }
        }
    }

    impl<'el, F: TextRenderer> ElementHandler for TestElementHandler<F> {
        type Error = Infallible;
        type Color = F::Color;

        fn measure(&self, st: &str) -> u32 {
            str_width(&self.style, st)
        }

        fn whitespace(&mut self, _string: &str, count: u32, width: u32) -> Result<(), Self::Error> {
            self.elements
                .push(RenderElement::Space(count, (width > 0) as bool));
            Ok(())
        }

        fn printed_characters(&mut self, str: &str, width: Option<u32>) -> Result<(), Self::Error> {
            self.elements.push(RenderElement::String(
                str.to_owned(),
                width.unwrap_or_else(|| self.measure(str)),
            ));
            Ok(())
        }

        fn move_cursor(&mut self, by: i32) -> Result<(), Self::Error> {
            self.elements.push(RenderElement::MoveCursor(by));
            Ok(())
        }

        fn change_text_style(
            &mut self,
            change: ChangeTextStyle<Self::Color>,
        ) -> Result<(), Self::Error> {
            self.elements.push(RenderElement::ChangeTextStyle(change));
            Ok(())
        }
    }

    #[track_caller]
    pub(crate) fn assert_line_elements<'a, M>(
        parser: &mut Parser<'a, Rgb888>,
        max_chars: u32,
        elements: &[RenderElement<Rgb888>],
        plugin: &PluginWrapper<'a, M, Rgb888>,
    ) where
        M: Plugin<'a, Rgb888>,
    {
        let style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On.into());

        let config = SpaceConfig::new_from_renderer(&style);
        let cursor = Cursor::new(
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, max_chars, 1)),
            style.line_height(),
            LineHeight::Percent(100),
            TabSize::Spaces(4).into_pixels(&style),
        )
        .line();

        let text_box_style = TextBoxStyle::default();

        let mut handler = TestElementHandler::new(style);
        let mut line1 = LineElementParser::new(parser, plugin, cursor, config, &text_box_style);

        line1.process(&mut handler).unwrap();

        assert_eq!(handler.elements, elements);
    }

    #[test]
    fn insufficient_width_no_looping() {
        let mut parser = Parser::parse("foobar");

        let style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On.into());

        let config = SpaceConfig::new_from_renderer(&style);
        let cursor = Cursor::new(
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, 1, 1) - Size::new(1, 0)),
            style.line_height(),
            LineHeight::Percent(100),
            TabSize::Spaces(4).into_pixels(&style),
        )
        .line();

        let text_box_style = TextBoxStyle::default();

        let plugin = PluginWrapper::new(NoPlugin::<Rgb888>::new());
        let mut handler = TestElementHandler::new(style);
        let mut line1 =
            LineElementParser::new(&mut parser, &plugin, cursor, config, &text_box_style);

        line1.process(&mut handler).unwrap();

        assert_eq!(handler.elements, &[]);
    }

    #[test]
    fn soft_hyphen_no_wrapping() {
        let mut parser = Parser::parse("sam\u{00AD}ple");
        let mw = PluginWrapper::new(NoPlugin::<Rgb888>::new());

        assert_line_elements(
            &mut parser,
            6,
            &[
                RenderElement::string("sam", 18),
                RenderElement::string("ple", 18),
            ],
            &mw,
        );
    }

    #[test]
    fn soft_hyphen() {
        let mut parser = Parser::parse("sam\u{00AD}ple");
        let mw = PluginWrapper::new(NoPlugin::<Rgb888>::new());

        assert_line_elements(
            &mut parser,
            5,
            &[
                RenderElement::string("sam", 18),
                RenderElement::string("-", 6),
            ],
            &mw,
        );
        assert_line_elements(&mut parser, 5, &[RenderElement::string("ple", 18)], &mw);
    }

    #[test]
    fn soft_hyphen_wrapped() {
        let mut parser = Parser::parse("sam\u{00AD}mm");
        let mw = PluginWrapper::new(NoPlugin::<Rgb888>::new());

        assert_line_elements(&mut parser, 3, &[RenderElement::string("sam", 18)], &mw);
        assert_line_elements(
            &mut parser,
            3,
            &[
                RenderElement::string("-", 6),
                RenderElement::string("mm", 12),
            ],
            &mw,
        );
    }

    #[test]
    fn nbsp_issue() {
        let mut parser = Parser::parse("a b c\u{a0}d e f");
        let mw = PluginWrapper::new(NoPlugin::<Rgb888>::new());

        assert_line_elements(
            &mut parser,
            5,
            &[
                RenderElement::string("a", 6),
                RenderElement::Space(1, true),
                RenderElement::string("b", 6),
                RenderElement::Space(1, false),
            ],
            &mw,
        );
        assert_line_elements(
            &mut parser,
            5,
            &[
                RenderElement::string("c", 6),
                RenderElement::Space(1, true),
                RenderElement::string("d", 6),
                RenderElement::Space(1, true),
                RenderElement::string("e", 6),
            ],
            &mw,
        );
        assert_line_elements(
            &mut parser,
            5,
            &[
                RenderElement::Space(0, false), // FIXME: why 🤷‍♂️ Should have eaten in prev line
                RenderElement::string("f", 6),
            ],
            &mw,
        );
    }

    #[test]
    fn soft_hyphen_issue_42() {
        let mut parser =
            Parser::parse("super\u{AD}cali\u{AD}fragi\u{AD}listic\u{AD}espeali\u{AD}docious");
        let mw = PluginWrapper::new(NoPlugin::<Rgb888>::new());

        assert_line_elements(&mut parser, 5, &[RenderElement::string("super", 30)], &mw);
        assert_line_elements(
            &mut parser,
            5,
            &[
                RenderElement::string("-", 6),
                RenderElement::string("cali", 24),
            ],
            &mw,
        );
    }

    #[test]
    fn nbsp_is_rendered_as_space() {
        let mut parser = Parser::parse("glued\u{a0}words");
        let mw = PluginWrapper::new(NoPlugin::<Rgb888>::new());

        assert_line_elements(
            &mut parser,
            50,
            &[
                RenderElement::string("glued", 30),
                RenderElement::Space(1, true),
                RenderElement::string("words", 30),
            ],
            &mw,
        );
    }

    #[test]
    fn tabs() {
        let mut parser = Parser::parse("a\tword\nand\t\tanother\t");
        let mw = PluginWrapper::new(NoPlugin::<Rgb888>::new());

        assert_line_elements(
            &mut parser,
            16,
            &[
                RenderElement::string("a", 6),
                RenderElement::Space(0, true),
                RenderElement::string("word", 24),
                RenderElement::Space(0, false), // the newline
            ],
            &mw,
        );
        assert_line_elements(
            &mut parser,
            16,
            &[
                RenderElement::string("and", 18),
                RenderElement::Space(0, true),
                RenderElement::Space(0, true),
                RenderElement::string("another", 42),
                RenderElement::MoveCursor(6),
            ],
            &mw,
        );
    }

    #[test]
    fn space_wrapping_issue() {
        let mut parser = Parser::parse("Hello,      s");
        let mw = PluginWrapper::new(NoPlugin::<Rgb888>::new());

        assert_line_elements(
            &mut parser,
            10,
            &[
                RenderElement::string("Hello,", 36),
                RenderElement::Space(4, false),
            ],
            &mw,
        );
        assert_line_elements(
            &mut parser,
            10,
            &[RenderElement::Space(1, true), RenderElement::string("s", 6)],
            &mw,
        );
    }

    #[test]
    fn cursor_limit() {
        let mut parser = Parser::parse("Some sample text");
        let mw = PluginWrapper::new(NoPlugin::<Rgb888>::new());

        assert_line_elements(&mut parser, 2, &[RenderElement::string("So", 12)], &mw);
    }
}
