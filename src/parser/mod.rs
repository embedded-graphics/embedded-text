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
        self.inner.next().map(|(start, c)| match c {
            '\n' => Token::NewLine,

            c if c.is_whitespace() => {
                let mut len = 1;
                for (_, c) in self.inner.clone() {
                    if c.is_whitespace() {
                        // consume the whitespace
                        self.inner.next();
                        len += 1;
                    } else {
                        break;
                    }
                }
                Token::Whitespace(len)
            }

            _ => {
                for (possible_end, c) in self.inner.clone() {
                    if c.is_whitespace() {
                        return Token::Word(unsafe {
                            // don't worry
                            string.get_unchecked(0..possible_end - start)
                        });
                    } else {
                        // consume the character
                        self.inner.next();
                    }
                }
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
