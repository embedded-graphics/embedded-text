//! Height adjustment options
//!
//! This module defines various options to handle height adjustment of a [`StyledTextBox`].
//! Although it is necessary to specify the size of the bounding box when constructing a
//! [`TextBox`], sometimes we may want the text to stretch the text box. Height modes help us
//! achieve this.
//!
//! [`TextBox`]: ../../struct.TextBox.html
use embedded_graphics::prelude::*;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
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
}

/// Keep the original height set while constructing the [`TextBox`].
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
pub struct Exact;

impl HeightMode for Exact {
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
}

/// If the text doesn't fill the original height, shrink the [`StyledTextBox`] to be as tall as the
/// text.
///
/// # Example: `ShrinkToText` does not grow the [`TextBox`].
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
///     .height_mode(ShrinkToText)
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
///     .height_mode(ShrinkToText)
///     .text_color(BinaryColor::On)
///     .build();
///
/// let size = text_box.into_styled(style).size();
/// assert_eq!(size, Size::new(60, 16));
/// ```
///
/// [`TextBox`]: ../../struct.TextBox.html
#[derive(Copy, Clone, Debug)]
pub struct ShrinkToText;

impl HeightMode for ShrinkToText {
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
