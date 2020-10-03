//! Line iterator.
//!
//! Provide elements (spaces or characters) to render as long as they fit in the current line
use crate::{
    alignment::HorizontalTextAlignment,
    parser::{Parser, Token, SPEC_CHAR_NBSP},
    rendering::{cursor::Cursor, space_config::*},
    style::TabSize,
    utils::font_ext::FontExt,
};
use core::{marker::PhantomData, str::Chars};
use embedded_graphics::prelude::*;

/// Internal state used to render a line.
#[derive(Debug)]
enum State<'a> {
    /// Decide what to do next.
    ProcessToken(Token<'a>),

    /// Render a character in a word. (remaining_characters, current_character)
    Word(Chars<'a>),

    /// Signal that the renderer has finished, store the token that was consumed but not rendered.
    Done(Option<Token<'a>>),
}

/// What to draw
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RenderElement {
    /// Render a whitespace block with the given width and count
    Space(u32, u32),

    /// Render the given character
    PrintedCharacter(char),
}

/// Pixel iterator to render a single line of styled text.
#[derive(Debug)]
pub struct LineElementIterator<'a, F, SP, A>
where
    F: Font + Copy,
    SP: SpaceConfig<Font = F>,
    A: HorizontalTextAlignment,
{
    /// Position information.
    pub cursor: Cursor<F>,

    /// The text to draw.
    pub parser: Parser<'a>,

    pub(crate) pos: Point,
    current_token: State<'a>,
    config: SP,
    first_word: bool,
    alignment: PhantomData<A>,
    tab_size: TabSize<F>,
}

impl<'a, F, SP, A> LineElementIterator<'a, F, SP, A>
where
    F: Font + Copy,
    SP: SpaceConfig<Font = F>,
    A: HorizontalTextAlignment,
{
    /// Creates a new pixel iterator to draw the given character.
    #[inline]
    #[must_use]
    pub fn new(
        mut parser: Parser<'a>,
        cursor: Cursor<F>,
        config: SP,
        carried_token: Option<Token<'a>>,
        tab_size: TabSize<F>,
    ) -> Self {
        let current_token = carried_token
            .filter(|t| ![Token::NewLine, Token::CarriageReturn, Token::Break(None)].contains(t))
            .or_else(|| parser.next())
            .map_or(State::Done(None), State::ProcessToken);

        Self {
            parser,
            current_token,
            config,
            cursor,
            first_word: true,
            alignment: PhantomData,
            pos: Point::zero(),
            tab_size,
        }
    }

    fn next_token(&mut self) {
        match self.parser.next() {
            None => self.finish_end_of_string(),
            Some(t) => self.current_token = State::ProcessToken(t),
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

    fn finish_end_of_string(&mut self) {
        self.current_token = State::Done(None);
    }

    fn finish_wrapped(&mut self) {
        self.finish(Token::Break(None));
    }

    fn finish(&mut self, t: Token<'a>) {
        self.current_token = match t {
            Token::NewLine => {
                self.cursor.new_line();
                self.cursor.carriage_return();

                State::Done(Some(Token::NewLine))
            }

            Token::CarriageReturn => {
                self.cursor.carriage_return();

                State::Done(Some(Token::CarriageReturn))
            }

            c => {
                self.cursor.new_line();
                self.cursor.carriage_return();

                State::Done(Some(c))
            }
        };
    }

    fn next_word_width(&mut self) -> Option<u32> {
        let mut width = None;
        let mut lookahead = self.parser.clone();

        'lookahead: loop {
            match lookahead.next() {
                Some(Token::Word(w)) => {
                    let w = F::str_width_nocr(w);

                    width = width.map_or(Some(w), |acc| Some(acc + w));
                }
                Some(Token::Break(Some(c))) => {
                    let w = F::total_char_width(c);
                    width = width.map_or(Some(w), |acc| Some(acc + w));
                    break 'lookahead;
                }
                _ => break 'lookahead,
            }
        }

        width
    }

    fn count_widest_space_seq(&mut self, n: u32) -> u32 {
        // we could also binary search but I don't think it's worth it
        let mut spaces_to_render = 0;
        let available = self.cursor.space();
        while spaces_to_render < n && self.config.peek_next_width(spaces_to_render + 1) < available
        {
            spaces_to_render += 1;
        }

        spaces_to_render
    }
}

impl<F, SP, A> Iterator for LineElementIterator<'_, F, SP, A>
where
    F: Font + Copy,
    SP: SpaceConfig<Font = F>,
    A: HorizontalTextAlignment,
{
    type Item = RenderElement;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current_token {
                // No token being processed, get next one
                State::ProcessToken(ref token) => {
                    let token = token.clone();
                    match token {
                        Token::Whitespace(n) => {
                            // This mess decides if we want to render whitespace at all.
                            // The current horizontal alignment can ignore spaces at the beginning
                            // and end of a line.
                            let mut would_wrap = false;
                            let render_whitespace = if self.first_word {
                                A::STARTING_SPACES
                            } else if A::ENDING_SPACES {
                                true
                            } else if let Some(word_width) = self.next_word_width() {
                                // Check if space + w fits in line, otherwise it's up to config
                                let space_width = self.config.peek_next_width(n);
                                let fits = self.cursor.fits_in_line(space_width + word_width);

                                would_wrap = !fits;

                                fits
                            } else {
                                false
                            };

                            if render_whitespace {
                                // take as many spaces as possible and save the rest in state
                                let spaces_to_render = self.count_widest_space_seq(n);

                                if spaces_to_render > 0 {
                                    let space_width = self.config.consume(spaces_to_render);
                                    self.pos = self.cursor.position;
                                    self.cursor.advance_unchecked(space_width);
                                    let carried = n - spaces_to_render;

                                    if carried == 0 {
                                        self.next_token();
                                    } else {
                                        // n > 0 only if not every space was rendered
                                        self.finish(Token::Whitespace(carried));
                                    }

                                    break Some(RenderElement::Space(
                                        space_width,
                                        spaces_to_render,
                                    ));
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
                                self.cursor.fits_in_line(word_width)
                            } else {
                                // Next token is not a Word, consume Break and continue
                                true
                            };

                            if fits {
                                self.next_token();
                            } else if let Some(c) = c {
                                // If a Break contains a character, display it if the next
                                // Word token does not fit the line.
                                let pos = self.cursor.position;
                                if self.cursor.advance(F::total_char_width(c)) {
                                    self.pos = pos;
                                    self.finish_wrapped();
                                    break Some(RenderElement::PrintedCharacter(c));
                                } else {
                                    // this line is done
                                    self.finish(Token::ExtraCharacter(c));
                                }
                            } else {
                                // this line is done
                                self.finish_wrapped();
                            }
                        }

                        Token::ExtraCharacter(c) => {
                            let pos = self.cursor.position;
                            if self.cursor.advance(F::total_char_width(c)) {
                                self.pos = pos;
                                self.next_token();
                                break Some(RenderElement::PrintedCharacter(c));
                            }

                            // ExtraCharacter currently may only be the first one.
                            // If it doesn't fit, stop.
                            self.finish_end_of_string();
                        }

                        Token::Word(w) => {
                            // FIXME: this isn't exactly optimal when outside of the display area
                            if self.first_word {
                                self.first_word = false;
                                self.current_token = State::Word(w.chars());
                            } else if self.cursor.fits_in_line(F::str_width_nocr(w)) {
                                self.current_token = State::Word(w.chars());
                            } else {
                                self.finish(token);
                            }
                        }

                        Token::Tab => {
                            let sp_width = self.tab_size.next_width(self.cursor.x_in_line());
                            let tab_width = if self.cursor.advance(sp_width) {
                                self.next_token();
                                sp_width
                            } else {
                                // If we can't render the whole tab since we don't fit in the line,
                                // render it using all the available space - it will be < tab size.
                                let available_space = self.cursor.space();
                                self.finish_wrapped();
                                available_space
                            };

                            // don't count tabs as spaces
                            break Some(RenderElement::Space(tab_width, 0));
                        }

                        Token::Escape => self.next_token(),

                        Token::NewLine | Token::CarriageReturn => {
                            // we're done
                            self.finish(token);
                        }
                    }
                }

                State::Word(ref mut chars) => {
                    let word = chars.as_str();

                    match chars.next() {
                        Some(c) => {
                            let mut ret_val = None;
                            let pos = self.cursor.position;

                            if c == SPEC_CHAR_NBSP {
                                // nbsp
                                let sp_width = self.config.peek_next_width(1);

                                if self.cursor.advance(sp_width) {
                                    ret_val = Some(RenderElement::Space(sp_width, 1));
                                    self.config.consume(1); // we have peeked the value, consume it
                                }
                            } else if self.cursor.advance(F::total_char_width(c)) {
                                ret_val = Some(RenderElement::PrintedCharacter(c));
                            }

                            if ret_val.is_some() {
                                // We have something to return
                                self.pos = pos;
                                self.current_token = State::Word(chars.clone());

                                break ret_val;
                            } else if self.cursor.x_in_line() > 0 {
                                // There's already something in this line, let's carry the whole
                                // word (the part that wasn't consumed so far) to the next.
                                // This can happen because words can be longer than the line itself.
                                self.finish(Token::Word(word));
                            } else {
                                // Weird case where width doesn't permit drawing anything. Consume
                                // token to avoid infinite loop.
                                self.finish_end_of_string();
                            }
                        }

                        None => self.next_token(),
                    }
                }

                State::Done(_) => break None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::alignment::LeftAligned;
    use embedded_graphics::fonts::Font6x8;
    use embedded_graphics::primitives::Rectangle;

    #[test]
    fn soft_hyphen_no_wrapping() {
        let parser = Parser::parse("sam\u{00AD}ple");
        let config: UniformSpaceConfig<Font6x8> = UniformSpaceConfig::default();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 6 - 1, 8)), 0);

        let iter: LineElementIterator<'_, _, _, LeftAligned> =
            LineElementIterator::new(parser, cursor, config, None, TabSize::default());

        assert_eq!(
            iter.collect::<Vec<RenderElement>>(),
            vec![
                RenderElement::PrintedCharacter('s'),
                RenderElement::PrintedCharacter('a'),
                RenderElement::PrintedCharacter('m'),
                RenderElement::PrintedCharacter('p'),
                RenderElement::PrintedCharacter('l'),
                RenderElement::PrintedCharacter('e'),
            ]
        );
    }

    fn collect_mut<I: Iterator<Item = T>, T>(iter: &mut I) -> Vec<T> {
        let mut v = Vec::new();
        v.extend(iter);

        v
    }

    #[test]
    fn soft_hyphen() {
        let parser = Parser::parse("sam\u{00AD}ple");
        let config: UniformSpaceConfig<Font6x8> = UniformSpaceConfig::default();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(6 * 6 - 2, 16)), 0);

        let mut line1: LineElementIterator<'_, _, _, LeftAligned> =
            LineElementIterator::new(parser, cursor, config, None, TabSize::default());

        assert_eq!(
            collect_mut(&mut line1),
            vec![
                RenderElement::PrintedCharacter('s'),
                RenderElement::PrintedCharacter('a'),
                RenderElement::PrintedCharacter('m'),
                RenderElement::PrintedCharacter('-'),
            ]
        );

        assert_eq!(line1.cursor.position, Point::new(0, 8));

        let carried = line1.remaining_token();
        let line2: LineElementIterator<'_, _, _, LeftAligned> = LineElementIterator::new(
            line1.parser,
            line1.cursor,
            config,
            carried,
            TabSize::default(),
        );

        assert_eq!(
            line2.collect::<Vec<RenderElement>>(),
            vec![
                RenderElement::PrintedCharacter('p'),
                RenderElement::PrintedCharacter('l'),
                RenderElement::PrintedCharacter('e'),
            ]
        );
    }

    #[test]
    fn soft_hyphen_issue_42() {
        let parser =
            Parser::parse("super\u{AD}cali\u{AD}fragi\u{AD}listic\u{AD}espeali\u{AD}docious");
        let config: UniformSpaceConfig<Font6x8> = UniformSpaceConfig::default();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(5 * 6 - 1, 16)), 0);

        let mut line1: LineElementIterator<'_, _, _, LeftAligned> =
            LineElementIterator::new(parser, cursor, config, None, TabSize::default());

        assert_eq!(
            collect_mut(&mut line1),
            vec![
                RenderElement::PrintedCharacter('s'),
                RenderElement::PrintedCharacter('u'),
                RenderElement::PrintedCharacter('p'),
                RenderElement::PrintedCharacter('e'),
                RenderElement::PrintedCharacter('r'),
            ]
        );

        assert_eq!(line1.cursor.position, Point::new(0, 8));

        let carried = line1.remaining_token();
        let line2: LineElementIterator<'_, _, _, LeftAligned> = LineElementIterator::new(
            line1.parser,
            line1.cursor,
            config,
            carried,
            TabSize::default(),
        );

        assert_eq!(
            line2.collect::<Vec<RenderElement>>(),
            vec![
                RenderElement::PrintedCharacter('-'),
                RenderElement::PrintedCharacter('c'),
                RenderElement::PrintedCharacter('a'),
                RenderElement::PrintedCharacter('l'),
                RenderElement::PrintedCharacter('i'),
            ]
        );
    }

    #[test]
    fn nbsp_is_rendered_as_space() {
        let text = "glued\u{a0}words";
        let parser = Parser::parse(text);
        let config: UniformSpaceConfig<Font6x8> = UniformSpaceConfig::default();

        let cursor = Cursor::new(
            Rectangle::new(
                Point::zero(),
                Point::new(text.chars().count() as i32 * 6 - 1, 16),
            ),
            0,
        );

        let mut line1: LineElementIterator<'_, _, _, LeftAligned> =
            LineElementIterator::new(parser, cursor, config, None, TabSize::default());

        assert_eq!(
            collect_mut(&mut line1),
            vec![
                RenderElement::PrintedCharacter('g'),
                RenderElement::PrintedCharacter('l'),
                RenderElement::PrintedCharacter('u'),
                RenderElement::PrintedCharacter('e'),
                RenderElement::PrintedCharacter('d'),
                RenderElement::Space(6, 1),
                RenderElement::PrintedCharacter('w'),
                RenderElement::PrintedCharacter('o'),
                RenderElement::PrintedCharacter('r'),
                RenderElement::PrintedCharacter('d'),
                RenderElement::PrintedCharacter('s'),
            ]
        );
    }

    #[test]
    fn tabs() {
        let text = "a\tword\nand\t\tanother\t";
        let parser = Parser::parse(text);
        let config: UniformSpaceConfig<Font6x8> = UniformSpaceConfig::default();

        let cursor = Cursor::new(Rectangle::new(Point::zero(), Point::new(16 * 6 - 1, 16)), 0);

        let mut line1: LineElementIterator<'_, _, _, LeftAligned> =
            LineElementIterator::new(parser, cursor, config, None, TabSize::default());

        assert_eq!(
            collect_mut(&mut line1),
            vec![
                RenderElement::PrintedCharacter('a'),
                RenderElement::Space(6 * 3, 0),
                RenderElement::PrintedCharacter('w'),
                RenderElement::PrintedCharacter('o'),
                RenderElement::PrintedCharacter('r'),
                RenderElement::PrintedCharacter('d'),
            ]
        );

        let carried = line1.remaining_token();
        let mut line2: LineElementIterator<'_, _, _, LeftAligned> = LineElementIterator::new(
            line1.parser,
            line1.cursor,
            config,
            carried,
            TabSize::default(),
        );

        assert_eq!(
            collect_mut(&mut line2),
            vec![
                RenderElement::PrintedCharacter('a'),
                RenderElement::PrintedCharacter('n'),
                RenderElement::PrintedCharacter('d'),
                RenderElement::Space(6, 0),
                RenderElement::Space(6 * 4, 0),
                RenderElement::PrintedCharacter('a'),
                RenderElement::PrintedCharacter('n'),
                RenderElement::PrintedCharacter('o'),
                RenderElement::PrintedCharacter('t'),
                RenderElement::PrintedCharacter('h'),
                RenderElement::PrintedCharacter('e'),
                RenderElement::PrintedCharacter('r'),
                RenderElement::Space(6, 0),
            ]
        );
    }
}
