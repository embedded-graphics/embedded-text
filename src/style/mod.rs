use embedded_graphics::{prelude::*, style::TextStyle};

pub mod builder;

use crate::{alignment::TextAlignment, rendering::StyledFramedTextIterator, TextBox};
pub use builder::TextBoxStyleBuilder;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct TextBoxStyle<C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    pub text_style: TextStyle<C, F>,
    pub alignment: A,
}

impl<C, F, A> TextBoxStyle<C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    /// Creates a textbox style with transparent background.
    pub fn new(font: F, text_color: C, alignment: A) -> Self {
        Self {
            text_style: TextStyle::new(font, text_color),
            alignment,
        }
    }
}

pub struct StyledTextBox<'a, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    pub text_box: TextBox<'a>,
    pub style: TextBoxStyle<C, F, A>,
}

impl<'a, C, F, A> Drawable<C> for &'a StyledTextBox<'a, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
    StyledFramedTextIterator<'a, C, F, A>: Iterator<Item = Pixel<C>>,
{
    fn draw<D: DrawTarget<C>>(self, display: &mut D) -> Result<(), D::Error> {
        display.draw_iter(A::into_pixel_iterator(self))
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
        // TODO: remove if fixed in embedded-graphics
        let width = (self.bottom_right().x - self.top_left().x) as u32 + 1;
        let height = (self.bottom_right().y - self.top_left().y) as u32 + 1;

        Size::new(width, height)
    }
}
