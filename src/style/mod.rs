use embedded_graphics::{prelude::*, style::TextStyle};

/// Textbox style builder
pub mod builder;

use crate::{
    alignment::TextAlignment,
    rendering::{StateFactory, StyledTextBoxIterator},
    utils::rect_ext::RectExt,
    TextBox,
};
pub use builder::TextBoxStyleBuilder;

/// Styling options of a [`TextBox`].
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct TextBoxStyle<C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    /// Style properties for text.
    pub text_style: TextStyle<C, F>,

    /// Horizontal alignment
    pub alignment: A,
}

impl<C, F, A> TextBoxStyle<C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    /// Creates a textbox style with transparent background.
    #[inline]
    pub fn new(font: F, text_color: C, alignment: A) -> Self {
        Self {
            text_style: TextStyle::new(font, text_color),
            alignment,
        }
    }
}

/// A styled [`TextBox`] struct.
pub struct StyledTextBox<'a, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    /// A [`TextBox`] that has an associated [`TextBoxStyle`]
    pub text_box: TextBox<'a>,

    /// The style of the [`TextBox`]
    pub style: TextBoxStyle<C, F, A>,
}

impl<'a, C, F, A> Drawable<C> for &'a StyledTextBox<'a, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
    StyledTextBoxIterator<'a, C, F, A>: Iterator<Item = Pixel<C>>,
    StyledTextBox<'a, C, F, A>: StateFactory,
{
    #[inline]
    fn draw<D: DrawTarget<C>>(self, display: &mut D) -> Result<(), D::Error> {
        display.draw_iter(StyledTextBoxIterator::new(self))
    }
}

impl<C, F, A> Transform for StyledTextBox<'_, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    #[inline]
    #[must_use]
    fn translate(&self, by: Point) -> Self {
        Self {
            text_box: self.text_box.translate(by),
            style: self.style,
        }
    }

    #[inline]
    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.text_box.bounds.translate_mut(by);

        self
    }
}

impl<C, F, A> Dimensions for StyledTextBox<'_, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    #[inline]
    #[must_use]
    fn top_left(&self) -> Point {
        self.text_box.bounds.top_left
    }

    #[inline]
    #[must_use]
    fn bottom_right(&self) -> Point {
        self.text_box.bounds.bottom_right
    }

    #[inline]
    #[must_use]
    fn size(&self) -> Size {
        RectExt::size(self.text_box.bounds)
    }
}
