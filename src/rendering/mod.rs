//! Pixel iterators used for text rendering.

pub mod ansi;
pub mod character;
pub mod cursor;
pub mod line;
pub mod line_iter;
pub mod modified_whitespace;
pub mod space_config;
pub mod whitespace;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{cursor::Cursor, line::StyledLinePixelIterator, space_config::SpaceConfig},
    style::{color::Rgb, height_mode::HeightMode, TextBoxStyle},
    StyledTextBox,
};
use embedded_graphics::prelude::*;

/// State variable used by the right aligned text renderer.
#[derive(Debug)]
pub enum State<'a, C, F, SP, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Starts processing a line.
    NextLine(Option<Token<'a>>, Cursor<F>, Parser<'a>),

    /// Renders the processed line.
    DrawLine(StyledLinePixelIterator<'a, C, F, SP, A, V, H>),
}

/// This trait is used to associate a renderer type to a horizontal alignment option.
///
/// Implementing this trait is only necessary when creating new alignment algorithms.
pub trait RendererFactory<'a, C: PixelColor> {
    /// The type of the pixel iterator.
    type Renderer: Iterator<Item = Pixel<C>>;

    /// Creates a new renderer object.
    fn create_renderer(&self) -> Self::Renderer;
}

type LineIteratorSource<'a, C, F, A, V, H, SP> =
    fn(
        TextBoxStyle<C, F, A, V, H>,
        Option<Token<'a>>,
        Cursor<F>,
        Parser<'a>,
    ) -> StyledLinePixelIterator<'a, C, F, SP, A, V, H>;

/// Pixel iterator for styled text.
pub struct StyledTextBoxIterator<'a, C, F, A, V, H, SP>
where
    C: PixelColor,
    F: Font + Copy,
{
    style: TextBoxStyle<C, F, A, V, H>,
    state: State<'a, C, F, SP, A, V, H>,
    next_line_fn: LineIteratorSource<'a, C, F, A, V, H, SP>,
}

impl<'a, C, F, A, V, H, SP> StyledTextBoxIterator<'a, C, F, A, V, H, SP>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
    SP: SpaceConfig<Font = F>,
{
    /// Creates a new pixel iterator to render the styled [`TextBox`].
    ///
    /// [`TextBox`]: ../struct.TextBox.html
    #[inline]
    #[must_use]
    pub fn new(
        styled: &StyledTextBox<'a, C, F, A, V, H>,
        f: LineIteratorSource<'a, C, F, A, V, H, SP>,
    ) -> Self {
        let mut cursor = Cursor::new(styled.text_box.bounds, styled.style.line_spacing);

        V::apply_vertical_alignment(&mut cursor, &styled);

        Self {
            style: styled.style,
            state: State::NextLine(None, cursor, Parser::parse(styled.text_box.text)),
            next_line_fn: f,
        }
    }
}

impl<'a, C, F, A, V, H, SP> Iterator for StyledTextBoxIterator<'a, C, F, A, V, H, SP>
where
    C: PixelColor + From<Rgb>,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
    SP: SpaceConfig<Font = F>,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                State::NextLine(ref carried_token, ref cursor, ref parser) => {
                    if carried_token.is_none() && parser.is_empty() {
                        break None;
                    }

                    let f = self.next_line_fn;
                    self.state = State::DrawLine(f(
                        self.style,
                        carried_token.clone(),
                        *cursor,
                        parser.clone(),
                    ));
                }

                State::DrawLine(ref mut line_iterator) => {
                    if let pixel @ Some(_) = line_iterator.next() {
                        break pixel;
                    }

                    self.style = line_iterator.style;
                    self.state = State::NextLine(
                        line_iterator.remaining_token(),
                        line_iterator.cursor(),
                        line_iterator.parser(),
                    );
                }
            }
        }
    }
}
