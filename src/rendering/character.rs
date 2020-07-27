//! Character rendering
use crate::utils::font_ext::FontExt;
use embedded_graphics::{prelude::*, style::TextStyle};

/// Pixel iterator to render a styled character
#[derive(Clone, Debug)]
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
                self.style.text_color
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

#[cfg(test)]
mod test {
    use super::StyledCharacterIterator;
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        style::TextStyleBuilder,
    };

    #[test]
    fn transparent_char() {
        let mut display = MockDisplay::new();
        let style = TextStyleBuilder::new(Font6x8)
            .background_color(BinaryColor::On)
            .build();

        StyledCharacterIterator::new('A', Point::zero(), style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "#   ##   ",
                " ### #   ",
                " ### #   ",
                "     #   ",
                " ### #   ",
                " ### #   ",
                " ### #   ",
                "######   "
            ])
        );
    }
}
