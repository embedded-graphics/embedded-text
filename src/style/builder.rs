//! Textbox style builder.
use crate::{
    alignment::{HorizontalTextAlignment, LeftAligned, TopAligned, VerticalTextAlignment},
    style::{
        height_mode::{Exact, HeightMode},
        vertical_overdraw::FullRowsOnly,
        TabSize, TextBoxStyle, UndefinedCharacterStyle,
    },
};
use embedded_graphics::text::renderer::TextRenderer;

/// [`TextBoxStyle`] builder object.
///
/// [`TextBoxStyle`]: ../struct.TextBoxStyle.html
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TextBoxStyleBuilder<F, A, V, H> {
    text_box_style: TextBoxStyle<F, A, V, H>,
}

impl Default
    for TextBoxStyleBuilder<UndefinedCharacterStyle, LeftAligned, TopAligned, Exact<FullRowsOnly>>
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TextBoxStyleBuilder<UndefinedCharacterStyle, LeftAligned, TopAligned, Exact<FullRowsOnly>> {
    /// Creates a new `TextBoxStyleBuilder` with a given MonoFont.
    ///
    /// Default settings are:
    ///  - [`LeftAligned`]
    ///  - [`TopAligned`]
    ///  - Text color: transparent
    ///  - Background color: transparent
    ///  - Height mode: [`Exact`]
    ///  - Line spacing: 0px
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            text_box_style: TextBoxStyle {
                character_style: UndefinedCharacterStyle,
                alignment: LeftAligned,
                vertical_alignment: TopAligned,
                height_mode: Exact(FullRowsOnly),
                line_spacing: 0,
                tab_size: TabSize::default(),
            },
        }
    }
}

impl<F, A, V, H> TextBoxStyleBuilder<F, A, V, H>
where
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    /// Sets the vertical space between lines, in pixels.
    ///
    /// *Note:* You can set negative values as line spacing if you wish your lines to overlap.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use embedded_text::prelude::*;
    /// # use embedded_graphics::{
    /// #     mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    /// #     pixelcolor::BinaryColor,
    /// #     prelude::*,
    /// # };
    /// #
    /// # let character_style = MonoTextStyleBuilder::new()
    /// #     .font(&FONT_6X9)
    /// #     .text_color(BinaryColor::On)
    /// #     .build();
    /// let style = TextBoxStyleBuilder::new()
    ///     .character_style(character_style)
    ///     .line_spacing(3)
    ///     .build();
    /// ```
    #[inline]
    #[must_use]
    pub fn line_spacing(self, line_spacing: i32) -> Self {
        Self {
            text_box_style: TextBoxStyle {
                line_spacing,
                ..self.text_box_style
            },
        }
    }

    /// Sets the character style.
    #[inline]
    #[must_use]
    pub fn character_style<CS: TextRenderer>(
        self,
        character_style: CS,
    ) -> TextBoxStyleBuilder<CS, A, V, H> {
        TextBoxStyleBuilder {
            text_box_style: TextBoxStyle {
                character_style,
                alignment: self.text_box_style.alignment,
                line_spacing: self.text_box_style.line_spacing,
                vertical_alignment: self.text_box_style.vertical_alignment,
                height_mode: self.text_box_style.height_mode,
                tab_size: self.text_box_style.tab_size,
            },
        }
    }

    /// Sets the horizontal text alignment.
    #[inline]
    #[must_use]
    pub fn alignment<TA: HorizontalTextAlignment>(
        self,
        alignment: TA,
    ) -> TextBoxStyleBuilder<F, TA, V, H> {
        TextBoxStyleBuilder {
            text_box_style: TextBoxStyle {
                character_style: self.text_box_style.character_style,
                alignment,
                line_spacing: self.text_box_style.line_spacing,
                vertical_alignment: self.text_box_style.vertical_alignment,
                height_mode: self.text_box_style.height_mode,
                tab_size: self.text_box_style.tab_size,
            },
        }
    }

    /// Sets the vertical text alignment.
    #[inline]
    #[must_use]
    pub fn vertical_alignment<VA: VerticalTextAlignment>(
        self,
        vertical_alignment: VA,
    ) -> TextBoxStyleBuilder<F, A, VA, H> {
        TextBoxStyleBuilder {
            text_box_style: TextBoxStyle {
                character_style: self.text_box_style.character_style,
                alignment: self.text_box_style.alignment,
                line_spacing: self.text_box_style.line_spacing,
                vertical_alignment,
                height_mode: self.text_box_style.height_mode,
                tab_size: self.text_box_style.tab_size,
            },
        }
    }

    /// Sets the height mode.
    #[inline]
    #[must_use]
    pub fn height_mode<HM: HeightMode>(self, height_mode: HM) -> TextBoxStyleBuilder<F, A, V, HM> {
        TextBoxStyleBuilder {
            text_box_style: TextBoxStyle {
                character_style: self.text_box_style.character_style,
                alignment: self.text_box_style.alignment,
                line_spacing: self.text_box_style.line_spacing,
                vertical_alignment: self.text_box_style.vertical_alignment,
                height_mode,
                tab_size: self.text_box_style.tab_size,
            },
        }
    }

    /// Sets the tab size.
    #[inline]
    #[must_use]
    pub fn tab_size(self, tab_size: TabSize) -> Self {
        Self {
            text_box_style: TextBoxStyle {
                tab_size,
                ..self.text_box_style
            },
        }
    }
}

impl<F, A, V, H> TextBoxStyleBuilder<F, A, V, H>
where
    F: TextRenderer,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    /// Builds the [`TextBoxStyle`].
    ///
    /// [`TextBoxStyle`]: ../struct.TextBoxStyle.html
    #[inline]
    #[must_use]
    pub fn build(self) -> TextBoxStyle<F, A, V, H> {
        self.text_box_style
    }
}
