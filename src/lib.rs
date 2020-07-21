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
//! ![embedded-text example](assets/example.png)
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
//! ## Development setup
//!
//! ### Minimum supported Rust version
//! The minimum supported Rust version for embedded-layout is 1.40.0 or greater. However, the documentation uses the `intra-crate links` feature which requires nightly Rust. Ensure you have the latest stable version of Rust installed, preferably through https://rustup.rs.
//!
//! ### Installation
//!
//! For setup in general, follow the installation instructions for [`embedded-graphics`].
//!
//! To install SDL2 on Windows, see https://github.com/Rust-SDL2/rust-sdl2#windows-msvc
//!
//! ## Attribution
//!
//! The example text is copied from https://www.lipsum.com
//!
//! [embedded-graphics]: https://github.com/jamwaffles/embedded-graphics/
//! [the embedded-graphics simulator]: https://github.com/jamwaffles/embedded-graphics/tree/master/simulator
//! [simulator README]: https://github.com/jamwaffles/embedded-graphics/tree/master/simulator#usage-without-sdl2

#![cfg_attr(not(test), no_std)]
#![deny(clippy::missing_inline_in_public_items)]
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
