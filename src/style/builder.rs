//! Textbox style builder.
use crate::{
    alignment::{
        horizontal::{HorizontalTextAlignment, LeftAligned},
        vertical::{top::Top, VerticalTextAlignment},
    },
    style::TextBoxStyle,
};
use embedded_graphics::{
    prelude::*,
    style::{TextStyle, TextStyleBuilder},
};

/// [`TextBoxStyle`] builder object.
///
/// [`TextBoxStyle`]: ../struct.TextBoxStyle.html
pub struct TextBoxStyleBuilder<C, F, A, V>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
{
    text_style_builder: TextStyleBuilder<C, F>,
    alignment: A,
    vertical_alignment: V,
}

impl<C, F> TextBoxStyleBuilder<C, F, LeftAligned, Top>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Creates a new textbox style builder with a given font.
    ///
    /// Default settings are:
    ///  - [`LeftAligned`]
    ///  - Text color: transparent
    ///  - Backgound color: transparent
    #[inline]
    #[must_use]
    pub fn new(font: F) -> Self {
        Self {
            text_style_builder: TextStyleBuilder::new(font),
            alignment: LeftAligned,
            vertical_alignment: Top,
        }
    }
}

impl<C, F, A, V> TextBoxStyleBuilder<C, F, A, V>
where
    C: PixelColor,
    F: Font + Copy,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
{
    /// Sets the text color.
    ///
    /// *Note:* once the text color is set, there is no way to reset it to transparent.
    #[inline]
    #[must_use]
    pub fn text_color(self, text_color: C) -> Self {
        Self {
            text_style_builder: self.text_style_builder.text_color(text_color),
            ..self
        }
    }

    /// Sets the background color.
    ///
    /// *Note:* once the background color is set, there is no way to reset it to transparent.
    ///
    /// # Example: transparent text with background.
    ///
    /// ```rust
    /// # use embedded_text::style::builder::TextBoxStyleBuilder;
    /// # use embedded_graphics::fonts::Font6x8;
    /// # use embedded_graphics::pixelcolor::BinaryColor;
    /// #
    /// let style = TextBoxStyleBuilder::new(Font6x8)
    ///     .background_color(BinaryColor::On)
    ///     .build();
    /// ```
    #[inline]
    #[must_use]
    pub fn background_color(self, background_color: C) -> Self {
        Self {
            text_style_builder: self.text_style_builder.background_color(background_color),
            ..self
        }
    }

    /// Copies properties from an existing text style object.
    #[inline]
    #[must_use]
    pub fn text_style(self, text_style: TextStyle<C, F>) -> Self {
        let mut text_style_builder = self.text_style_builder;

        if let Some(color) = text_style.background_color {
            text_style_builder = text_style_builder.background_color(color);
        }

        if let Some(color) = text_style.text_color {
            text_style_builder = text_style_builder.text_color(color);
        }

        Self {
            text_style_builder,
            ..self
        }
    }

    /// Sets the horizontal text alignment.
    #[inline]
    #[must_use]
    pub fn alignment<TA: HorizontalTextAlignment>(
        self,
        alignment: TA,
    ) -> TextBoxStyleBuilder<C, F, TA, V> {
        TextBoxStyleBuilder {
            text_style_builder: self.text_style_builder,
            alignment,
            vertical_alignment: self.vertical_alignment,
        }
    }

    /// Sets the vertical text alignment.
    #[inline]
    #[must_use]
    pub fn vertical_alignment<VA: VerticalTextAlignment>(
        self,
        vertical_alignment: VA,
    ) -> TextBoxStyleBuilder<C, F, A, VA> {
        TextBoxStyleBuilder {
            text_style_builder: self.text_style_builder,
            alignment: self.alignment,
            vertical_alignment,
        }
    }

    /// Builds the [`TextBoxStyle`].
    ///
    /// [`TextBoxStyle`]: ../struct.TextBoxStyle.html
    #[inline]
    #[must_use]
    pub fn build(self) -> TextBoxStyle<C, F, A, V> {
        TextBoxStyle {
            text_style: self.text_style_builder.build(),
            alignment: self.alignment,
            vertical_alignment: self.vertical_alignment,
        }
    }
}

#[cfg(test)]
mod test {
    use super::TextBoxStyleBuilder;
    use embedded_graphics::{
        fonts::Font6x8,
        pixelcolor::BinaryColor,
        style::{TextStyle, TextStyleBuilder},
    };

    #[test]
    fn test_text_style_copy() {
        let text_styles: [TextStyle<_, _>; 2] = [
            TextStyleBuilder::new(Font6x8)
                .text_color(BinaryColor::On)
                .build(),
            TextStyleBuilder::new(Font6x8)
                .background_color(BinaryColor::On)
                .build(),
        ];

        for &text_style in text_styles.iter() {
            let style = TextBoxStyleBuilder::new(Font6x8)
                .text_style(text_style)
                .build();

            assert_eq!(style.text_style, text_style);
        }
    }
}
