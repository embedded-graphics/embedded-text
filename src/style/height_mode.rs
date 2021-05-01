//! Height adjustment options.
//!
//! This module defines various options to set the height of a [`TextBox`]. Although it is
//! necessary to specify the size of a [`TextBox`], sometimes we may want the text to stretch or
//! shrink the text box. Height modes help us achieve this.
//!
//! [`TextBox`]: ../../struct.TextBox.html
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    rendering::cursor::Cursor,
    style::vertical_overdraw::VerticalOverdraw,
    TextBox,
};
use core::ops::Range;
use embedded_graphics::{geometry::Dimensions, text::renderer::TextRenderer};

/// Specifies how the [`TextBox`]'s height should be adjusted.
///
/// [`TextBox`]: ../../struct.TextBox.html
pub trait HeightMode: Copy + Sized {
    /// Apply the height mode to the text box.
    ///
    /// *Note:* This function normally does not need to be called manually.
    fn apply<F, A, V>(text_box: &mut TextBox<'_, F, A, V, Self>)
    where
        F: TextRenderer,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment;

    /// Calculate the range of rows of the current line that can be drawn.
    ///
    /// If a line does not fully fit in the bounding box, some `HeightMode` options allow drawing
    /// partial lines. For a partial line, this function calculates, which rows of each character
    /// should be displayed.
    fn calculate_displayed_row_range(self, cursor: &Cursor) -> Range<i32>;
}

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
///
/// [`TextBox`]: ../../struct.TextBox.html
#[derive(Copy, Clone, Debug)]
pub struct Exact(pub VerticalOverdraw);

impl HeightMode for Exact {
    #[inline]
    fn apply<F, A, V>(_text_box: &mut TextBox<'_, F, A, V, Self>)
    where
        F: TextRenderer,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
    {
    }

    #[inline]
    fn calculate_displayed_row_range(self, cursor: &Cursor) -> Range<i32> {
        self.0.calculate_displayed_row_range(cursor)
    }
}

/// Sets the height of the [`TextBox`] to exactly fit the text.
///
/// Note: in this mode, vertical alignment is meaningless. Make sure to use [`TopAligned`] for
/// efficiency.
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
///     style::{height_mode::FitToText, TextBoxStyleBuilder},
///     TextBox,
/// };
///
/// let character_style = MonoTextStyleBuilder::new()
///     .font(&FONT_6X9)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
/// let style = TextBoxStyleBuilder::new().height_mode(FitToText).build();
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
///     mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
///     pixelcolor::BinaryColor,
///     prelude::*,
///     primitives::Rectangle,
/// };
/// use embedded_text::{
///     style::{height_mode::FitToText, TextBoxStyleBuilder},
///     TextBox,
/// };
///
/// let character_style = MonoTextStyleBuilder::new()
///     .font(&FONT_6X9)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
/// let style = TextBoxStyleBuilder::new().height_mode(FitToText).build();
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
/// [`TopAligned`]: ../../alignment/top/struct.TopAligned.html
/// [`TextBox`]: ../../struct.TextBox.html
#[derive(Copy, Clone, Debug)]
pub struct FitToText;

impl HeightMode for FitToText {
    #[inline]
    fn apply<F, A, V>(text_box: &mut TextBox<'_, F, A, V, Self>)
    where
        F: TextRenderer,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
    {
        text_box.fit_height();
    }

    #[inline]
    fn calculate_displayed_row_range(self, cursor: &Cursor) -> Range<i32> {
        // FitToText always sets the bounding box to the exact size of the text, so every row is
        // always fully displayed
        0..cursor.line_height() as i32
    }
}

/// If the text does not fill the bounding box, shrink the [`TextBox`] to be as tall as the
/// text.
///
/// # Example: `ShrinkToText` does not grow the [`TextBox`].
///
/// ```rust
/// use embedded_graphics::{
///     mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
///     pixelcolor::BinaryColor,
///     prelude::*,
///     primitives::Rectangle,
/// };
/// use embedded_text::{
///     style::{height_mode::ShrinkToText, vertical_overdraw::VerticalOverdraw, TextBoxStyleBuilder},
///     TextBox,
/// };
///
/// let character_style = MonoTextStyleBuilder::new()
///     .font(&FONT_6X9)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
/// let style = TextBoxStyleBuilder::new()
///     .height_mode(ShrinkToText(VerticalOverdraw::FullRowsOnly))
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
///     mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
///     pixelcolor::BinaryColor,
///     prelude::*,
///     primitives::Rectangle,
/// };
/// use embedded_text::{
///     style::{height_mode::ShrinkToText, vertical_overdraw::VerticalOverdraw, TextBoxStyleBuilder},
///     TextBox,
/// };
///
/// let character_style = MonoTextStyleBuilder::new()
///     .font(&FONT_6X9)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
/// let style = TextBoxStyleBuilder::new()
///     .height_mode(ShrinkToText(VerticalOverdraw::FullRowsOnly))
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
///
/// [`TextBox`]: ../../struct.TextBox.html
#[derive(Copy, Clone, Debug)]
pub struct ShrinkToText(pub VerticalOverdraw);

impl HeightMode for ShrinkToText {
    #[inline]
    fn apply<F, A, V>(text_box: &mut TextBox<'_, F, A, V, Self>)
    where
        F: TextRenderer,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
    {
        text_box.fit_height_limited(text_box.bounding_box().size.height);
    }

    #[inline]
    fn calculate_displayed_row_range(self, cursor: &Cursor) -> Range<i32> {
        self.0.calculate_displayed_row_range(cursor)
    }
}
