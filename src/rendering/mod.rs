//! Pixel iterators used for text rendering

/// Cursor to track rendering position
pub mod cursor;

use crate::{
    alignment::TextAlignment,
    parser::Parser,
    style::{StyledTextBox, TextBoxStyle},
    utils::font_ext::FontExt,
};
use core::marker::PhantomData;
use cursor::Cursor;
use embedded_graphics::{prelude::*, style::TextStyle};

/// Pixel iterator to render a styled character
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct StyledCharacterIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// The character to draw.
    pub character: char,
    style: TextStyle<C, F>,
    pos: Point,
    char_walk: Point,
    max_x: i32,
}

impl<C, F> StyledCharacterIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Creates a new pixel iterator to draw the given character.
    #[inline]
    #[must_use]
    pub fn new(character: char, pos: Point, style: TextStyle<C, F>) -> Self {
        Self {
            character,
            style,
            pos,
            char_walk: Point::zero(),
            max_x: F::char_width(character) as i32 - 1,
        }
    }
}

impl<C, F> Iterator for StyledCharacterIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.char_walk.y >= F::CHARACTER_SIZE.height as i32 {
                // Done with this char, move on to the next one
                break None;
            }
            let pos = self.char_walk;

            if pos.x < self.max_x {
                self.char_walk.x += 1;
            } else {
                self.char_walk.x = 0;
                self.char_walk.y += 1;
            }

            let color = if F::character_point(self.character, pos) {
                self.style.text_color.or(self.style.background_color)
            } else {
                self.style.background_color
            };

            // Skip to next point if pixel is transparent
            if let Some(color) = color {
                let p = self.pos + pos;
                break Some(Pixel(p, color));
            }
        }
    }
}

/// Pixel iterator to render font spacing
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct EmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    _font: PhantomData<F>,
    color: Option<C>,
    pos: Point,
    char_walk: Point,
    walk_max_x: i32,
}

impl<C, F> EmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Creates a new pixel iterator to draw empty spaces.
    #[inline]
    #[must_use]
    pub fn new(pos: Point, width: u32, style: TextStyle<C, F>) -> Self {
        Self {
            _font: PhantomData,
            color: style.background_color,
            pos,
            char_walk: Point::zero(),
            walk_max_x: width as i32 - 1,
        }
    }
}

impl<C, F> Iterator for EmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(color) = self.color {
            if self.walk_max_x < 0 || self.char_walk.y >= F::CHARACTER_SIZE.height as i32 {
                // Done with filling this space
                None
            } else {
                let p = self.pos + self.char_walk;

                if self.char_walk.x < self.walk_max_x {
                    self.char_walk.x += 1;
                } else {
                    self.char_walk.x = 0;
                    self.char_walk.y += 1;
                }

                // Skip to next point if pixel is transparent
                Some(Pixel(p, color))
            }
        } else {
            None
        }
    }
}

/// This trait is used to associate a state type to a horizontal alignment option.
pub trait StateFactory {
    /// The type of the state variable used for rendering.
    type PixelIteratorState;

    /// Creates a new state variable.
    fn create_state() -> Self::PixelIteratorState;
}

/// Pixel iterator for styled text.
pub struct StyledTextBoxIterator<'a, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
    StyledTextBox<'a, C, F, A>: StateFactory,
{
    /// Parser to process the text during rendering
    pub parser: Parser<'a>,

    /// Style used for rendering
    pub style: TextBoxStyle<C, F, A>,

    /// Position information
    pub cursor: Cursor<F>,

    /// State information used by the rendering algorithms
    pub state: <StyledTextBox<'a, C, F, A> as StateFactory>::PixelIteratorState,
}

impl<'a, C, F, A> StyledTextBoxIterator<'a, C, F, A>
where
    C: PixelColor,
    F: Font + Copy,
    A: TextAlignment,
    StyledTextBox<'a, C, F, A>: StateFactory,
{
    /// Creates a new pixel iterator to render the styled [`TextBox`]
    #[inline]
    #[must_use]
    pub fn new(styled: &'a StyledTextBox<'a, C, F, A>) -> Self {
        Self {
            parser: Parser::parse(styled.text_box.text),
            style: styled.style,
            cursor: Cursor::new(styled.text_box.bounds),
            state: <StyledTextBox<'a, C, F, A> as StateFactory>::create_state(),
        }
    }
}
