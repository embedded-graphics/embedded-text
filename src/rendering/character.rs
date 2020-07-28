//! Character rendering
use core::marker::PhantomData;
use embedded_graphics::{prelude::*, style::TextStyle};

/// Represents a glyph to be drawn
#[derive(Copy, Clone, Debug)]
pub struct Glyph<F: Font> {
    _font: PhantomData<F>,
    char_offset: u32,
}

impl<F> Glyph<F>
where
    F: Font,
{
    /// Creates a glyph from a character
    #[inline]
    pub fn new(c: char) -> Self {
        Self {
            _font: PhantomData,
            char_offset: F::char_offset(c),
        }
    }

    /// Returns the value (foreground/background) of a given point
    #[inline]
    #[must_use]
    pub fn point(&self, p: Point) -> bool {
        let char_per_row = F::FONT_IMAGE_WIDTH / F::CHARACTER_SIZE.width;

        let row = self.char_offset / char_per_row;

        // Top left corner of character, in pixels
        let char_x = (self.char_offset - (row * char_per_row)) * F::CHARACTER_SIZE.width;
        let char_y = row * F::CHARACTER_SIZE.height;

        // Bit index
        // = X pixel offset for char
        // + Character row offset (row 0 = 0, row 1 = (192 * 8) = 1536)
        // + X offset for the pixel block that comprises this char
        // + Y offset for pixel block
        let bitmap_bit_index = char_x + p.x as u32 + ((char_y + p.y as u32) * F::FONT_IMAGE_WIDTH);

        let bitmap_byte = bitmap_bit_index / 8;
        let bitmap_bit = 7 - (bitmap_bit_index % 8);

        F::FONT_IMAGE[bitmap_byte as usize] & (1 << bitmap_bit) != 0
    }
}

/// Pixel iterator to render a styled character
#[derive(Clone, Debug)]
pub struct StyledCharacterIterator<C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    character: Glyph<F>,
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
            character: Glyph::new(character),
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

            let color = if self.character.point(pos) {
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
