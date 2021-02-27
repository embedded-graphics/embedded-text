//! Height adjustment options.
//!
//! This module defines various options to set the height of a [`StyledTextBox`]. Although it is
//! necessary to specify the size of a [`TextBox`], sometimes we may want the text to stretch or
//! shrink the text box. Height modes help us achieve this.
//!
//! [`TextBox`]: ../../struct.TextBox.html
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    rendering::cursor::Cursor,
    style::{color::Rgb, vertical_overdraw::VerticalOverdraw},
    StyledTextBox,
};
use core::ops::Range;
use embedded_graphics::{
    geometry::Dimensions,
    text::{CharacterStyle, TextRenderer},
};

/// Specifies how the [`TextBox`]'s height is adjusted when it is turned into a [`StyledTextBox`].
///
/// [`TextBox`]: ../../struct.TextBox.html
pub trait HeightMode: Copy {
    /// Apply the height mode to the textbox
    ///
    /// *Note:* This function is used by [`TextBox::into_styled`] and normally does not need to be
    /// called manually.
    ///
    /// [`TextBox::into_styled`]: ../../struct.TextBox.html#method.into_styled
    fn apply<F, A, V, H>(text_box: &mut StyledTextBox<'_, F, A, V, H>)
    where
        F: TextRenderer + CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb>,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode;

    /// Calculate the range of rows of the current line that can be drawn.
    ///
    /// If a line does not fully fit in the bounding box, some `HeightMode` options allow drawing
    /// partial lines. For a partial line, this function calculates, which rows of each character
    /// should be displayed.
    fn calculate_displayed_row_range(cursor: &Cursor) -> Range<i32>;
}

/// Keep the original [`TextBox`] height.
///
/// # Example:
///
/// ```rust
/// use embedded_text::prelude::*;
/// use embedded_graphics::{
///     mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
///     pixelcolor::BinaryColor,
///     prelude::*,
/// };
///
/// let character_style = MonoTextStyleBuilder::new()
///     .font(Font6x9)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
/// let style = TextBoxStyleBuilder::new()
///     .character_style(character_style)
///     .build();
///
/// // This TextBox contains two lines of text, but is 60px high
/// let text_box = TextBox::new(
///     "Two lines\nof text",
///     Rectangle::new(Point::zero(), Size::new(60, 60)),
/// );
///
/// // Exact does not change the size of the TextBox
/// let orig_size = text_box.bounding_box().size;
/// let size = text_box.into_styled(style).bounding_box().size;
/// assert_eq!(size, orig_size);
/// ```
///
/// [`TextBox`]: ../../struct.TextBox.html
#[derive(Copy, Clone, Debug)]
pub struct Exact<OV: VerticalOverdraw>(pub OV);

impl<OV> HeightMode for Exact<OV>
where
    OV: VerticalOverdraw,
{
    #[inline]
    fn apply<F, A, V, H>(_text_box: &mut StyledTextBox<'_, F, A, V, H>)
    where
        F: TextRenderer + CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb>,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
    }

    #[inline]
    fn calculate_displayed_row_range(cursor: &Cursor) -> Range<i32> {
        OV::calculate_displayed_row_range(cursor)
    }
}

/// Sets the height of the [`StyledTextBox`] to exactly fit the text.
///
/// Note: in this mode, vertical alignment is meaningless. Make sure to use [`TopAligned`] for
/// efficiency.
///
/// # Example: `FitToText` grows the [`TextBox`].
///
/// ```rust
/// use embedded_text::prelude::*;
/// use embedded_graphics::{
///     mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
///     pixelcolor::BinaryColor,
///     prelude::*,
/// };
///
/// let character_style = MonoTextStyleBuilder::new()
///     .font(Font6x9)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
/// let style = TextBoxStyleBuilder::new()
///     .character_style(character_style)
///     .height_mode(FitToText)
///     .build();
///
/// // This TextBox contains two lines of text, but is 1px high
/// let text_box = TextBox::new(
///     "Two lines\nof text",
///     Rectangle::new(Point::zero(), Size::new(60, 0)),
/// );
///
/// // FitToText grows the TextBox to the height of the text
/// let size = text_box.into_styled(style).bounding_box().size;
/// assert_eq!(size, Size::new(60, 18));
/// ```
///
/// # Example: `FitToText` also shrinks the [`TextBox`].
///
/// ```rust
/// use embedded_text::prelude::*;
/// use embedded_graphics::{
///     mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
///     pixelcolor::BinaryColor,
///     prelude::*,
/// };
///
/// let character_style = MonoTextStyleBuilder::new()
///     .font(Font6x9)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
/// let style = TextBoxStyleBuilder::new()
///     .character_style(character_style)
///     .height_mode(FitToText)
///     .build();
///
/// // This TextBox contains two lines of text, but is 1px high
/// let text_box = TextBox::new(
///     "Two lines\nof text",
///     Rectangle::new(Point::zero(), Size::new(60, 60)),
/// );
///
/// // FitToText shrinks the TextBox to the height of the text
/// let size = text_box.into_styled(style).bounding_box().size;
/// assert_eq!(size, Size::new(60, 18));
/// ```
///
/// [`TopAligned`]: ../../alignment/top/struct.TopAligned.html
/// [`TextBox`]: ../../struct.TextBox.html
#[derive(Copy, Clone, Debug)]
pub struct FitToText;

impl HeightMode for FitToText {
    #[inline]
    fn apply<F, A, V, H>(text_box: &mut StyledTextBox<'_, F, A, V, H>)
    where
        F: TextRenderer + CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb>,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        text_box.fit_height();
    }

    #[inline]
    fn calculate_displayed_row_range(cursor: &Cursor) -> Range<i32> {
        // FitToText always sets the bounding box to the exact size of the text, so every row is
        // always fully displayed
        0..cursor.line_height() as i32
    }
}

/// If the text does not fill the bounding box, shrink the [`StyledTextBox`] to be as tall as the
/// text.
///
/// # Example: `ShrinkToText` does not grow the [`TextBox`].
///
/// ```rust
/// use embedded_text::{prelude::*, style::vertical_overdraw::FullRowsOnly};
/// use embedded_graphics::{
///     mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
///     pixelcolor::BinaryColor,
///     prelude::*,
/// };
///
/// let character_style = MonoTextStyleBuilder::new()
///     .font(Font6x9)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
/// let style = TextBoxStyleBuilder::new()
///     .character_style(character_style)
///     .height_mode(ShrinkToText(FullRowsOnly))
///     .build();
///
/// // This TextBox contains two lines of text, but is 1px high
/// let text_box = TextBox::new(
///     "Two lines\nof text",
///     Rectangle::new(Point::zero(), Size::new(60, 0)),
/// );
///
/// let size = text_box.into_styled(style).bounding_box().size;
/// assert_eq!(size, Size::new(60, 0));
/// ```
///
/// # Example: `ShrinkToText` shrinks the [`TextBox`].
///
/// ```rust
/// use embedded_text::{prelude::*, style::vertical_overdraw::FullRowsOnly};
/// use embedded_graphics::{
///     mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
///     pixelcolor::BinaryColor,
///     prelude::*,
/// };
///
/// let character_style = MonoTextStyleBuilder::new()
///     .font(Font6x9)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // Set style, use 6x9 MonoFont so the 2 lines are 18px high.
/// let style = TextBoxStyleBuilder::new()
///     .character_style(character_style)
///     .height_mode(ShrinkToText(FullRowsOnly))
///     .build();
///
/// // This TextBox contains two lines of text, but is 60px high
/// let text_box = TextBox::new(
///     "Two lines\nof text",
///     Rectangle::new(Point::zero(), Size::new(60, 60)),
/// );
///
/// let size = text_box.into_styled(style).bounding_box().size;
/// assert_eq!(size, Size::new(60, 18));
/// ```
///
/// [`TextBox`]: ../../struct.TextBox.html
#[derive(Copy, Clone, Debug)]
pub struct ShrinkToText<OV: VerticalOverdraw>(pub OV);

impl<OV> HeightMode for ShrinkToText<OV>
where
    OV: VerticalOverdraw,
{
    #[inline]
    fn apply<F, A, V, H>(text_box: &mut StyledTextBox<'_, F, A, V, H>)
    where
        F: TextRenderer + CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb>,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        text_box.fit_height_limited(text_box.bounding_box().size.height);
    }

    #[inline]
    fn calculate_displayed_row_range(cursor: &Cursor) -> Range<i32> {
        OV::calculate_displayed_row_range(cursor)
    }
}
