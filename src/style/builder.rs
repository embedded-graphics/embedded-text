//! Text box style builder.
use embedded_graphics::text::LineHeight;

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
/// [`TextBoxStyle`]: struct.TextBoxStyle.html
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
    /// Creates a new text box style builder object.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            style: TextBoxStyle {
                alignment: LeftAligned,
                vertical_alignment: TopAligned,
                height_mode: Exact(FullRowsOnly),
                line_height: LineHeight::Percent(100),
                tab_size: TabSize::Spaces(4),
            },
        }
    }
}

impl<A, V, H> TextBoxStyleBuilder<A, V, H> {
    /// Sets the line height.
    ///
    /// The line height is defined as the vertical distance between the baseline of two adjacent lines
    /// of text.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use embedded_text::style::TextBoxStyleBuilder;
    /// # use embedded_graphics::text::LineHeight;
    /// #
    /// let style = TextBoxStyleBuilder::new()
    ///     .line_height(LineHeight::Pixels(12))
    ///     .build();
    /// ```
    #[inline]
    #[must_use]
    pub fn line_height(mut self, line_height: LineHeight) -> Self {
        self.style.line_height = line_height;

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
                line_height: self.style.line_height,
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
                line_height: self.style.line_height,
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
                line_height: self.style.line_height,
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
    /// [`TextBoxStyle`]: struct.TextBoxStyle.html
    #[inline]
    #[must_use]
    pub fn build(self) -> TextBoxStyle<A, V, H> {
        self.style
    }
}

impl<A, V, H> From<&TextBoxStyle<A, V, H>> for TextBoxStyleBuilder<A, V, H>
where
    A: Copy,
    V: Copy,
    H: Copy,
{
    #[inline]
    fn from(style: &TextBoxStyle<A, V, H>) -> Self {
        Self { style: *style }
    }
}
