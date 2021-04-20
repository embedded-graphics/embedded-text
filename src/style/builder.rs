//! Textbox style builder.
use crate::{
    alignment::{HorizontalTextAlignment, LeftAligned, TopAligned, VerticalTextAlignment},
    style::{
        height_mode::{Exact, HeightMode},
        vertical_overdraw::FullRowsOnly,
        TabSize, TextBoxStyle,
    },
};

/// [`TextBoxStyle`] builder object.
///
/// [`TextBoxStyle`]: ../struct.TextBoxStyle.html
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TextBoxStyleBuilder<A, V, H> {
    style: TextBoxStyle<A, V, H>,
}

impl Default for TextBoxStyleBuilder<LeftAligned, TopAligned, Exact<FullRowsOnly>> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TextBoxStyleBuilder<LeftAligned, TopAligned, Exact<FullRowsOnly>> {
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
    pub const fn new() -> Self {
        Self {
            style: TextBoxStyle {
                alignment: LeftAligned,
                vertical_alignment: TopAligned,
                height_mode: Exact(FullRowsOnly),
                line_spacing: 0,
                tab_size: TabSize::Spaces(4),
            },
        }
    }
}

impl<A, V, H> TextBoxStyleBuilder<A, V, H>
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
    /// let style = TextBoxStyleBuilder::new()
    ///     .line_spacing(3)
    ///     .build();
    /// ```
    #[inline]
    #[must_use]
    pub fn line_spacing(mut self, line_spacing: i32) -> Self {
        self.style.line_spacing = line_spacing;

        self
    }

    /// Sets the horizontal text alignment.
    #[inline]
    #[must_use]
    pub fn alignment<TA: HorizontalTextAlignment>(
        self,
        alignment: TA,
    ) -> TextBoxStyleBuilder<TA, V, H> {
        TextBoxStyleBuilder {
            style: TextBoxStyle {
                alignment,
                line_spacing: self.style.line_spacing,
                vertical_alignment: self.style.vertical_alignment,
                height_mode: self.style.height_mode,
                tab_size: self.style.tab_size,
            },
        }
    }

    /// Sets the vertical text alignment.
    #[inline]
    #[must_use]
    pub fn vertical_alignment<VA: VerticalTextAlignment>(
        self,
        vertical_alignment: VA,
    ) -> TextBoxStyleBuilder<A, VA, H> {
        TextBoxStyleBuilder {
            style: TextBoxStyle {
                alignment: self.style.alignment,
                line_spacing: self.style.line_spacing,
                vertical_alignment,
                height_mode: self.style.height_mode,
                tab_size: self.style.tab_size,
            },
        }
    }

    /// Sets the height mode.
    #[inline]
    #[must_use]
    pub fn height_mode<HM: HeightMode>(self, height_mode: HM) -> TextBoxStyleBuilder<A, V, HM> {
        TextBoxStyleBuilder {
            style: TextBoxStyle {
                alignment: self.style.alignment,
                line_spacing: self.style.line_spacing,
                vertical_alignment: self.style.vertical_alignment,
                height_mode,
                tab_size: self.style.tab_size,
            },
        }
    }

    /// Sets the tab size.
    #[inline]
    #[must_use]
    pub fn tab_size(mut self, tab_size: TabSize) -> Self {
        self.style.tab_size = tab_size;

        self
    }
}

impl<A, V, H> TextBoxStyleBuilder<A, V, H>
where
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    /// Builds the [`TextBoxStyle`].
    ///
    /// [`TextBoxStyle`]: ../struct.TextBoxStyle.html
    #[inline]
    #[must_use]
    pub fn build(self) -> TextBoxStyle<A, V, H> {
        self.style
    }
}
