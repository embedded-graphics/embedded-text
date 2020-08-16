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

/// Change the height to exactly fit the text
///
/// Note: in this mode, vertical alignment is meaningless. Make sure to use [`TopAligned`] for
/// efficiency.
///
/// [`TopAligned`]: ../../alignment/top/struct.TopAligned.html
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

#[cfg(test)]
mod test_styled {
    use super::*;
    use crate::{
        prelude::*,
        style::height_mode::{FitToText, ShrinkToText},
    };
    use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor};

    #[test]
    fn test_exact() {
        // way too high text box
        let text_box = TextBox::new(
            "Two lines\nof text",
            Rectangle::new(Point::zero(), Point::new(59, 60)),
        );

        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .build();

        let orig_size = text_box.size();
        let size = text_box.into_styled(style).size();

        assert_eq!(size, orig_size);
    }

    #[test]
    fn test_shrink_bigger() {
        // way too high text box
        let text_box = TextBox::new(
            "Two lines\nof text",
            Rectangle::new(Point::zero(), Point::new(59, 60)),
        );

        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .height_mode(ShrinkToText)
            .build();

        let size = text_box.into_styled(style).size();

        assert_eq!(size, Size::new(60, 16));
    }

    #[test]
    fn test_shrink_smaller() {
        // way too high text box
        let text_box = TextBox::new(
            "Two lines\nof text",
            Rectangle::new(Point::zero(), Point::new(59, 7)),
        );

        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .height_mode(ShrinkToText)
            .build();

        let size = text_box.into_styled(style).size();

        assert_eq!(size, Size::new(60, 8));
    }

    #[test]
    fn test_fit_bigger() {
        // way too high text box
        let text_box = TextBox::new(
            "Two lines\nof text",
            Rectangle::new(Point::zero(), Point::new(59, 60)),
        );

        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .height_mode(FitToText)
            .build();

        let size = text_box.into_styled(style).size();

        assert_eq!(size, Size::new(60, 16));
    }

    #[test]
    fn test_fit_smaller() {
        // way too high text box
        let text_box = TextBox::new(
            "Two lines\nof text",
            Rectangle::new(Point::zero(), Point::new(59, 0)),
        );

        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .height_mode(FitToText)
            .build();

        let size = text_box.into_styled(style).size();

        assert_eq!(size, Size::new(60, 16));
    }
}
