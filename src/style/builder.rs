use crate::{alignment::LeftAligned, style::TextBoxStyle, TextAlignment};
use embedded_graphics::{prelude::*, style::TextStyleBuilder};

pub struct TextBoxStyleBuilder<C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    text_style_builder: TextStyleBuilder<C, F>,
    alignment: A,
}

impl<C, F> TextBoxStyleBuilder<C, F, LeftAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Creates a new text style builder with a given font.
    pub fn new(font: F) -> Self {
        Self {
            text_style_builder: TextStyleBuilder::new(font),
            alignment: LeftAligned,
        }
    }
}

impl<C, F, A> TextBoxStyleBuilder<C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
{
    /// Sets the text color.
    pub fn text_color(self, text_color: C) -> Self {
        Self {
            text_style_builder: self.text_style_builder.text_color(text_color),
            ..self
        }
    }

    /// Sets the background color.
    pub fn background_color(self, background_color: C) -> Self {
        Self {
            text_style_builder: self.text_style_builder.background_color(background_color),
            ..self
        }
    }

    /// Sets the text alignment.
    pub fn align<AA: TextAlignment>(self, alignment: AA) -> TextBoxStyleBuilder<C, F, AA> {
        TextBoxStyleBuilder {
            text_style_builder: self.text_style_builder,
            alignment,
        }
    }

    /// Builds the text style.
    pub fn build(self) -> TextBoxStyle<C, F, A> {
        TextBoxStyle {
            text_style: self.text_style_builder.build(),
            alignment: self.alignment,
        }
    }
}
