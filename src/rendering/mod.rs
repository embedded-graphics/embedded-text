//! Pixel iterators used for text rendering.

pub mod character;
pub mod cursor;
pub mod line;
pub mod whitespace;

use crate::{
    alignment::{horizontal::HorizontalTextAlignment, vertical::VerticalTextAlignment},
    parser::Parser,
    rendering::cursor::Cursor,
    style::{StyledTextBox, TextBoxStyle},
};
use embedded_graphics::prelude::*;

/// This trait is used to associate a state type to a horizontal alignment option.
///
/// Implementing this trait is only necessary when creating new alignment algorithms.
pub trait StateFactory<F: Font> {
    /// The type of the state variable used for rendering.
    type PixelIteratorState;

    /// Creates a new state variable.
    fn create_state(&self, cursor: Cursor<F>) -> Self::PixelIteratorState;
}

/// Pixel iterator for styled text.
pub struct StyledTextBoxIterator<'a, C, F, A, V>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    StyledTextBox<'a, C, F, A, V>: StateFactory<F>,
{
    /// Parser to process the text during rendering.
    pub parser: Parser<'a>,

    /// Style used for rendering.
    pub style: TextBoxStyle<C, F, A, V>,

    /// State information used by the rendering algorithms.
    pub state: <StyledTextBox<'a, C, F, A, V> as StateFactory<F>>::PixelIteratorState,
}

impl<'a, C, F, A, V> StyledTextBoxIterator<'a, C, F, A, V>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    StyledTextBox<'a, C, F, A, V>: StateFactory<F>,
{
    /// Creates a new pixel iterator to render the styled [`TextBox`].
    ///
    /// [`TextBox`]: ../struct.TextBox.html
    #[inline]
    #[must_use]
    pub fn new(styled: &'a StyledTextBox<'a, C, F, A, V>) -> Self {
        let mut cursor = Cursor::new(styled.text_box.bounds);

        V::apply_vertical_alignment(&mut cursor, &styled);

        Self {
            parser: Parser::parse(styled.text_box.text),
            style: styled.style,
            state: styled.create_state(cursor),
        }
    }
}
