//! Line iterator.
//!
//! Provide tokens to render as long as they fit in the current line
use crate::{
    alignment::HorizontalTextAlignment,
    parser::{Parser, Token},
    rendering::{cursor::Cursor, space_config::*},
    utils::font_ext::FontExt,
};
use core::{marker::PhantomData, str::Chars};
use embedded_graphics::prelude::*;

/// Internal state used to render a line.
#[derive(Debug)]
pub enum State<'a> {
    /// Decide what to do next.
    ProcessToken(Token<'a>),

    /// Render a character in a word. (remaining_characters, current_character)
    WordChar(Chars<'a>, char),

    /// Render a printed space in a word. (remaining_characters, rendered_width)
    WordSpace(Chars<'a>, u32),

    /// Signal that the renderer has finished, store the token that was consumed but not rendered.
    Done(Option<Token<'a>>),
}

/// What to draw
#[derive(Copy, Clone, Debug)]
pub enum RenderElement {
    /// Render a whitespace block with the given width
    Space(u32),

    /// Render the given character
    PrintedCharacter(char),
}

/// Pixel iterator to render a single line of styled text.
#[derive(Debug)]
pub struct LineElementIterator<'a, F, SP, A>
where
    F: Font + Copy,
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
{
    /// Position information.
    pub cursor: Cursor<F>,

    /// The text to draw.
    pub parser: Parser<'a>,

    current_token: State<'a>,
    config: SP,
    first_word: bool,
    alignment: PhantomData<A>,
}

impl<'a, F, SP, A> LineElementIterator<'a, F, SP, A>
where
    F: Font + Copy,
    SP: SpaceConfig,
    A: HorizontalTextAlignment,
{
    /// Creates a new pixel iterator to draw the given character.
    #[inline]
    #[must_use]
    pub fn new(
        mut parser: Parser<'a>,
        mut cursor: Cursor<F>,
        config: SP,
        carried_token: Option<Token<'a>>,
    ) -> Self {
        let current_token = carried_token
            .or_else(|| parser.next())
            .or_else(|| {
                cursor.new_line();
                cursor.carriage_return();
                None
            })
            .map_or(State::Done(None), State::ProcessToken);

        Self {
            parser,
            current_token,
            config,
            cursor,
            first_word: true,
            alignment: PhantomData,
        }
    }

    fn next_token(&mut self) {
        match self.parser.next() {
            None => self.finish(Token::NewLine),
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

    fn try_draw_next_character(&mut self, word: &'a str) {
        let mut lookahead = word.chars();
        match lookahead.next() {
            None => self.next_token(),
            Some(c) => {
                if c == '\u{A0}' {
                    // nbsp
                    let sp_width = self.config.peek_next_width(1);

                    if self.cursor.advance(sp_width) {
                        self.config.consume(1); // we have peeked the value, consume it
                        self.current_token = State::WordSpace(lookahead, sp_width);
                        return;
                    }
                } else {
                    // character done, move to the next one
                    let char_width = F::total_char_width(c);

                    if self.cursor.advance(char_width) {
                        self.current_token = State::WordChar(lookahead, c);
                        return;
                    }
                }

                // word wrapping, this line is done
                self.finish(Token::Word(word));
            }
        };
    }

    fn finish(&mut self, t: Token<'a>) {
        self.current_token = match t {
            Token::NewLine => {
                self.cursor.new_line();
                self.cursor.carriage_return();

                State::Done(None)
            }

            Token::CarriageReturn => {
                self.cursor.carriage_return();

                State::Done(None)
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
            let token = lookahead.next();
            match token {
                Some(Token::Word(w)) => {
                    let w = F::str_width_nocr(w);

                    width = width.map_or(Some(w), |acc| Some(acc + w));
                }
                _ => break 'lookahead,
            };
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
    SP: SpaceConfig,
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
                                    self.cursor.advance_unchecked(space_width);
                                    let carried = n - spaces_to_render;

                                    if carried == 0 {
                                        self.next_token();
                                    } else {
                                        // n > 0 only if not every space was rendered
                                        self.finish(Token::Whitespace(carried));
                                    }

                                    break Some(RenderElement::Space(space_width));
                                } else {
                                    // there are spaces to render but none fit the line
                                    // eat one as a newline and stop
                                    self.finish(if n > 1 {
                                        Token::Whitespace(n - 1)
                                    } else {
                                        Token::NewLine
                                    });
                                }
                            } else if would_wrap {
                                self.finish(Token::NewLine);
                            } else {
                                // nothing, process next token
                                self.next_token();
                            }
                        }

                        Token::Break => {
                            // At this moment, Break tokens just ensure that there are no consecutive
                            // Word tokens. Later, they should be responsible for word wrapping if
                            // the next Word token (or non-breaking token sequences) do not fit into
                            // the line.
                            self.next_token();
                        }

                        Token::Word(w) => {
                            // FIXME: this isn't exactly optimal when outside of the display area
                            if self.first_word {
                                self.first_word = false;

                                self.try_draw_next_character(w);
                            } else if self.cursor.fits_in_line(F::str_width_nocr(w)) {
                                self.try_draw_next_character(w);
                            } else {
                                self.finish(token);
                            }
                        }

                        Token::NewLine | Token::CarriageReturn => {
                            // we're done
                            self.finish(token);
                        }
                    }
                }

                State::WordChar(ref chars, ref c) => {
                    let c = *c;
                    let word = chars.as_str();
                    self.try_draw_next_character(word);

                    break Some(RenderElement::PrintedCharacter(c));
                }

                State::WordSpace(ref chars, ref width) => {
                    let width = *width;
                    let word = chars.as_str();
                    self.try_draw_next_character(word);

                    break Some(RenderElement::Space(width));
                }

                State::Done(_) => {
                    break None;
                }
            }
        }
    }
}
