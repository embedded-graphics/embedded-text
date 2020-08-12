//! TextBox for embedded-graphics.
//!
//! This crate provides a configurable [`TextBox`] to render multiline text inside a bounding
//! `Rectangle` using [embedded-graphics].
//!
//! [`TextBox`] supports the common text alignments:
//!  - [`LeftAligned`]
//!  - [`RightAligned`]
//!  - [`CenterAligned`]
//!  - [`Justified`]
//!
//! ## Example
//!
//! The examples are based on [the embedded-graphics simulator]. The simulator is built on top of
//! `SDL2`. See the [simulator README] for more information.
//!
//! ![embedded-text example with center aligned text](https://raw.githubusercontent.com/bugadani/embedded-text/master/assets/center.png)
//!
//! ```rust,no_run
//! use embedded_graphics_simulator::{
//!     BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
//! };
//!
//! use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
//!
//! use embedded_text::{alignment::horizontal::CenterAligned, prelude::*};
//!
//! fn main() -> Result<(), core::convert::Infallible> {
//!     let text = "Hello, World!\nLorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.";
//!
//!     let textbox_style = TextBoxStyleBuilder::new(Font6x8)
//!         .alignment(CenterAligned)
//!         .text_color(BinaryColor::On)
//!         .build();
//!
//!     let height = textbox_style.measure_text_height(text, 129);
//!
//!     // Create a window just tall enough to fit the text.
//!     let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(129, height));
//!
//!     TextBox::new(
//!         text,
//!         Rectangle::new(Point::zero(), Point::new(128, height as i32 - 1)),
//!     )
//!     .into_styled(textbox_style)
//!     .draw(&mut display)
//!     .unwrap();
//!
//!     let output_settings = OutputSettingsBuilder::new()
//!         .theme(BinaryColorTheme::OledBlue)
//!         .build();
//!     Window::new("Hello center aligned TextBox", &output_settings).show_static(&display);
//!     Ok(())
//! }
//! ```
//!
//! [embedded-graphics]: https://github.com/jamwaffles/embedded-graphics/
//! [the embedded-graphics simulator]: https://github.com/jamwaffles/embedded-graphics/tree/master/simulator
//! [simulator README]: https://github.com/jamwaffles/embedded-graphics/tree/master/simulator#usage-without-sdl2
//! [`TextBox`]: ./struct.TextBox.html
//! [`LeftAligned`]: ./alignment/horizontal/left/struct.LeftAligned.html
//! [`RightAligned`]: ./alignment/horizontal/right/struct.RightAligned.html
//! [`CenterAligned`]: ./alignment/horizontal/center/struct.CenterAligned.html
//! [`Justified`]: ./alignment/horizontal/justified/struct.Justified.html

#![cfg_attr(not(test), no_std)]
#![deny(clippy::missing_inline_in_public_items)]
#![deny(clippy::cargo)]
#![deny(missing_docs)]
#![warn(clippy::all)]

use embedded_graphics::{prelude::*, primitives::Rectangle};

pub mod alignment;
pub mod parser;
pub mod rendering;
pub mod style;
pub mod utils;

use alignment::{horizontal::HorizontalTextAlignment, vertical::VerticalTextAlignment};
use style::{StyledTextBox, TextBoxStyle};

/// Prelude.
///
/// A collection of useful imports. Also re-exports some types from `embedded-graphics` for
/// convenience.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        style::{TextBoxStyle, TextBoxStyleBuilder},
        TextBox,
    };

    #[doc(no_inline)]
    pub use embedded_graphics::{
        primitives::Rectangle,
        style::{TextStyle, TextStyleBuilder},
    };
}

/// A textbox object.
///
/// The `TextBox` struct represents a piece of text that can be drawn on a display inside the given
/// bounding box.
///
/// The struct only contains the text and the bounding box, no additional information. To draw
/// a textbox it is necessary to attach a style to it using the [`into_styled`] method to create a
/// [`StyledTextBox`] object.
///
/// See the [module-level documentation] for more information.
///
/// [`into_styled`]: #method.into_styled
/// [`StyledTextBox`]: style/struct.StyledTextBox.html
/// [module-level documentation]: index.html
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

    /// Attaches a [`TextBoxStyle`] to the textbox object.
    #[inline]
    #[must_use]
    pub fn into_styled<C, F, A, V>(
        self,
        style: TextBoxStyle<C, F, A, V>,
    ) -> StyledTextBox<'a, C, F, A, V>
    where
        C: PixelColor,
        F: Font + Copy,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
    {
        StyledTextBox {
            text_box: self,
            style,
        }
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
    fn top_left(&self) -> Point {
        self.bounds.top_left
    }

    #[inline]
    #[must_use]
    fn bottom_right(&self) -> Point {
        self.bounds.bottom_right
    }

    #[inline]
    #[must_use]
    fn size(&self) -> Size {
        crate::utils::rect_ext::RectExt::size(self.bounds)
    }
}
