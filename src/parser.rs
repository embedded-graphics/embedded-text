//! Parse text into words, newlines and whitespace sequences.
//!
//! ```rust
//! use embedded_text::parser::{Parser, Token};
//!
//! let parser = Parser::parse("Hello, world!\n");
//! let tokens = parser.collect::<Vec<Token<'_>>>();
//!
//! assert_eq!(vec![
//!     Token::Word("Hello,"),
//!     Token::Whitespace(1),
//!     Token::Word("world!"),
//!     Token::NewLine
//! ], tokens);
//! ```
use core::str::Chars;

/// A text token
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'a> {
    /// A newline character.
    NewLine,

    /// A number of whitespace characters.
    Whitespace(u32),

    /// A word (a sequence of non-whitespace characters).
    Word(&'a str),
}

/// Text parser. Turns a string into a stream of [`Token`] objects.
///
/// [`Token`]: enum.Token.html
#[derive(Clone, Debug)]
pub struct Parser<'a> {
    inner: Chars<'a>,
}

impl<'a> Parser<'a> {
    /// Create a new parser object to process the given piece of text.
    #[inline]
    #[must_use]
    pub fn parse(text: &'a str) -> Self {
        Self {
            inner: text.chars(),
        }
    }

    /// Returns the next token without advancing.
    #[inline]
    #[must_use]
    pub fn peek(&self) -> Option<Token> {
        self.clone().next()
    }

    /// Returns true if there are no tokens to process.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.as_str().is_empty()
    }

    /// Returns the number of unprocessed bytes.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.inner.as_str().len()
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let string = self.inner.as_str();
        self.inner.next().map(|c| match c {
            '\n' => Token::NewLine,

            c if c.is_whitespace() => {
                let mut len = 0;
                for (idx, c) in string.char_indices() {
                    if c.is_whitespace() && c != '\n' {
                        len += 1;
                    } else {
                        // consume the whitespaces
                        self.inner = unsafe { string.get_unchecked(idx..) }.chars();
                        return Token::Whitespace(len);
                    }
                }

                // consume all the text
                self.inner = "".chars();
                Token::Whitespace(len)
            }

            _ => {
                for (possible_end, c) in string.char_indices() {
                    if c.is_whitespace() {
                        let (word, rest) = unsafe {
                            // don't worry
                            (
                                string.get_unchecked(0..possible_end),
                                string.get_unchecked(possible_end..),
                            )
                        };
                        self.inner = rest.chars();
                        return Token::Word(word);
                    }
                }

                // consume all the text
                self.inner = "".chars();
                Token::Word(&string)
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Parser, Token};
    #[test]
    fn parse() {
        // (At least) for now, \r is considered a whitespace
        let text = "Lorem ipsum \r dolor sit amet, conse😅ctetur adipiscing\nelit";

        assert_eq!(
            Parser::parse(text).collect::<Vec<Token>>(),
            vec![
                Token::Word("Lorem"),
                Token::Whitespace(1),
                Token::Word("ipsum"),
                Token::Whitespace(3),
                Token::Word("dolor"),
                Token::Whitespace(1),
                Token::Word("sit"),
                Token::Whitespace(1),
                Token::Word("amet,"),
                Token::Whitespace(1),
                Token::Word("conse😅ctetur"),
                Token::Whitespace(1),
                Token::Word("adipiscing"),
                Token::NewLine,
                Token::Word("elit"),
            ]
        );
    }

    #[test]
    fn parse_multibyte_last() {
        // (At least) for now, \r is considered a whitespace
        let text = "test😅";

        assert_eq!(
            Parser::parse(text).collect::<Vec<Token>>(),
            vec![Token::Word("test😅"),]
        );
    }
}
