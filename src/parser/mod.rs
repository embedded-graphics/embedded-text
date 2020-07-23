//! Parse text into words, newlines and whitespace sequences
use core::str::CharIndices;

/// A text token
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'a> {
    /// A newline character
    NewLine,

    /// n whitespace characters
    Whitespace(u32),

    /// A word (a sequence of non-whitespace characters)
    Word(&'a str),
}

/// The parser struct
#[derive(Clone)]
pub struct Parser<'a> {
    inner: CharIndices<'a>,
}

impl<'a> Parser<'a> {
    /// Create a new parser object to process the given piece of text
    #[inline]
    #[must_use]
    pub fn parse(text: &'a str) -> Self {
        Self {
            inner: text.char_indices(),
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let string = self.inner.as_str();
        self.inner.next().map(|(_, c)| match c {
            '\n' => Token::NewLine,

            c if c.is_whitespace() => {
                let mut len = 0;
                for (idx, c) in string.char_indices() {
                    if c.is_whitespace() {
                        len += 1;
                    } else {
                        // consume the whitespaces
                        self.inner = unsafe { string.get_unchecked(idx..) }.char_indices();
                        return Token::Whitespace(len);
                    }
                }

                // consume all the text
                self.inner = "".char_indices();
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
                        self.inner = rest.char_indices();
                        return Token::Word(word);
                    }
                }

                // consume all the text
                self.inner = "".char_indices();
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
