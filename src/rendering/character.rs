//! Character rendering.
use crate::utils::font_ext::FontExt;
use core::{marker::PhantomData, ops::Range};
use embedded_graphics::{prelude::*, style::MonoTextStyle};
use embedded_graphics_core::{
    pixelcolor::BinaryColor,
    primitives::{rectangle, Rectangle},
};

/// Pixel iterator to render a single styled character.
///
/// This struct may be used to implement custom rendering algorithms. Internally, this pixel
/// iterator is used by [`StyledLinePixelIterator`] to render characters.
///
/// [`StyledLinePixelIterator`]: ../line/struct.StyledLinePixelIterator.html
#[derive(Clone, Debug)]
pub struct Pixels<F> {
    points: rectangle::Points,

    underline: bool,
    strikethrough: bool,

    char_px_offset: u32,
    byte_index: usize,
    bit_mask: u8,

    _font: PhantomData<F>,
}

impl<F> Pixels<F>
where
    F: MonoFont,
{
    /// Creates a new pixel iterator to draw the given character.
    #[inline]
    #[must_use]
    pub fn new(character: char, rows: Range<i32>, underline: bool, strikethrough: bool) -> Self {
        let start = rows.start;
        let rows = rows.count() as u32;
        // adjust height if whole character is displayed for underline
        let rows = if underline { rows + 1 } else { rows };

        // Char _code_ offset from first char, most often a space
        // E.g. first char = ' ' (32), target char = '!' (33), offset = 33 - 32 = 1
        let char_offset = F::char_offset(character);

        // Top left corner of character, in pixels.
        let char_per_row = F::FONT_IMAGE_WIDTH / F::CHARACTER_SIZE.width;
        let char_x = char_offset % char_per_row * F::CHARACTER_SIZE.width;
        let char_y = char_offset / char_per_row * F::CHARACTER_SIZE.height;

        Self {
            points: Rectangle::new(
                Point::new(0, start),
                Size::new(F::CHARACTER_SIZE.width, rows),
            )
            .points(),
            underline,
            strikethrough,
            char_px_offset: char_x + char_y * F::FONT_IMAGE_WIDTH,
            byte_index: 0,
            bit_mask: 0,
            _font: PhantomData,
        }
    }

    fn start_row(&mut self, y: i32) {
        let index = self.char_px_offset + y as u32 * F::FONT_IMAGE_WIDTH;

        self.byte_index = (index / 8) as usize;
        self.bit_mask = 0x80 >> (index % 8);
    }

    fn next_point(&mut self, pos: Point) -> BinaryColor {
        let is_underline = self.underline && pos.y as u32 == F::CHARACTER_SIZE.height;
        let is_strikethrough = self.strikethrough && pos.y as u32 == F::strikethrough_pos();

        let color = BinaryColor::from(
            is_underline
                || is_strikethrough
                || (F::FONT_IMAGE[self.byte_index] & self.bit_mask != 0),
        );

        if self.bit_mask != 0x01 {
            self.bit_mask >>= 1;
        } else {
            self.bit_mask = 0x80;
            self.byte_index += 1;
        }

        color
    }
}

impl<F> Iterator for Pixels<F>
where
    F: MonoFont,
{
    type Item = Pixel<BinaryColor>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.points.next().map(|point| {
            if point.x == 0 {
                self.start_row(point.y);
            }
            Pixel(point, self.next_point(point))
        })
    }
}

/// Renderer to draw a character with additional decoration.
pub struct GlyphRenderer<C, F>
where
    C: PixelColor,
    F: MonoFont,
{
    character: char,
    pos: Point,
    rows: Range<i32>,

    style: MonoTextStyle<C, F>,
    underline: bool,
    strikethrough: bool,
}

impl<C, F> GlyphRenderer<C, F>
where
    C: PixelColor,
    F: MonoFont,
{
    /// Creates a new renderer.
    pub fn new(
        character: char,
        style: MonoTextStyle<C, F>,
        pos: Point,
        rows: Range<i32>,
        underline: bool,
        strikethrough: bool,
    ) -> Self {
        Self {
            character,
            style,
            pos,
            rows,
            underline,
            strikethrough,
        }
    }

    /// Returns the pixel iterator.
    pub fn pixels(&self) -> Pixels<F> {
        Pixels::<F>::new(
            self.character,
            self.rows.clone(),
            self.underline,
            self.strikethrough,
        )
    }
}

impl<C, F> Drawable for GlyphRenderer<C, F>
where
    C: PixelColor,
    F: MonoFont,
{
    type Color = C;

    fn draw<D>(&self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        display.draw_iter(self.pixels().flat_map(|Pixel(point, color)| {
            let color = match color {
                BinaryColor::Off => self.style.background_color,
                BinaryColor::On => self.style.text_color,
            };
            color.map(|c| Pixel(self.pos + point, c))
        }))
    }
}

#[cfg(test)]
mod test {
    use super::Pixels;
    use embedded_graphics::{fonts::Font6x8, mock_display::MockDisplay, prelude::*};

    #[test]
    fn draw_char() {
        let mut display = MockDisplay::new();

        Pixels::<Font6x8>::new('A', 0..Font6x8::CHARACTER_SIZE.height as i32, false, false)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            ".###..    ",
            "#...#.    ",
            "#...#.    ",
            "#####.    ",
            "#...#.    ",
            "#...#.    ",
            "#...#.    ",
            "......    ",
        ]);
    }

    #[test]
    fn strikethrough() {
        let mut display = MockDisplay::new();

        Pixels::<Font6x8>::new('A', 0..Font6x8::CHARACTER_SIZE.height as i32, false, true)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            ".###..    ",
            "#...#.    ",
            "#...#.    ",
            "#####.    ",
            "######    ",
            "#...#.    ",
            "#...#.    ",
            "......    ",
        ]);
    }

    #[test]
    fn underline() {
        let mut display = MockDisplay::new();

        Pixels::<Font6x8>::new('A', 0..Font6x8::CHARACTER_SIZE.height as i32, true, false)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            ".###..    ",
            "#...#.    ",
            "#...#.    ",
            "#####.    ",
            "#...#.    ",
            "#...#.    ",
            "#...#.    ",
            "......    ",
            "######    ",
        ]);
    }

    #[test]
    fn partial_draw() {
        let mut display = MockDisplay::new();

        Pixels::<Font6x8>::new(
            'A',
            2..Font6x8::CHARACTER_SIZE.height as i32 - 2,
            false,
            false,
        )
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "         ",
            "         ",
            "#...#.   ",
            "#####.   ",
            "#...#.   ",
            "#...#.   ",
            "         ",
            "         ",
        ]);
    }
}
