//! Pixel iterators used for text rendering.

pub mod character;
pub mod cursor;
pub mod line;
pub mod line_iter;
pub mod space_config;
pub mod whitespace;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::Parser,
    rendering::cursor::Cursor,
    style::{height_mode::HeightMode, TextBoxStyle},
    StyledTextBox,
};
use embedded_graphics::prelude::*;

/// This trait is used to associate a state type to a horizontal alignment option.
///
/// Implementing this trait is only necessary when creating new alignment algorithms.
pub trait StateFactory<'a, F: Font> {
    /// The type of the state variable used for rendering.
    type PixelIteratorState;

    /// Creates a new state variable.
    fn create_state(&self, cursor: Cursor<F>, parser: Parser<'a>) -> Self::PixelIteratorState;
}

/// Pixel iterator for styled text.
pub struct StyledTextBoxIterator<'a, C, F, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
    StyledTextBox<'a, C, F, A, V, H>: StateFactory<'a, F>,
{
    /// Style used for rendering.
    pub style: TextBoxStyle<C, F, A, V, H>,

    /// State information used by the rendering algorithms.
    pub state: <StyledTextBox<'a, C, F, A, V, H> as StateFactory<'a, F>>::PixelIteratorState,
}

impl<'a, C, F, A, V, H> StyledTextBoxIterator<'a, C, F, A, V, H>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
    StyledTextBox<'a, C, F, A, V, H>: StateFactory<'a, F>,
{
    /// Creates a new pixel iterator to render the styled [`TextBox`].
    ///
    /// [`TextBox`]: ../struct.TextBox.html
    #[inline]
    #[must_use]
    pub fn new(styled: &'a StyledTextBox<'a, C, F, A, V, H>) -> Self {
        let mut cursor = Cursor::new(styled.text_box.bounds);

        V::apply_vertical_alignment(&mut cursor, &styled);

        Self {
            style: styled.style,
            state: styled.create_state(cursor, Parser::parse(styled.text_box.text)),
        }
    }
}
