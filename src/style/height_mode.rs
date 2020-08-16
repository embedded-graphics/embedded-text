//! Height adjustment options
//!
//! This module defines various options to handle height adjustment of a [`StyledTextBox`].
//! Although it is necessary to specify the size of the bounding box when constructing a
//! [`TextBox`], sometimes we may want the text to stretch the text box. Height modes help us
//! achieve this.

use embedded_graphics::prelude::*;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    style::StyledTextBox,
};

/// Specifies how the [`TextBox`]'s height is adjusted when it is turned into a [`StyledTextBox`].
pub trait HeightMode: Copy {
    /// Apply the height mode to the textbox
    fn apply<C, F, A, V, H>(text_box: &mut StyledTextBox<'_, C, F, A, V, H>)
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode;
}

/// Keep the original height set while constructing the [`TextBox`].
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

/// Change the height to exactly fit the text
///
/// Note: in this mode, vertical alignment is meaningless. Make sure to use [`TopAligned`] for
/// efficiency.
///
/// [`TopAligned`]: ../alignment/top/struct.TopAligned.html
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
