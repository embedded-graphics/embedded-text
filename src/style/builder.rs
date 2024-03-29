//! Text box style builder.
use embedded_graphics::text::LineHeight;

use crate::{
    alignment::{HorizontalAlignment, VerticalAlignment},
    style::{HeightMode, TabSize, TextBoxStyle, VerticalOverdraw},
};

/// [`TextBoxStyle`] builder object.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[must_use]
pub struct TextBoxStyleBuilder {
    style: TextBoxStyle,
    leading_spaces: Option<bool>,
    trailing_spaces: Option<bool>,
}

impl TextBoxStyleBuilder {
    /// Create a new builder object.
    #[inline]
    pub const fn default() -> Self {
        Self::new()
    }

    /// Creates a new text box style builder object.
    #[inline]
    pub const fn new() -> Self {
        Self {
            style: TextBoxStyle {
                alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Top,
                height_mode: HeightMode::Exact(VerticalOverdraw::FullRowsOnly),
                line_height: LineHeight::Percent(100),
                paragraph_spacing: 0,
                tab_size: TabSize::Spaces(4),
                // we will update these at build time
                leading_spaces: false,
                trailing_spaces: false,
            },
            leading_spaces: None,
            trailing_spaces: None,
        }
    }

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
    pub const fn line_height(mut self, line_height: LineHeight) -> Self {
        self.style.line_height = line_height;

        self
    }

    /// Sets the paragraph spacing.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use embedded_text::style::TextBoxStyleBuilder;
    /// # use embedded_graphics::text::LineHeight;
    /// #
    /// let style = TextBoxStyleBuilder::new()
    ///     .paragraph_spacing(0)
    ///     .build();
    /// ```
    #[inline]
    pub const fn paragraph_spacing(mut self, paragraph_spacing: u32) -> Self {
        self.style.paragraph_spacing = paragraph_spacing;

        self
    }

    /// Sets the horizontal text alignment.
    #[inline]
    pub const fn alignment(mut self, alignment: HorizontalAlignment) -> TextBoxStyleBuilder {
        self.style.alignment = alignment;

        self
    }

    /// Sets the vertical text alignment.
    #[inline]
    pub const fn vertical_alignment(
        mut self,
        vertical_alignment: VerticalAlignment,
    ) -> TextBoxStyleBuilder {
        self.style.vertical_alignment = vertical_alignment;

        self
    }

    /// Sets the height mode.
    #[inline]
    pub const fn height_mode(mut self, height_mode: HeightMode) -> TextBoxStyleBuilder {
        self.style.height_mode = height_mode;

        self
    }

    /// Sets the tab size.
    #[inline]
    pub const fn tab_size(mut self, tab_size: TabSize) -> Self {
        self.style.tab_size = tab_size;

        self
    }

    /// Render leading spaces.
    #[inline]
    pub const fn leading_spaces(mut self, render: bool) -> Self {
        self.leading_spaces = Some(render);

        self
    }

    /// Render trailing spaces.
    #[inline]
    pub const fn trailing_spaces(mut self, render: bool) -> Self {
        self.trailing_spaces = Some(render);

        self
    }

    /// Builds the [`TextBoxStyle`].
    #[inline]
    pub const fn build(mut self) -> TextBoxStyle {
        self.style.leading_spaces = match self.leading_spaces {
            Some(leading_spaces) => leading_spaces,
            None => self.style.alignment.leading_spaces(),
        };

        self.style.trailing_spaces = match self.trailing_spaces {
            Some(trailing_spaces) => trailing_spaces,
            None => self.style.alignment.trailing_spaces(),
        };

        self.style
    }
}

impl From<&TextBoxStyle> for TextBoxStyleBuilder {
    #[inline]
    fn from(style: &TextBoxStyle) -> Self {
        Self {
            style: *style,
            leading_spaces: None,
            trailing_spaces: None,
        }
    }
}
