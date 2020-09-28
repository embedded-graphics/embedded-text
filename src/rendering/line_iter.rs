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
    /// Fetch next token.
    FetchNext,

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
        parser: Parser<'a>,
        cursor: Cursor<F>,
        config: SP,
        carried_token: Option<Token<'a>>,
    ) -> Self {
        Self {
            parser,
            current_token: carried_token.map_or(State::FetchNext, State::ProcessToken),
            config,
            cursor,
            first_word: true,
            alignment: PhantomData,
        }
    }

    fn next_token(&mut self) {
        self.current_token = State::FetchNext;
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

    fn try_draw_next_character(&mut self, word: &'a str) -> State<'a> {
        let mut lookahead = word.chars();
        lookahead.next().map_or(State::FetchNext, |c| {
            if c == '\u{A0}' {
                // nbsp
                let sp_width = self.config.peek_next_width(1);

                if self.cursor.advance(sp_width) {
                    self.config.consume(1); // we have peeked the value, consume it
                    return State::WordSpace(lookahead, sp_width);
                }
            } else {
                // character done, move to the next one
                let char_width = F::total_char_width(c);

                if self.cursor.advance(char_width) {
                    return State::WordChar(lookahead, c);
                }
            }

            // word wrapping, this line is done
            Self::finish(&mut self.cursor, Token::Word(word))
        })
    }

    fn finish_draw_whitespace(cursor: &mut Cursor<F>, carried: u32) -> State<'a> {
        if carried == 0 {
            State::FetchNext
        } else {
            // n > 0 only if not every space was rendered
            Self::finish(cursor, Token::Whitespace(carried))
        }
    }

    fn finish(cursor: &mut Cursor<F>, t: Token<'a>) -> State<'a> {
        match t {
            Token::NewLine => {
                cursor.new_line();
                cursor.carriage_return();

                State::Done(None)
            }

            Token::CarriageReturn => {
                cursor.carriage_return();

                State::Done(None)
            }

            c => {
                cursor.new_line();
                cursor.carriage_return();

                State::Done(Some(c))
            }
        }
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
                            } else if let Some(word_width) = self.next_word_width() {
                                // Check if space + w fits in line, otherwise it's up to config
                                let space_width = self.config.peek_next_width(n);
                                let fits = self.fits_in_line(space_width + word_width);

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

                                    self.current_token = Self::finish_draw_whitespace(
                                        &mut self.cursor,
                                        n - spaces_to_render,
                                    );

                                    break Some(RenderElement::Space(space_width));
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

                        Token::Break => {
                            // At this moment, Break tokens just ensure that there are no consecutive
                            // Word tokens. Later, they should be responsible for word wrapping if
                            // the next Word token (or non-breaking token sequences) do not fit into
                            // the line.
                            State::FetchNext
                        }

                        Token::Word(w) => {
                            // FIXME: this isn't exactly optimal when outside of the display area
                            if self.first_word {
                                self.first_word = false;

                                self.try_draw_next_character(w)
                            } else if self.fits_in_line(F::str_width_nocr(w)) {
                                self.try_draw_next_character(w)
                            } else {
                                Self::finish(&mut self.cursor, token)
                            }
                        }

                        Token::NewLine | Token::CarriageReturn => {
                            // we're done
                            Self::finish(&mut self.cursor, token)
                        }
                    }
                }

                State::WordChar(ref chars, ref c) => {
                    let c = *c;
                    let word = chars.as_str();
                    self.current_token = self.try_draw_next_character(word);

                    break Some(RenderElement::PrintedCharacter(c));
                }

                State::WordSpace(ref chars, ref width) => {
                    let width = *width;
                    let word = chars.as_str();
                    self.current_token = self.try_draw_next_character(word);

                    break Some(RenderElement::Space(width));
                }

                State::Done(_) => {
                    break None;
                }
            }
        }
    }
}
