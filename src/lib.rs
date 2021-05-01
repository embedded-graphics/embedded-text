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
//!     mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
//!     pixelcolor::BinaryColor,
//!     prelude::*,
//!     primitives::Rectangle,
//! };
//! use embedded_graphics_simulator::{
//!     BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
//! };
//! use embedded_text::{
//!     style::{height_mode::FitToText, TextBoxStyleBuilder},
//!     TextBox,
//! };
//! fn main() {
//!     let text = "Hello, World!\n\
//!     Lorem Ipsum is simply dummy text of the printing and typesetting industry. \
//!     Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
//!     an unknown printer took a galley of type and scrambled it to make a type specimen book.";
//!
//!     // Specify the styling options:
//!     // * Use the 6x9 monospace Font from embedded-graphics.
//!     // * Draw the text horizontally left aligned (default option, not specified here).
//!     // * Use `FitToText` height mode to stretch the text box to the exact height of the text.
//!     // * Draw the text with `BinaryColor::On`, which will be displayed as light blue.
//!     let character_style = MonoTextStyleBuilder::new()
//!         .font(&FONT_6X9)
//!         .text_color(BinaryColor::On)
//!         .build();
//!
//!     let textbox_style = TextBoxStyleBuilder::new().height_mode(FitToText).build();
//!
//!     // Specify the bounding box. Note the 0px height. The `FitToText` height mode will
//!     // measure and adjust the height of the text box in `into_styled()`.
//!     let bounds = Rectangle::new(Point::zero(), Size::new(128, 0));
//!
//!     // Create the text box and apply styling options.
//!     let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);
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
//!
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
mod parser;
mod rendering;
pub mod style;

mod utils;

use crate::{
    alignment::{HorizontalTextAlignment, LeftAligned, TopAligned, VerticalTextAlignment},
    style::{
        height_mode::{Exact, HeightMode},
        TextBoxStyle,
    },
};
use embedded_graphics::{
    geometry::{Dimensions, Point},
    primitives::Rectangle,
    text::renderer::{CharacterStyle, TextRenderer},
    transform::Transform,
};

/// A text box object.
///
/// The `TextBox` struct represents a piece of text that can be drawn on a display inside the given
/// bounding box.
///
/// Use the [`draw`] method to draw the textbox on a display.
///
/// See the [module-level documentation] for more information.
///
/// [`into_styled`]: #method.into_styled
/// [`TextBoxStyle`]: style/struct.TextBoxStyle.html
/// [module-level documentation]: index.html
/// [`draw`]: #method.draw
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TextBox<'a, S, A, V, H> {
    /// The text to be displayed in this `TextBox`
    pub text: &'a str,

    /// The bounding box of this `TextBox`
    pub bounds: Rectangle,

    /// The character style of the [`TextBox`].
    pub character_style: S,

    /// The style of the [`TextBox`].
    pub style: TextBoxStyle<A, V, H>,
}

impl<'a, S> TextBox<'a, S, LeftAligned, TopAligned, Exact>
where
    S: TextRenderer + CharacterStyle,
{
    /// Creates a new `TextBox` instance with a given bounding `Rectangle`.
    #[inline]
    #[must_use]
    pub fn new(text: &'a str, bounds: Rectangle, character_style: S) -> Self {
        TextBox::with_textbox_style(text, bounds, character_style, TextBoxStyle::default())
    }
}

impl<'a, S, A, V, H> TextBox<'a, S, A, V, H>
where
    S: TextRenderer + CharacterStyle,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    /// Creates a new `TextBox` instance with a given bounding `Rectangle` and a given `TextBoxStyle`.
    #[inline]
    #[must_use]
    pub fn with_textbox_style(
        text: &'a str,
        bounds: Rectangle,
        character_style: S,
        textbox_style: TextBoxStyle<A, V, H>,
    ) -> Self {
        let mut styled = TextBox {
            text,
            bounds,
            character_style,
            style: textbox_style,
        };

        H::apply(&mut styled);

        styled
    }

    /// Creates a new `TextBox` instance with a given bounding `Rectangle` and a given `TextBoxStyle`.
    #[inline]
    #[must_use]
    pub fn with_alignment(
        text: &'a str,
        bounds: Rectangle,
        character_style: S,
        alignment: A,
    ) -> TextBox<'a, S, A, TopAligned, Exact> {
        TextBox::with_textbox_style(
            text,
            bounds,
            character_style,
            TextBoxStyle::with_alignment(alignment),
        )
    }

    /// Creates a new `TextBox` instance with a given bounding `Rectangle` and a given `TextBoxStyle`.
    #[inline]
    #[must_use]
    pub fn with_vertical_alignment(
        text: &'a str,
        bounds: Rectangle,
        character_style: S,
        vertical_alignment: V,
    ) -> TextBox<'a, S, LeftAligned, V, Exact> {
        TextBox::with_textbox_style(
            text,
            bounds,
            character_style,
            TextBoxStyle::with_vertical_alignment(vertical_alignment),
        )
    }
}

impl<S, A, V, H> Transform for TextBox<'_, S, A, V, H>
where
    Self: Clone,
{
    #[inline]
    #[must_use]
    fn translate(&self, by: Point) -> Self {
        Self {
            bounds: self.bounds.translate(by),
            ..self.clone()
        }
    }

    #[inline]
    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.bounds.translate_mut(by);

        self
    }
}

impl<S, A, V, H> Dimensions for TextBox<'_, S, A, V, H> {
    #[inline]
    #[must_use]
    fn bounding_box(&self) -> Rectangle {
        self.bounds
    }
}

impl<S, A, V, H> TextBox<'_, S, A, V, H>
where
    S: TextRenderer,
    A: HorizontalTextAlignment,
{
    /// Sets the height of the [`TextBox`] to the height of the text.
    #[inline]
    pub fn fit_height(&mut self) -> &mut Self {
        self.fit_height_limited(u32::max_value())
    }

    /// Sets the height of the [`TextBox`] to the height of the text, limited to `max_height`.
    ///
    /// This method allows you to set a maximum height. The [`TextBox`] will take up at most
    /// `max_height` pixel vertical space.
    #[inline]
    pub fn fit_height_limited(&mut self, max_height: u32) -> &mut Self {
        // Measure text given the width of the textbox
        let text_height = self
            .style
            .measure_text_height(
                &self.character_style,
                self.text,
                self.bounding_box().size.width,
            )
            .min(max_height)
            .min(i32::max_value() as u32);

        // Apply height
        self.bounds.size.height = text_height;

        self
    }
}
