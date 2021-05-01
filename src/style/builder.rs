//! Text box style builder.
use embedded_graphics::text::LineHeight;

use crate::{
    alignment::{HorizontalTextAlignment, LeftAligned, TopAligned, VerticalTextAlignment},
    style::{height_mode::HeightMode, vertical_overdraw::VerticalOverdraw, TabSize, TextBoxStyle},
};

/// [`TextBoxStyle`] builder object.
///
/// [`TextBoxStyle`]: struct.TextBoxStyle.html
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TextBoxStyleBuilder<A, V> {
    style: TextBoxStyle<A, V>,
}

impl Default for TextBoxStyleBuilder<LeftAligned, TopAligned> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TextBoxStyleBuilder<LeftAligned, TopAligned> {
    /// Creates a new text box style builder object.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            style: TextBoxStyle {
                alignment: LeftAligned,
                vertical_alignment: TopAligned,
                height_mode: HeightMode::Exact(VerticalOverdraw::FullRowsOnly),
                line_height: LineHeight::Percent(100),
                tab_size: TabSize::Spaces(4),
            },
        }
    }
}

impl<A, V> TextBoxStyleBuilder<A, V> {
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
    ) -> TextBoxStyleBuilder<TA, V> {
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
    ) -> TextBoxStyleBuilder<A, VA> {
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
    pub fn height_mode(mut self, height_mode: HeightMode) -> TextBoxStyleBuilder<A, V> {
        self.style.height_mode = height_mode;
        self
    }

    /// Sets the tab size.
    #[inline]
    #[must_use]
    pub fn tab_size(mut self, tab_size: TabSize) -> Self {
        self.style.tab_size = tab_size;

        self
    }
}

impl<A, V> TextBoxStyleBuilder<A, V>
where
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
{
    /// Builds the [`TextBoxStyle`].
    ///
    /// [`TextBoxStyle`]: struct.TextBoxStyle.html
    #[inline]
    #[must_use]
    pub fn build(self) -> TextBoxStyle<A, V> {
        self.style
    }
}

impl<A, V> From<&TextBoxStyle<A, V>> for TextBoxStyleBuilder<A, V>
where
    A: Copy,
    V: Copy,
{
    #[inline]
    fn from(style: &TextBoxStyle<A, V>) -> Self {
        Self { style: *style }
    }
}
