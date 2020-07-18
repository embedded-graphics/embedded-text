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
        // TODO: remove if fixed in embedded-graphics
        let width = (self.bottom_right().x - self.top_left().x) as u32 + 1;
        let height = (self.bottom_right().y - self.top_left().y) as u32 + 1;

        Size::new(width, height)
    }
}
