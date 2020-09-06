//! Height adjustment options.
//!
//! This module defines various options to set the height of a [`StyledTextBox`]. Although it is
//! necessary to specify the size of a [`TextBox`], sometimes we may want the text to stretch or
//! shrink the text box. Height modes help us achieve this.
//!
//! [`TextBox`]: ../../struct.TextBox.html
use core::ops::Range;
use embedded_graphics::prelude::*;

use crate::style::vertical_overdraw::VerticalOverdraw;
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    rendering::cursor::Cursor,
    StyledTextBox,
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
    fn apply<C, F, A, V, H>(text_box: &mut StyledTextBox<'_, C, F, A, V, H>)
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode;

    /// Calculate the range of rows of the current line that can be drawn.
    ///
    /// If a line does not fully fit in the bounding box, some `HeightMode` options allow drawing
    /// partial lines. For a partial line, this function calculates, which rows of each character
    /// should be displayed.
    fn calculate_displayed_row_range<F: Font>(cursor: &Cursor<F>) -> Range<i32> {
        // TODO this default code only covers the "full lines only" mode.
        if cursor.in_display_area() {
            0..F::CHARACTER_SIZE.height as i32
        } else {
            0..0
        }
    }
}

/// Keep the original [`TextBox`] height.
///
/// # Example:
///
/// ```rust
/// use embedded_text::prelude::*;
/// use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
///
/// // This TextBox contains two lines of text, but is 60px high
/// let text_box = TextBox::new(
///     "Two lines\nof text",
///     Rectangle::new(Point::zero(), Point::new(59, 59)),
/// );
///
/// // Set style, use 6x8 font so the 2 lines are 16px high.
/// let style = TextBoxStyleBuilder::new(Font6x8)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // Exact does not change the size of the TextBox
/// let orig_size = text_box.size();
/// let size = text_box.into_styled(style).size();
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
    fn apply<C, F, A, V, H>(_text_box: &mut StyledTextBox<'_, C, F, A, V, H>)
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
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
/// use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
///
/// // This TextBox contains two lines of text, but is 1px high
/// let text_box = TextBox::new(
///     "Two lines\nof text",
///     Rectangle::new(Point::zero(), Point::new(59, 0)),
/// );
///
/// // Set style, use 6x8 font so the 2 lines are 16px high.
/// let style = TextBoxStyleBuilder::new(Font6x8)
///     .height_mode(FitToText)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // FitToText grows the TextBox to the height of the text
/// let size = text_box.into_styled(style).size();
/// assert_eq!(size, Size::new(60, 16));
/// ```
///
/// # Example: `FitToText` also shrinks the [`TextBox`].
///
/// ```rust
/// use embedded_text::prelude::*;
/// use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
///
/// // This TextBox contains two lines of text, but is 1px high
/// let text_box = TextBox::new(
///     "Two lines\nof text",
///     Rectangle::new(Point::zero(), Point::new(59, 59)),
/// );
///
/// // Set style, use 6x8 font so the 2 lines are 16px high.
/// let style = TextBoxStyleBuilder::new(Font6x8)
///     .height_mode(FitToText)
///     .text_color(BinaryColor::On)
///     .build();
///
/// // FitToText shrinks the TextBox to the height of the text
/// let size = text_box.into_styled(style).size();
/// assert_eq!(size, Size::new(60, 16));
/// ```
///
/// [`TopAligned`]: ../../alignment/top/struct.TopAligned.html
/// [`TextBox`]: ../../struct.TextBox.html
#[derive(Copy, Clone, Debug)]
pub struct FitToText;

impl HeightMode for FitToText {
    #[inline]
    fn apply<C, F, A, V, H>(text_box: &mut StyledTextBox<'_, C, F, A, V, H>)
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        text_box.fit_height();
    }

    #[inline]
    fn calculate_displayed_row_range<F: Font>(_: &Cursor<F>) -> Range<i32> {
        // FitToText always sets the bounding box to the exact size of the text, so every row is
        // always fully displayed
        0..F::CHARACTER_SIZE.height as i32
    }
}

/// If the text does not fill the bounding box, shrink the [`StyledTextBox`] to be as tall as the
/// text.
///
/// # Example: `ShrinkToText` does not grow the [`TextBox`].
///
/// ```rust
/// use embedded_text::{prelude::*, style::vertical_overdraw::FullRowsOnly};
/// use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
///
/// // This TextBox contains two lines of text, but is 1px high
/// let text_box = TextBox::new(
///     "Two lines\nof text",
///     Rectangle::new(Point::zero(), Point::new(59, 0)),
/// );
///
/// // Set style, use 6x8 font so the 2 lines are 16px high.
/// let style = TextBoxStyleBuilder::new(Font6x8)
///     .height_mode(ShrinkToText(FullRowsOnly))
///     .text_color(BinaryColor::On)
///     .build();
///
/// let size = text_box.into_styled(style).size();
/// assert_eq!(size, Size::new(60, 1));
/// ```
///
/// # Example: `ShrinkToText` shrinks the [`TextBox`].
///
/// ```rust
/// use embedded_text::{prelude::*, style::vertical_overdraw::FullRowsOnly};
/// use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
///
/// // This TextBox contains two lines of text, but is 60px high
/// let text_box = TextBox::new(
///     "Two lines\nof text",
///     Rectangle::new(Point::zero(), Point::new(59, 59)),
/// );
///
/// // Set style, use 6x8 font so the 2 lines are 16px high.
/// let style = TextBoxStyleBuilder::new(Font6x8)
///     .height_mode(ShrinkToText(FullRowsOnly))
///     .text_color(BinaryColor::On)
///     .build();
///
/// let size = text_box.into_styled(style).size();
/// assert_eq!(size, Size::new(60, 16));
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
    fn apply<C, F, A, V, H>(text_box: &mut StyledTextBox<'_, C, F, A, V, H>)
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        text_box.fit_height_limited(text_box.size().height);
    }
}
