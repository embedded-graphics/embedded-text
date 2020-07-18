//! This crate implements rendering text in a given area for embedded-graphics
#![cfg_attr(not(test), no_std)]

use embedded_graphics::{prelude::*, primitives::Rectangle};

/// Horizontal text alignment opitons
pub mod alignment;

/// Parse text into smaller units
pub mod parser;

/// Helpers to render text
pub mod rendering;

/// Textbox styling
pub mod style;

use style::{StyledTextBox, TextBoxStyle};

/// Text alignment
pub trait TextAlignment {}

/// A piece of text with an associated area on the display
pub struct TextBox<'a> {
    pub text: &'a str,
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
        F: Font,
        A: TextAlignment,
    {
        StyledTextBox {
            text_box: self,
            style,
        }
    }
}
