//! Height adjustment options.
//!
//! This module defines various options to set the height of a [`TextBox`]. Although it is
//! necessary to specify the size of a [`TextBox`], sometimes we may want the text to stretch or
//! shrink the text box. Height modes help us achieve this.
//!
//! [`TextBox`]: ../../struct.TextBox.html
use crate::{rendering::cursor::Cursor, style::VerticalOverdraw, TextBox};
use core::ops::Range;
use embedded_graphics::{geometry::Dimensions, text::renderer::TextRenderer};

/// Specifies how the [`TextBox`]'s height should be adjusted.
///
/// [`TextBox`]: ../struct.TextBox.html
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum HeightMode {
    /// Keep the original [`TextBox`] height.
    ///
    /// # Example:
    ///
    /// ```rust
    /// use embedded_graphics::{
    ///     mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    ///     pixelcolor::BinaryColor,
    ///     prelude::*,
    ///     primitives::Rectangle,
    /// };
    /// use embedded_text::{style::TextBoxStyleBuilder, TextBox};
    ///
    /// let character_style = MonoTextStyleBuilder::new()
    ///     .font(&FONT_6X9)
    ///     .text_color(BinaryColor::On)
    ///     .build();
    ///
    /// // This TextBox contains two lines of text, but is 60px high
    /// let text_box = TextBox::new(
    ///     "Two lines\nof text",
    ///     Rectangle::new(Point::zero(), Size::new(60, 60)),
    ///     character_style,
    /// );
    ///
    /// // Exact does not change the size of the TextBox
    /// let orig_size = text_box.bounding_box().size;
    /// let size = text_box.bounding_box().size;
    /// assert_eq!(size, orig_size);
    /// ```
    Exact(VerticalOverdraw),

    /// Sets the height of the [`TextBox`] to exactly fit the text.
    ///
    /// Note: in this mode, vertical alignment is meaningless. Make sure to use [`Top`] alignment
    /// for efficiency.
    ///
    /// # Example: `FitToText` grows the [`TextBox`].
    ///
    /// ```rust
    /// use embedded_graphics::{
    ///     mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    ///     pixelcolor::BinaryColor,
    ///     prelude::*,
    ///     primitives::Rectangle,
    /// };
    /// use embedded_text::{
    ///     style::{HeightMode, TextBoxStyleBuilder},
    ///     TextBox,
    /// };
    ///
    /// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
    /// let character_style = MonoTextStyleBuilder::new()
    ///     .font(&FONT_6X9)
    ///     .text_color(BinaryColor::On)
    ///     .build();
    ///
    /// let style = TextBoxStyleBuilder::new().height_mode(HeightMode::FitToText).build();
    ///
    /// // This TextBox contains two lines of text, but is 1px high
    /// let text_box = TextBox::with_textbox_style(
    ///     "Two lines\nof text",
    ///     Rectangle::new(Point::zero(), Size::new(60, 0)),
    ///     character_style,
    ///     style,
    /// );
    ///
    /// // FitToText grows the TextBox to the height of the text
    /// let size = text_box.bounding_box().size;
    /// assert_eq!(size, Size::new(60, 18));
    /// ```
    ///
    /// # Example: `FitToText` also shrinks the [`TextBox`].
    ///
    /// ```rust
    /// use embedded_graphics::{
    ///     mono_font::{ascii::FONT_6X9, MonoTextStyle},
    ///     pixelcolor::BinaryColor,
    ///     prelude::*,
    ///     primitives::Rectangle,
    /// };
    /// use embedded_text::{
    ///     style::{HeightMode, TextBoxStyleBuilder},
    ///     TextBox,
    /// };
    ///
    /// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
    /// let character_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
    ///
    /// let style = TextBoxStyleBuilder::new().height_mode(HeightMode::FitToText).build();
    ///
    /// // This TextBox contains two lines of text, but is 1px high
    /// let text_box = TextBox::with_textbox_style(
    ///     "Two lines\nof text",
    ///     Rectangle::new(Point::zero(), Size::new(60, 60)),
    ///     character_style,
    ///     style,
    /// );
    ///
    /// // FitToText shrinks the TextBox to the height of the text
    /// let size = text_box.bounding_box().size;
    /// assert_eq!(size, Size::new(60, 18));
    /// ```
    ///
    /// [`TopAligned`]: ../alignment/enum.VerticalAlignment.html#variant.Top
    FitToText,

    /// If the text does not fill the bounding box, shrink the [`TextBox`] to be as tall as the
    /// text.
    ///
    /// # Example: `ShrinkToText` does not grow the [`TextBox`].
    ///
    /// ```rust
    /// use embedded_graphics::{
    ///     mono_font::{ascii::FONT_6X9, MonoTextStyle},
    ///     pixelcolor::BinaryColor,
    ///     prelude::*,
    ///     primitives::Rectangle,
    /// };
    /// use embedded_text::{
    ///     style::{HeightMode, VerticalOverdraw, TextBoxStyleBuilder},
    ///     TextBox,
    /// };
    ///
    /// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
    /// let character_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
    ///
    /// let style = TextBoxStyleBuilder::new()
    ///     .height_mode(HeightMode::ShrinkToText(VerticalOverdraw::FullRowsOnly))
    ///     .build();
    ///
    /// // This TextBox contains two lines of text, but is 1px high
    /// let text_box = TextBox::with_textbox_style(
    ///     "Two lines\nof text",
    ///     Rectangle::new(Point::zero(), Size::new(60, 0)),
    ///     character_style,
    ///     style,
    /// );
    ///
    /// let size = text_box.bounding_box().size;
    /// assert_eq!(size, Size::new(60, 0));
    /// ```
    ///
    /// # Example: `ShrinkToText` shrinks the [`TextBox`].
    ///
    /// ```rust
    /// use embedded_graphics::{
    ///     mono_font::{ascii::FONT_6X9, MonoTextStyle},
    ///     pixelcolor::BinaryColor,
    ///     prelude::*,
    ///     primitives::Rectangle,
    /// };
    /// use embedded_text::{
    ///     style::{HeightMode, VerticalOverdraw, TextBoxStyleBuilder},
    ///     TextBox,
    /// };
    ///
    /// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
    /// let character_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
    ///
    /// let style = TextBoxStyleBuilder::new()
    ///     .height_mode(HeightMode::ShrinkToText(VerticalOverdraw::FullRowsOnly))
    ///     .build();
    ///
    /// // This TextBox contains two lines of text, but is 60px high
    /// let text_box = TextBox::with_textbox_style(
    ///     "Two lines\nof text",
    ///     Rectangle::new(Point::zero(), Size::new(60, 60)),
    ///     character_style,
    ///     style,
    /// );
    ///
    /// let size = text_box.bounding_box().size;
    /// assert_eq!(size, Size::new(60, 18));
    /// ```
    ShrinkToText(VerticalOverdraw),
}

impl HeightMode {
    /// Apply the height mode to the text box.
    ///
    /// *Note:* This function normally does not need to be called manually.
    pub(crate) fn apply<F>(self, text_box: &mut TextBox<'_, F>)
    where
        F: TextRenderer,
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
    pub(crate) fn calculate_displayed_row_range(self, cursor: &Cursor) -> Range<i32> {
        let overdraw = match self {
            HeightMode::Exact(overdraw) | HeightMode::ShrinkToText(overdraw) => overdraw,
            HeightMode::FitToText => VerticalOverdraw::Visible,
        };

        overdraw.calculate_displayed_row_range(cursor)
    }
}
