//! Pixel iterators used for text rendering
use embedded_graphics::{prelude::*, style::TextStyle};

/// Pixel iterator to render a styled character
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct StyledCharacterIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    character: char,
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

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.char_walk.y >= F::CHARACTER_SIZE.height as i32 {
                // Done with this char, move on to the next one
                break None;
            } else {
                let color = if F::character_pixel(
                    self.character,
                    self.char_walk.x as u32,
                    self.char_walk.y as u32,
                ) {
                    self.style.text_color.or(self.style.background_color)
                } else {
                    self.style.background_color
                };

                let p = self.pos + self.char_walk;

                if self.char_walk.x < self.max_x {
                    self.char_walk.x += 1;
                } else {
                    self.char_walk.x = 0;
                    self.char_walk.y += 1;
                }

                // Skip to next point if pixel is transparent
                if let Some(color) = color {
                    break Some(Pixel(p, color));
                }
            }
        }
    }
}

/// Pixel iterator to render font spacing
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct EmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    style: TextStyle<C, F>,
    pos: Point,
    char_walk: Point,
}

impl<C, F> EmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    pub fn new(pos: Point, style: TextStyle<C, F>) -> Self {
        Self {
            style,
            pos,
            char_walk: Point::zero(),
        }
    }
}

impl<C, F> Iterator for EmptySpaceIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        if F::CHARACTER_SPACING == 0 {
            None
        } else if let Some(color) = self.style.background_color {
            if self.char_walk.y >= F::CHARACTER_SIZE.height as i32 {
                // Done with filling this space
                None
            } else {
                let p = self.pos + self.char_walk;

                if self.char_walk.x < F::CHARACTER_SPACING as i32 - 1 {
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
