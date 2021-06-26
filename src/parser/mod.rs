//! Parse text into words, newlines and whitespace sequences.
//!
//! ```rust,ignore
//! use embedded_text::parser::{Parser, Token};
//!
//! let parser = Parser::parse("Hello, world!\n");
//! let tokens = parser.collect::<Vec<Token<'_>>>();
//!
//! assert_eq!(
//!     vec![
//!         Token::Word("Hello,"),
//!         Token::Whitespace(1, " "),
//!         Token::Word("world!"),
//!         Token::NewLine
//!     ],
//!     tokens
//! );
//! ```
#[cfg(feature = "ansi")]
use ansi_parser::AnsiSequence;
use core::str::Chars;

/// A text token
#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    /// A newline character.
    NewLine,

    /// A \r character.
    CarriageReturn,

    /// A \t character.
    Tab,

    /// A number of whitespace characters.
    Whitespace(u32, &'a str),

    /// A word (a sequence of non-whitespace characters).
    Word(&'a str),

    /// A possible wrapping point
    Break(&'a str, &'a str),

    /// An ANSI escape sequence
    #[cfg(feature = "ansi")]
    EscapeSequence(AnsiSequence),
}

/// Text parser. Turns a string into a stream of [`Token`] objects.
///
/// [`Token`]: enum.Token.html
#[derive(Clone, Debug)]
pub struct Parser<'a> {
    inner: Chars<'a>,
}

pub(crate) const SPEC_CHAR_NBSP: char = '\u{a0}';
pub(crate) const SPEC_CHAR_ZWSP: char = '\u{200b}';
pub(crate) const SPEC_CHAR_SHY: char = '\u{ad}';
pub(crate) const SPEC_CHAR_ESCAPE: char = '\x1b';

fn is_word_char(c: char) -> bool {
    // Word tokens are terminated when a whitespace, zwsp or shy character is found. An exception
    // to this rule is the nbsp, which is whitespace but is included in the word.
    (!c.is_whitespace() || c == SPEC_CHAR_NBSP)
        && ![SPEC_CHAR_ZWSP, SPEC_CHAR_SHY, SPEC_CHAR_ESCAPE].contains(&c)
}

fn is_space_char(c: char) -> bool {
    // zero-width space breaks whitespace sequences - this works as long as
    // space handling is symmetrical (i.e. starting == ending behaviour)
    c.is_whitespace() && !['\n', '\r', '\t', SPEC_CHAR_NBSP].contains(&c) || c == SPEC_CHAR_ZWSP
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

    /// Returns true if there are no tokens to process.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.as_str().is_empty()
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let string = self.inner.as_str();

        if let Some(c) = self.inner.next() {
            if is_word_char(c) {
                // find the longest consecutive slice of text for a Word token
                for c in &mut self.inner {
                    if !is_word_char(c) {
                        // pointer arithmetic to get the offset of `c` relative to `string`
                        let offset = {
                            let ptr_start = string.as_ptr() as usize;
                            let ptr_cur = self.inner.as_str().as_ptr() as usize;
                            ptr_cur - ptr_start - c.len_utf8()
                        };
                        self.inner = unsafe {
                            // SAFETY: we only work with character boundaries and
                            // offset is <= length
                            string.get_unchecked(offset..).chars()
                        };
                        return Some(Token::Word(unsafe {
                            // SAFETY: we only work with character boundaries and
                            // offset is <= length
                            string.get_unchecked(0..offset)
                        }));
                    }
                }

                // consumed all the text
                Some(Token::Word(string))
            } else {
                match c {
                    // special characters
                    '\n' => Some(Token::NewLine),
                    '\r' => Some(Token::CarriageReturn),
                    '\t' => Some(Token::Tab),
                    SPEC_CHAR_ZWSP => Some(Token::Whitespace(0, unsafe {
                        // SAFETY: we only work with character boundaries and
                        // offset is <= length
                        string.get_unchecked(0..c.len_utf8())
                    })),
                    SPEC_CHAR_SHY => Some(Token::Break(
                        "-", // translate SHY to a printable character
                        unsafe {
                            // SAFETY: we only work with character boundaries and
                            // offset is <= length
                            string.get_unchecked(0..c.len_utf8())
                        },
                    )),
                    #[cfg(feature = "ansi")]
                    SPEC_CHAR_ESCAPE => ansi_parser::parse_escape(string).map_or(
                        Some(Token::EscapeSequence(AnsiSequence::Escape)),
                        |(string, output)| {
                            self.inner = string.chars();
                            Some(Token::EscapeSequence(output))
                        },
                    ),

                    // count consecutive whitespace
                    _ => {
                        let mut len = 1;
                        for c in &mut self.inner {
                            if is_space_char(c) {
                                if c != SPEC_CHAR_ZWSP {
                                    len += 1;
                                }
                            } else {
                                // pointer arithmetic to get the offset of `c` relative to `string`
                                let offset = {
                                    let ptr_start = string.as_ptr() as usize;
                                    let ptr_cur = self.inner.as_str().as_ptr() as usize;
                                    ptr_cur - ptr_start - c.len_utf8()
                                };
                                // consume the whitespaces
                                self.inner = unsafe {
                                    // SAFETY: we only work with character boundaries and
                                    // offset is <= length
                                    string.get_unchecked(offset..).chars()
                                };
                                return Some(Token::Whitespace(len, unsafe {
                                    // SAFETY: we only work with character boundaries and
                                    // offset is <= length
                                    string.get_unchecked(0..offset)
                                }));
                            }
                        }

                        // consumed all the text
                        Some(Token::Whitespace(len, string))
                    }
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Parser, Token};

    #[track_caller]
    pub fn assert_tokens(text: &str, tokens: std::vec::Vec<Token>) {
        assert_eq!(
            Parser::parse(text).collect::<std::vec::Vec<Token>>(),
            tokens
        )
    }

    #[test]
    fn test_parse() {
        assert_tokens(
            "Lorem ipsum \r dolor sit am\u{00AD}et,\tconse😅ctetur adipiscing\nelit",
            vec![
                Token::Word("Lorem"),
                Token::Whitespace(1, " "),
                Token::Word("ipsum"),
                Token::Whitespace(1, " "),
                Token::CarriageReturn,
                Token::Whitespace(1, " "),
                Token::Word("dolor"),
                Token::Whitespace(1, " "),
                Token::Word("sit"),
                Token::Whitespace(1, " "),
                Token::Word("am"),
                Token::Break("-", "\u{ad}"),
                Token::Word("et,"),
                Token::Tab,
                Token::Word("conse😅ctetur"),
                Token::Whitespace(1, " "),
                Token::Word("adipiscing"),
                Token::NewLine,
                Token::Word("elit"),
            ],
        );
    }

    #[test]
    fn parse_zwsp() {
        assert_eq!(9, "two\u{200B}words".chars().count());

        assert_tokens(
            "two\u{200B}words",
            vec![
                Token::Word("two"),
                Token::Whitespace(0, "\u{200B}"),
                Token::Word("words"),
            ],
        );

        // ZWSP is not counted
        assert_tokens("  \u{200B} ", vec![Token::Whitespace(3, "  \u{200B} ")]);
    }

    #[test]
    fn parse_multibyte_last() {
        assert_tokens("test😅", vec![Token::Word("test😅")]);
    }

    #[test]
    fn parse_nbsp_as_word_char() {
        assert_eq!(9, "test\u{A0}word".chars().count());
        assert_tokens("test\u{A0}word", vec![Token::Word("test\u{A0}word")]);
        assert_tokens(
            " \u{A0}word",
            vec![Token::Whitespace(1, " "), Token::Word("\u{A0}word")],
        );
    }

    #[test]
    fn parse_shy_issue_42() {
        assert_tokens(
            "foo\u{AD}bar",
            vec![
                Token::Word("foo"),
                Token::Break("-", "\u{ad}"),
                Token::Word("bar"),
            ],
        );
    }
}

#[cfg(all(feature = "ansi", test))]
mod ansi_parser_tests {

    use super::{test::assert_tokens, Token};
    use ansi_parser::AnsiSequence;
    use heapless::Vec;

    #[test]
    fn escape_char_ignored_if_not_ansi_sequence() {
        assert_tokens(
            "foo\x1bbar",
            vec![
                Token::Word("foo"),
                Token::EscapeSequence(AnsiSequence::Escape),
                Token::Word("bar"),
            ],
        );

        assert_tokens(
            "foo\x1b[bar",
            vec![
                Token::Word("foo"),
                Token::EscapeSequence(AnsiSequence::Escape),
                Token::Word("[bar"),
            ],
        );

        // can escape the escape char
        assert_tokens(
            "foo\x1b\x1bbar",
            vec![
                Token::Word("foo"),
                Token::EscapeSequence(AnsiSequence::Escape),
                Token::Word("bar"),
            ],
        );
    }

    #[test]
    fn escape_char_colors() {
        assert_tokens(
            "foo\x1b[34mbar",
            vec![
                Token::Word("foo"),
                Token::EscapeSequence(AnsiSequence::SetGraphicsMode(
                    Vec::from_slice(&[34]).unwrap(),
                )),
                Token::Word("bar"),
            ],
        );
        assert_tokens(
            "foo\x1b[95mbar",
            vec![
                Token::Word("foo"),
                Token::EscapeSequence(AnsiSequence::SetGraphicsMode(
                    Vec::from_slice(&[95]).unwrap(),
                )),
                Token::Word("bar"),
            ],
        );
        assert_tokens(
            "foo\x1b[48;5;16mbar",
            vec![
                Token::Word("foo"),
                Token::EscapeSequence(AnsiSequence::SetGraphicsMode(
                    Vec::from_slice(&[48, 5, 16]).unwrap(),
                )),
                Token::Word("bar"),
            ],
        );
    }
}
