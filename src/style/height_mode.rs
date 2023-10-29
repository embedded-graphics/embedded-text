//! Height adjustment options.
//!
//! This module defines various options to set the height of a [`TextBox`]. Although it is
//! necessary to specify the size of a [`TextBox`], sometimes we may want the text to stretch or
//! shrink the text box. Height modes help us achieve this.
use crate::{
    plugin::PluginMarker as Plugin, rendering::cursor::Cursor, style::VerticalOverdraw, TextBox,
};
use core::ops::Range;
use embedded_graphics::{geometry::Dimensions, text::renderer::TextRenderer};

/// Specifies how the [`TextBox`]'s height should be adjusted.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum HeightMode {
    /// Keep the original [`TextBox`] height.
    ///
    /// # Example: default mode is `Exact(FullRowsOnly)`
    ///
    /// ```rust
    /// # use embedded_graphics::{
    /// #     mono_font::{ascii::FONT_6X9, MonoTextStyle},
    /// #     pixelcolor::BinaryColor,
    /// #     prelude::*,
    /// # };
    /// # let character_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
    /// #
    /// use embedded_graphics::primitives::Rectangle;
    /// use embedded_text::TextBox;
    ///
    /// // The default option is Exact, which does not change the size of the TextBox.
    /// let bounding_box = Rectangle::new(Point::zero(), Size::new(60, 60));
    /// let text_box = TextBox::new(
    ///     "Two lines\nof text",
    ///     bounding_box,
    ///     character_style,
    /// );
    ///
    /// assert_eq!(text_box.bounding_box().size, Size::new(60, 60));
    /// ```
    ///
    /// # Example: display everything inside the bounding box.
    ///
    /// ```rust
    /// # use embedded_graphics::{
    /// #     mono_font::{ascii::FONT_6X9, MonoTextStyle},
    /// #     pixelcolor::BinaryColor,
    /// #     prelude::*,
    /// # };
    /// # let character_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
    /// #
    /// use embedded_graphics::primitives::Rectangle;
    /// use embedded_text::{TextBox, style::{HeightMode, VerticalOverdraw}};
    ///
    /// // `HeightMode::Exact(VerticalOverdraw::Hidden)` will display everything inside the text
    /// // box, not just completely visible rows.
    /// let bounding_box = Rectangle::new(Point::zero(), Size::new(60, 10));
    /// let text_box = TextBox::with_height_mode(
    ///     "Two lines\nof text",
    ///     bounding_box,
    ///     character_style,
    ///     HeightMode::Exact(VerticalOverdraw::Hidden),
    /// );
    ///
    /// assert_eq!(text_box.bounding_box().size, Size::new(60, 10));
    /// ```
    Exact(VerticalOverdraw),

    /// Sets the height of the [`TextBox`] to exactly fit the text.
    ///
    /// Note: in this mode, vertical alignment is meaningless. Make sure to use [`Top`] alignment
    /// for efficiency.
    ///
    /// # Example: `FitToText` shrinks the [`TextBox`].
    ///
    /// ```rust
    /// # use embedded_graphics::{
    /// #     mono_font::{ascii::FONT_6X9, MonoTextStyle},
    /// #     pixelcolor::BinaryColor,
    /// #     prelude::*,
    /// # };
    /// # let character_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
    /// use embedded_graphics::primitives::Rectangle;
    /// use embedded_text::{TextBox, style::HeightMode};
    ///
    /// // FitToText shrinks the TextBox to the height of the text
    /// let bounding_box = Rectangle::new(Point::zero(), Size::new(60, 60));
    /// let text_box = TextBox::with_height_mode(
    ///     "Two lines\nof text",
    ///     bounding_box,
    ///     character_style,
    ///     HeightMode::FitToText,
    /// );
    ///
    /// assert_eq!(text_box.bounding_box().size, Size::new(60, 18));
    /// ```
    ///
    /// # Example: `FitToText` grows the [`TextBox`].
    ///
    /// ```rust
    /// # use embedded_graphics::{
    /// #     mono_font::{ascii::FONT_6X9, MonoTextStyle},
    /// #     pixelcolor::BinaryColor,
    /// #     prelude::*,
    /// # };
    /// # let character_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
    /// use embedded_graphics::primitives::Rectangle;
    /// use embedded_text::{TextBox, style::HeightMode};
    ///
    /// // FitToText grows the TextBox to the height of the text
    /// let bounding_box = Rectangle::new(Point::zero(), Size::new(60, 0));
    /// let text_box = TextBox::with_height_mode(
    ///     "Two lines\nof text",
    ///     bounding_box,
    ///     character_style,
    ///     HeightMode::FitToText,
    /// );
    ///
    /// assert_eq!(text_box.bounding_box().size, Size::new(60, 18));
    /// ```
    ///
    /// [`Top`]: crate::alignment::VerticalAlignment::Top
    FitToText,

    /// If the text does not fill the bounding box, shrink the [`TextBox`] to be as tall as the
    /// text.
    ///
    /// # Example: `ShrinkToText` does not grow the [`TextBox`].
    ///
    /// ```rust
    /// # use embedded_graphics::{
    /// #     mono_font::{ascii::FONT_6X9, MonoTextStyle},
    /// #     pixelcolor::BinaryColor,
    /// #     prelude::*,
    /// # };
    /// # let character_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
    /// #
    /// use embedded_graphics::primitives::Rectangle;
    /// use embedded_text::{TextBox, style::{HeightMode, VerticalOverdraw}};
    ///
    /// // This TextBox contains two lines of text, but is 0px high
    /// let bounding_box = Rectangle::new(Point::zero(), Size::new(60, 0));
    /// let text_box = TextBox::with_height_mode(
    ///     "Two lines\nof text",
    ///     bounding_box,
    ///     character_style,
    ///     HeightMode::ShrinkToText(VerticalOverdraw::FullRowsOnly),
    /// );
    ///
    /// assert_eq!(text_box.bounding_box().size, Size::new(60, 0));
    /// ```
    ///
    /// # Example: `ShrinkToText` shrinks the [`TextBox`].
    ///
    /// ```rust
    /// # use embedded_graphics::{
    /// #     mono_font::{ascii::FONT_6X9, MonoTextStyle},
    /// #     pixelcolor::BinaryColor,
    /// #     prelude::*,
    /// # };
    /// # let character_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
    /// #
    /// use embedded_graphics::primitives::Rectangle;
    /// use embedded_text::{TextBox, style::{HeightMode, VerticalOverdraw}};
    ///
    /// // This TextBox contains two lines of text, but is 60px high
    /// let bounding_box = Rectangle::new(Point::zero(), Size::new(60, 60));
    /// let text_box = TextBox::with_height_mode(
    ///     "Two lines\nof text",
    ///     bounding_box,
    ///     character_style,
    ///     HeightMode::ShrinkToText(VerticalOverdraw::Hidden),
    /// );
    ///
    /// assert_eq!(text_box.bounding_box().size, Size::new(60, 18));
    /// ```
    ShrinkToText(VerticalOverdraw),
}

impl HeightMode {
    /// Apply the height mode to the text box.
    ///
    /// *Note:* This function normally does not need to be called manually.
    pub(crate) fn apply<'a, F, M>(self, text_box: &mut TextBox<'a, F, M>)
    where
        F: TextRenderer,
        M: Plugin<'a, F::Color>,
    {
        match self {
            HeightMode::Exact(_) => {}
            HeightMode::FitToText => {
                text_box.fit_height();
            }
            HeightMode::ShrinkToText(_) => {
                text_box.fit_height_limited(text_box.bounding_box().size.height);
            }
        }
    }

    /// Calculate the range of rows of the current line that can be drawn.
    ///
    /// If a line does not fully fit in the bounding box, some `HeightMode` options allow drawing
    /// partial lines. For a partial line, this function calculates, which rows of each character
    /// should be displayed.
    pub(crate) fn calculate_displayed_row_range(self, cursor: &Cursor) -> Range<u32> {
        let overdraw = match self {
            HeightMode::Exact(overdraw) | HeightMode::ShrinkToText(overdraw) => overdraw,
            HeightMode::FitToText => VerticalOverdraw::Visible,
        };

        overdraw.calculate_displayed_row_range(cursor)
    }
}
