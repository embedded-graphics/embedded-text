use embedded_graphics::{prelude::*, style::TextStyle};

pub mod builder;

use crate::{TextAlignment, TextBox};
pub use builder::TextBoxStyleBuilder;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct TextBoxStyle<C, F, A>
where
    C: PixelColor,
    F: Font,
    A: TextAlignment,
{
    pub text_style: TextStyle<C, F>,
    pub alignment: A,
}

impl<C, F, A> TextBoxStyle<C, F, A>
where
    C: PixelColor,
    F: Font,
    A: TextAlignment,
{
    /// Creates a textbox style with transparent background.
    pub fn new(font: F, text_color: C, alignment: A) -> Self {
        Self {
            text_style: TextStyle::new(font, text_color),
            alignment,
        }
    }
}

pub struct StyledTextBox<'a, C, F, A>
where
    C: PixelColor,
    F: Font,
    A: TextAlignment,
{
    pub text_box: TextBox<'a>,
    pub style: TextBoxStyle<C, F, A>,
}
