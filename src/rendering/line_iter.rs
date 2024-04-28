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
use az::SaturatingAs;
use embedded_graphics::prelude::PixelColor;

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

    /// Returns the left offset in pixels.
    fn measure_left_offset(&self, _st: &str) -> u32;

    /// Start a new line at the given horizontal offset in pixels.
    fn left_offset(&mut self, _offset: u32);

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
    C: PixelColor,
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
        // This looks extremely inefficient.
        let lookahead = self.plugin.clone();
        let mut lookahead_parser = self.parser.clone();

        let mut width = 0;
        let mut width_set = false;

        loop {
            lookahead.consume_peeked_token();
            match lookahead.peek_token(&mut lookahead_parser) {
                Some(Token::Word(w)) => {
                    width += handler.measure(w);
                    width_set = true;
                }

                Some(Token::Break(w)) => return Some(width + handler.measure(w)),
                Some(Token::ChangeTextStyle(_)) | Some(Token::MoveCursor { .. }) => {}

                _ => {
                    return match width_set {
                        true => Some(width),
                        false => None,
                    };
                }
            }
        }
    }

    fn move_cursor(&mut self, by: i32) -> Result<i32, i32> {
        self.cursor.move_cursor(by)
    }

    fn move_cursor_forward(&mut self, by: u32) -> Result<u32, u32> {
        self.cursor.move_cursor_forward(by)
    }

    fn longest_fitting_substr<E: ElementHandler>(
        &mut self,
        handler: &E,
        w: &'a str,
    ) -> (&'a str, &'a str) {
        let mut width = 0;
        for (idx, c) in w.char_indices() {
            let char_width = handler.measure(unsafe {
                // SAFETY: we are working on character boundaries
                w.get_unchecked(idx..idx + c.len_utf8())
            });
            if !self.cursor.fits_in_line(width + char_width) {
                unsafe {
                    if w.is_char_boundary(idx) {
                        return w.split_at(idx);
                    } else {
                        core::hint::unreachable_unchecked();
                    }
                }
            }
            width += char_width;
        }

        (w, "")
    }

    fn next_word_fits<E: ElementHandler>(&self, handler: &E) -> bool {
        let mut cursor = self.cursor.clone();

        let mut spaces = self.spaces;

        // This looks extremely inefficient.
        let lookahead = self.plugin.clone();
        let mut lookahead_parser = self.parser.clone();

        let mut exit = false;
        while !exit {
            lookahead.consume_peeked_token();
            let width = match lookahead.peek_token(&mut lookahead_parser) {
                Some(Token::Word(w)) | Some(Token::Break(w)) => {
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

            if cursor.move_cursor(width).is_err() {
                return false;
            }
        }

        true
    }

    fn render_trailing_spaces(&self) -> bool {
        self.style.trailing_spaces
    }

    fn skip_leading_spaces(&self) -> bool {
        self.empty && !self.style.leading_spaces
    }

    fn draw_whitespace<E: ElementHandler>(
        &mut self,
        handler: &mut E,
        string: &'a str,
        space_count: u32,
        space_width: u32,
    ) -> Result<bool, E::Error> {
        if self.skip_leading_spaces() {
            handler.whitespace(string, space_count, 0)?;
            return Ok(false);
        }

        match self.move_cursor_forward(space_width) {
            Ok(moved) => {
                handler.whitespace(
                    string,
                    space_count,
                    moved * self.should_draw_whitespace(handler) as u32,
                )?;
                Ok(false)
            }

            Err(moved) => {
                let single = space_width / space_count;
                let consumed = moved / single;
                if consumed > 0 {
                    let consumed_str = string
                        .char_indices()
                        .nth(consumed as usize)
                        .map(|(pos, _)| unsafe {
                            // SAFETY: Pos is a valid index, we just got it
                            string.get_unchecked(0..pos)
                        })
                        .unwrap_or(string);

                    let consumed_width = consumed * single;

                    let _ = self.move_cursor_forward(consumed_width);
                    handler.whitespace(
                        consumed_str,
                        consumed,
                        consumed_width * self.render_trailing_spaces() as u32,
                    )?;
                }

                self.plugin
                    .consume_partial((consumed + 1).min(space_count) as usize);
                Ok(true)
            }
        }
    }

    fn draw_tab<E: ElementHandler>(&mut self, handler: &mut E) -> Result<(), E::Error> {
        if self.skip_leading_spaces() {
            return Ok(());
        }

        let space_width = self.cursor.next_tab_width();
        match self.move_cursor_forward(space_width) {
            Ok(moved) if self.should_draw_whitespace(handler) => {
                handler.whitespace("\t", 0, moved)?
            }

            Ok(moved) | Err(moved) => handler.move_cursor(moved as i32)?,
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
                    self.draw_tab(handler)?;
                }

                Token::Break(c) => {
                    if let Some(word_width) = self.next_word_width(handler) {
                        if !self.cursor.fits_in_line(word_width) || self.empty {
                            // this line is done, decide how to end

                            // If the next Word token does not fit the line, display break character
                            let width = handler.measure(c);
                            if self.move_cursor_forward(width).is_ok() {
                                if let Some(Token::Break(c)) = self.plugin.render_token(token) {
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
                    if self.empty {
                        // If this is the first word on the line, offset the line by
                        // the word's left negative boundary to make sure it is not clipped.
                        let offset = handler.measure_left_offset(w);
                        if offset > 0 && self.move_cursor_forward(offset).is_ok() {
                            handler.left_offset(offset);
                        };
                    }

                    let width = handler.measure(w);
                    let (word, remainder) = if self.move_cursor_forward(width).is_ok() {
                        // We can move the cursor here since `process_word()`
                        // doesn't depend on it.
                        (w, "")
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

                    if !remainder.is_empty() {
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
        mut w: &str,
    ) -> Result<(), E::Error> {
        loop {
            let mut iter = w.char_indices();
            match iter.find(|(_, c)| *c == SPEC_CHAR_NBSP) {
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
                    w = iter.as_str();
                }

                None => return handler.printed_characters(w, None),
            }
        }
    }

    fn should_draw_whitespace<E: ElementHandler>(&self, handler: &E) -> bool {
        self.empty // We know that when this function is called,
                   // an empty line means leading spaces are allowed
            || self.render_trailing_spaces()
            || self.next_word_fits(handler)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use core::fmt::Debug;
    use std::convert::Infallible;

    use super::*;
    use crate::{
        plugin::{NoPlugin, PluginMarker as Plugin, PluginWrapper},
        rendering::{cursor::Cursor, space_config::SpaceConfig},
        style::TabSize,
        utils::{str_left_offset, str_width, test::size_for},
    };
    use embedded_graphics::{
        geometry::{Point, Size},
        mono_font::{ascii::FONT_6X9, MonoTextStyle},
        pixelcolor::{BinaryColor, Rgb888},
        primitives::Rectangle,
        text::{renderer::TextRenderer, LineHeight},
    };

    #[derive(PartialEq, Eq, Debug)]
    pub enum RenderElement<C: PixelColor> {
        LeftOffset(u32),
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

        fn measure_left_offset(&self, st: &str) -> u32 {
            str_left_offset(&self.style, st)
        }

        fn left_offset(&mut self, offset: u32) {
            self.elements.push(RenderElement::LeftOffset(offset));
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

        let space_width = str_width(&style, " ");
        let config = SpaceConfig::new(space_width, None);
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

        let space_width = str_width(&style, " ");
        let config = SpaceConfig::new(space_width, None);
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

    /// A font where each glyph is 4x10 pixels, where the
    /// glyph 'j' has a left side bearing of 2 pixels (renders with negative offset)
    struct TestTextStyle {}

    impl TextRenderer for TestTextStyle {
        type Color = Rgb888;

        fn draw_string<D>(
            &self,
            text: &str,
            position: Point,
            baseline: embedded_graphics::text::Baseline,
            _target: &mut D,
        ) -> Result<Point, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color>,
        {
            return Ok(self.measure_string(text, position, baseline).next_position);
        }

        fn draw_whitespace<D>(
            &self,
            width: u32,
            position: Point,
            _baseline: embedded_graphics::text::Baseline,
            _target: &mut D,
        ) -> Result<Point, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color>,
        {
            return Ok(Point::new(position.x + width as i32, position.y));
        }

        fn measure_string(
            &self,
            text: &str,
            position: Point,
            _baseline: embedded_graphics::text::Baseline,
        ) -> embedded_graphics::text::renderer::TextMetrics {
            let offset = if text.starts_with("j") { -2 } else { 0 };
            let width = text.len() as u32 * 4;
            let top_left = Point::new(position.x + offset, position.y);
            embedded_graphics::text::renderer::TextMetrics {
                bounding_box: Rectangle::new(top_left, Size::new(width, 10)),
                next_position: Point::new(top_left.x + width as i32, position.y),
            }
        }

        fn line_height(&self) -> u32 {
            10
        }
    }

    #[test]
    fn negative_left_side_bearing_of_the_first_glyph_sets_left_offset() {
        let text = "just a jet";
        let mut parser = Parser::parse(text);
        let plugin = PluginWrapper::new(NoPlugin::<Rgb888>::new());
        let style = TestTextStyle {};
        // the glyph 'j' occupies 2 pixels because of the negative left side bearing
        // however, the first 'j' is rendered in full because of the left line offset
        let size = Size::new(4 * text.len() as u32 - 2, 10);
        let config = SpaceConfig::new(str_width(&style, " "), None);
        let cursor = Cursor::new(
            Rectangle::new(Point::zero(), size),
            style.line_height(),
            LineHeight::Percent(100),
            TabSize::Spaces(4).into_pixels(&style),
        )
        .line();

        let text_box_style = TextBoxStyle::default();

        let mut handler = TestElementHandler::new(style);
        let mut line1 =
            LineElementParser::new(&mut parser, &plugin, cursor, config, &text_box_style);

        line1.process(&mut handler).unwrap();

        assert_eq!(
            handler.elements,
            &[
                RenderElement::LeftOffset(2),
                RenderElement::string("just", 14),
                RenderElement::Space(1, true),
                RenderElement::string("a", 4),
                RenderElement::Space(1, true),
                RenderElement::string("jet", 10),
            ]
        );
    }
}
