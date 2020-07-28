//! TextBox for embedded-graphics
//!
//! This crate provides a configurable `TextBox` to render multiline text using [embedded-graphics].
//!
//! `TextBox` supports the common text alignments:
//!  - `LeftAligned`
//!  - `RightAligned`
//!  - `CenterAligned`
//!  - `Justified`
//!
//! ## Example
//!
//! The examples are based on [the embedded-graphics simulator]. The simulator is built on top of
//! `SDL2`. See the [simulator README] for more information.
//!
//! ![embedded-text example with center aligned text](assets/center.png)
//!
//! ```rust,no_run
//! use embedded_graphics_simulator::{
//!     BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
//! };
//!
//! use embedded_graphics::{
//!     fonts::Font6x8, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
//! };
//!
//! use embedded_text::{alignment::CenterAligned, style::TextBoxStyleBuilder, TextBox};
//!
//! fn main() -> Result<(), core::convert::Infallible> {
//!     let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(129, 129));
//!
//!     let textbox_style = TextBoxStyleBuilder::new(Font6x8)
//!         .alignment(CenterAligned)
//!         .text_color(BinaryColor::On)
//!         .build();
//!
//!     TextBox::new(
//!         "Hello, World!\nLorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.",
//!         Rectangle::new(Point::zero(), Point::new(128, 128)),
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

#![cfg_attr(not(test), no_std)]
#![deny(clippy::missing_inline_in_public_items)]
#![deny(clippy::cargo)]
#![deny(missing_docs)]
#![warn(clippy::all)]

use embedded_graphics::{prelude::*, primitives::Rectangle};

/// Horizontal text alignment opitons
pub mod alignment;

/// Parse text into smaller units
pub mod parser;

/// Helpers to render text
pub mod rendering;

/// Textbox styling
pub mod style;

/// Helpers
pub mod utils;

use alignment::TextAlignment;
use style::{StyledTextBox, TextBoxStyle};

/// Prelude
///
/// Useful imports
pub mod prelude {
    pub use crate::{
        style::{TextBoxStyle, TextBoxStyleBuilder},
        TextBox,
    };

    pub use embedded_graphics::{
        primitives::Rectangle,
        style::{TextStyle, TextStyleBuilder},
    };
}

/// A piece of text with an associated area on the display
pub struct TextBox<'a> {
    /// The text to be displayed in this `TextBox`
    pub text: &'a str,

    /// The bounding box of this `TextBox`
    pub bounds: Rectangle,
}

impl<'a> TextBox<'a> {
    /// Creates a new `TextBox` instance with a given bounding box.
    #[inline]
    #[must_use]
    pub fn new(text: &'a str, bounds: Rectangle) -> Self {
        Self { text, bounds }
    }

    /// Attaches a textbox style to the textbox object.
    #[inline]
    #[must_use]
    pub fn into_styled<C, F, A>(self, style: TextBoxStyle<C, F, A>) -> StyledTextBox<'a, C, F, A>
    where
        C: PixelColor,
        F: Font + Copy,
        A: TextAlignment,
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
