//! This benchmark hopes to show that compared to the original implementation, the current one
//! either has better performance, or at least is not a big regression.
//!
//! This is relevant, because the new implementation generates less machine code:
//! https://godbolt.org/z/nWsTnx
use core::str::CharIndices;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use embedded_text::parser::{Parser, Token};

pub struct OriginalParser<'a> {
    text: &'a str,
    inner: CharIndices<'a>,
}

impl<'a> OriginalParser<'a> {
    #[inline]
    #[must_use]
    pub fn parse(text: &'a str) -> Self {
        Self {
            text,
            inner: text.char_indices(),
        }
    }
}

impl<'a> Iterator for OriginalParser<'a> {
    type Item = Token<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((idx, c)) = self.inner.next() {
            match c {
                '\n' => Some(Token::NewLine),
                c if c.is_whitespace() => {
                    let mut n = 1;
                    for (_, c) in self.inner.clone() {
                        if !c.is_whitespace() {
                            break;
                        }

                        n += 1;
                        self.inner.next();
                    }
                    Some(Token::Whitespace(n))
                }
                _ => {
                    let mut end = idx;
                    for (idx, c) in self.inner.clone() {
                        if c.is_whitespace() {
                            break;
                        }

                        end = idx;
                        self.inner.next();
                    }

                    Some(Token::Word(&self.text[idx..=end]))
                }
            }
        } else {
            None
        }
    }
}

const TEXT: &str = "Hello, World!\nLorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.";

fn benchmark_original(c: &mut Criterion) {
    c.bench_function("Original parser", |b| {
        b.iter(|| OriginalParser::parse(black_box(TEXT)).collect::<Vec<Token<'_>>>())
    });
}

fn benchmark_current(c: &mut Criterion) {
    c.bench_function("Current parser", |b| {
        b.iter(|| Parser::parse(black_box(TEXT)).collect::<Vec<Token<'_>>>())
    });
}

criterion_group!(parse, benchmark_original, benchmark_current);
criterion_main!(parse);
