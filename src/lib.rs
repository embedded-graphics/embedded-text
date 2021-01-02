//! TextBox for embedded-graphics.
//!
//! This crate provides a configurable [`TextBox`] to render multiline text inside a bounding
//! `Rectangle` using [embedded-graphics].
//!
//! [`TextBox`] supports the common text alignments:
//!  - Horizontal:
//!      - [`LeftAligned`]
//!      - [`RightAligned`]
//!      - [`CenterAligned`]
//!      - [`Justified`]
//!  - Vertical:
//!      - [`TopAligned`]
//!      - [`CenterAligned`]
//!      - [`BottomAligned`]
//!      - [`Scrolling`]
//!
//! [`TextBox`] also supports some special characters not handled by embedded-graphics' `Text`:
//!  - non-breaking space (`\u{200b}`)
//!  - zero-width space (`\u{a0}`)
//!  - soft hyphen (`\u{ad}`)
//!  - carriage return (`\r`)
//!  - tab (`\t`) with configurable tab size
//!
//! `TextBox` also supports text coloring using [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code).
//!
//! ### Example
//!
//! The examples are based on [the embedded-graphics simulator]. The simulator is built on top of
//! `SDL2`. See the [simulator README] for more information.
//!
//! ![embedded-text example with center aligned text](https://raw.githubusercontent.com/embedded-graphics/embedded-text/master/assets/center.png)
//!
//! ![embedded-text example with colored text](https://raw.githubusercontent.com/embedded-graphics/embedded-text/master/assets/colored_text.png)
//!
//! ```rust,no_run
//! use embedded_graphics::{
//!     fonts::Font6x8, pixelcolor::BinaryColor, prelude::*,
//! };
//! use embedded_graphics_core::primitives::Rectangle;
//! use embedded_graphics_simulator::{
//!     BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
//! };
//! use embedded_text::prelude::*;
//!
//! fn main() {
//!     let text = "Hello, World!\n\
//!     Lorem Ipsum is simply dummy text of the printing and typesetting industry. \
//!     Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
//!     an unknown printer took a galley of type and scrambled it to make a type specimen book.";
//!
//!     // Specify the styling options:
//!     // * Use the 6x8 MonoFont from embedded-graphics.
//!     // * Draw the text horizontally left aligned (default option, not specified here).
//!     // * Use `FitToText` height mode to stretch the text box to the exact height of the text.
//!     // * Draw the text with `BinaryColor::On`, which will be displayed as light blue.
//!     let textbox_style = TextBoxStyleBuilder::new(Font6x8)
//!         .text_color(BinaryColor::On)
//!         .height_mode(FitToText)
//!         .build();
//!
//!     // Specify the bounding box. Note the 0px height. The `FitToText` height mode will
//!     // measure and adjust the height of the text box in `into_styled()`.
//!     let bounds = Rectangle::new(Point::zero(), Size::new(128, 0));
//!
//!     // Create the text box and apply styling options.
//!     let text_box = TextBox::new(text, bounds).into_styled(textbox_style);
//!
//!     // Create a simulated display with the dimensions of the text box.
//!     let mut display = SimulatorDisplay::new(text_box.bounding_box().size);
//!
//!     // Draw the text box.
//!     text_box.draw(&mut display).unwrap();
//!
//!     // Set up the window and show the display's contents.
//!     let output_settings = OutputSettingsBuilder::new()
//!         .theme(BinaryColorTheme::OledBlue)
//!         .build();
//!     Window::new("Left aligned TextBox example", &output_settings).show_static(&display);
//! }
//! ```
//!
//! ## Cargo features
//!
//! * `ansi`: enables ANSI sequence support. This feature is enabled by default.
//!
//! [embedded-graphics]: https://github.com/embedded-graphics/embedded-graphics/
//! [the embedded-graphics simulator]: https://github.com/embedded-graphics/embedded-graphics/tree/master/simulator
//! [simulator README]: https://github.com/embedded-graphics/embedded-graphics/tree/master/simulator#usage-without-sdl2
//! [`TextBox`]: ./struct.TextBox.html
//! [`LeftAligned`]: ./alignment/left/struct.LeftAligned.html
//! [`RightAligned`]: ./alignment/right/struct.RightAligned.html
//! [`CenterAligned`]: ./alignment/center/struct.CenterAligned.html
//! [`Justified`]: ./alignment/justified/struct.Justified.html
//! [`TopAligned`]: ./alignment/top/struct.TopAligned.html
//! [`BottomAligned`]: ./alignment/bottom/struct.BottomAligned.html
//! [`Scrolling`]: ./alignment/scrolling/struct.Scrolling.html

#![cfg_attr(not(test), no_std)]
#![deny(clippy::missing_inline_in_public_items)]
#![deny(clippy::cargo)]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::needless_doctest_main)]

pub mod alignment;
pub mod parser;
pub mod rendering;
pub mod style;
pub mod utils;

use alignment::{HorizontalTextAlignment, VerticalTextAlignment};
use embedded_graphics::prelude::*;
use embedded_graphics_core::primitives::Rectangle;
use style::{height_mode::HeightMode, TextBoxStyle};

/// Prelude.
///
/// A collection of useful imports. Also re-exports some types from `embedded-graphics` for
/// convenience.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        alignment::*,
        style::{
            height_mode::{Exact, FitToText, HeightMode, ShrinkToText},
            TabSize, TextBoxStyle, TextBoxStyleBuilder,
        },
        StyledTextBox, TextBox,
    };

    #[doc(no_inline)]
    pub use embedded_graphics::{
        primitives::Rectangle,
        style::{MonoTextStyle, MonoTextStyleBuilder},
    };
}

/// A textbox object.
///
/// The `TextBox` struct represents a piece of text that can be drawn on a display inside the given
/// bounding box.
///
/// The struct only contains the text and the bounding box, no additional information. To draw
/// a `TextBox` it is necessary to attach a [`TextBoxStyle`] to it using the [`into_styled`] method
/// to create a [`StyledTextBox`] object.
///
/// See the [module-level documentation] for more information.
///
/// [`into_styled`]: #method.into_styled
/// [`StyledTextBox`]: struct.StyledTextBox.html
/// [`TextBoxStyle`]: style/struct.TextBoxStyle.html
/// [module-level documentation]: index.html
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TextBox<'a> {
    /// The text to be displayed in this `TextBox`
    pub text: &'a str,

    /// The bounding box of this `TextBox`
    pub bounds: Rectangle,
}

impl<'a> TextBox<'a> {
    /// Creates a new `TextBox` instance with a given bounding `Rectangle`.
    #[inline]
    #[must_use]
    pub fn new(text: &'a str, bounds: Rectangle) -> Self {
        Self { text, bounds }
    }

    /// Creates a [`StyledTextBox`] by attaching a [`TextBoxStyle`] to the `TextBox` object.
    ///
    /// By default, the size of the [`StyledTextBox`] is equal to the size of the [`TextBox`]. Use
    /// [`HeightMode`] options to change this.
    ///
    /// # Example:
    ///
    /// In this example, we make a [`TextBox`] and give it all our available space as size.
    /// We create a [`TextBoxStyle`] object to set how our [`TextBox`] should be drawn.
    ///  * Set the 6x8 MonoFont
    ///  * Set the text color to `BinaryColor::On`
    ///  * Leave the background color transparent
    ///  * Leave text alignment top/left
    ///  * Set [`ShrinkToText`] [`HeightMode`] to shrink the [`TextBox`] when possible.
    ///
    /// ```rust
    /// use embedded_text::{prelude::*, style::vertical_overdraw::FullRowsOnly};
    /// use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
    /// use embedded_graphics_core::primitives::Rectangle;
    ///
    /// let text_box = TextBox::new(
    ///     "Two lines\nof text",
    ///     Rectangle::new(Point::zero(), Size::new(60, 60)),
    /// );
    /// let style = TextBoxStyleBuilder::new(Font6x8)
    ///     .height_mode(ShrinkToText(FullRowsOnly))
    ///     .text_color(BinaryColor::On)
    ///     .build();
    ///
    /// let styled_text_box = text_box.into_styled(style);
    /// assert_eq!(16, styled_text_box.bounding_box().size.height);
    /// ```
    ///
    /// [`HeightMode`]: style/height_mode/trait.HeightMode.html
    /// [`ShrinkToText`]: style/height_mode/struct.ShrinkToText.html
    #[inline]
    #[must_use]
    pub fn into_styled<C, F, A, V, H>(
        self,
        style: TextBoxStyle<C, F, A, V, H>,
    ) -> StyledTextBox<'a, C, F, A, V, H>
    where
        C: PixelColor,
        F: MonoFont,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        let mut styled = StyledTextBox {
            text_box: self,
            style,
        };
        H::apply(&mut styled);

        styled
    }
}

impl Transform for TextBox<'_> {
    #[inline]
    #[must_use]
    fn translate(&self, by: Point) -> Self {
        Self {
            bounds: self.bounds.translate(by),
            ..*self
        }
    }

    #[inline]
    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.bounds.translate_mut(by);

        self
    }
}

impl Dimensions for TextBox<'_> {
    #[inline]
    #[must_use]
    fn bounding_box(&self) -> Rectangle {
        self.bounds
    }
}

/// A styled [`TextBox`] struct.
///
/// This structure is constructed by calling the [`into_styled`] method of a [`TextBox`] object.
/// Use the [`draw`] method to draw the textbox on a display.
///
/// [`TextBox`]: struct.TextBox.html
/// [`into_styled`]: struct.TextBox.html#method.into_styled
/// [`draw`]: #method.draw
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct StyledTextBox<'a, C, F, A, V, H>
where
    C: PixelColor,
    F: MonoFont,
{
    /// A [`TextBox`] that has an associated [`TextBoxStyle`].
    ///
    /// [`TextBoxStyle`]: style/struct.TextBoxStyle.html
    pub text_box: TextBox<'a>,

    /// The style of the [`TextBox`].
    pub style: TextBoxStyle<C, F, A, V, H>,
}

impl<C, F, A, V, H> StyledTextBox<'_, C, F, A, V, H>
where
    C: PixelColor,
    F: MonoFont,
    A: HorizontalTextAlignment,
{
    /// Sets the height of the [`StyledTextBox`] to the height of the text.
    #[inline]
    pub fn fit_height(&mut self) -> &mut Self {
        self.fit_height_limited(u32::max_value())
    }

    /// Sets the height of the [`StyledTextBox`] to the height of the text, limited to `max_height`.
    ///
    /// This method allows you to set a maximum height. The [`StyledTextBox`] will take up at most
    /// `max_height` pixel vertical space.
    #[inline]
    pub fn fit_height_limited(&mut self, max_height: u32) -> &mut Self {
        // Measure text given the width of the textbox
        let text_height = self
            .style
            .measure_text_height(self.text_box.text, self.text_box.bounding_box().size.width)
            .min(max_height)
            .min(i32::max_value() as u32);

        // Apply height
        self.text_box.bounds.size.height = text_height;

        self
    }
}

impl<C, F, A, V, H> Transform for StyledTextBox<'_, C, F, A, V, H>
where
    C: PixelColor,
    F: MonoFont,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    #[inline]
    #[must_use]
    fn translate(&self, by: Point) -> Self {
        Self {
            text_box: self.text_box.translate(by),
            style: self.style,
        }
    }

    #[inline]
    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.text_box.bounds.translate_mut(by);

        self
    }
}

impl<C, F, A, V, H> Dimensions for StyledTextBox<'_, C, F, A, V, H>
where
    C: PixelColor,
    F: MonoFont,
{
    #[inline]
    #[must_use]
    fn bounding_box(&self) -> Rectangle {
        self.text_box.bounds
    }
}
