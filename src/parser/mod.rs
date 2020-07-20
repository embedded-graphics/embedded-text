//! Parse text into words, newlines and whitespace sequences
use core::str::CharIndices;

/// A text token
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'a> {
    NewLine,
    Whitespace(u32),
    Word(&'a str),
}

/// The parser struct
#[derive(Clone)]
pub struct Parser<'a> {
    text: &'a str,
    inner: CharIndices<'a>,
}

impl<'a> Parser<'a> {
    /// Create a new parser object to process the given piece of text
    #[inline]
    #[must_use]
    pub fn parse(text: &'a str) -> Self {
        Self {
            text,
            inner: text.char_indices(),
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(idx, c)| match c {
            '\n' => Token::NewLine,
            c if c.is_whitespace() => {
                let mut n = 1;
                for (_, c) in self.inner.clone() {
                    if c.is_whitespace() {
                        self.inner.next();
                        n += 1;
                    } else {
                        break;
                    }
                }
                Token::Whitespace(n)
            }
            _ => {
                let mut end = idx;
                for (idx, c) in self.inner.clone() {
                    if c.is_whitespace() {
                        break;
                    } else {
                        end = idx;
                        self.inner.next();
                    }
                }

                Token::Word(&self.text[idx..=end])
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
}
