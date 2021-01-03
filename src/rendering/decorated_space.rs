//! Whitespace rendering modified for underlined/strikethrough text.

use crate::utils::font_ext::FontExt;
use core::{marker::PhantomData, ops::Range};
use embedded_graphics::{prelude::MonoFont, style::MonoTextStyle};
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{BinaryColor, PixelColor},
    primitives::{rectangle, PointsIter, Rectangle},
    Drawable, Pixel,
};

/// Pixel iterator to render boxes using a single color, and horizontal lines with a different one.
///
/// This struct may be used to implement custom rendering algorithms. Internally, this pixel
/// iterator is used by [`StyledLineRenderer`] to render whitespace.
///
/// [`StyledLineRenderer`]: ../line/struct.StyledLineRenderer.html
#[derive(Clone, Debug)]
pub struct Pixels<F> {
    points: rectangle::Points,
    underline: bool,
    strikethrough: bool,
    _font: PhantomData<F>,
}

impl<F> Pixels<F>
where
    F: MonoFont,
{
    /// Creates a new pixel iterator to draw empty spaces.
    #[inline]
    #[must_use]
    pub fn new(width: u32, rows: Range<i32>, underline: bool, strikethrough: bool) -> Self {
        let start = rows.start;
        let rows = rows.count() as u32;
        // adjust height if whole character is displayed for underline
        let rows = if underline { rows + 1 } else { rows };

        Self {
            points: Rectangle::new(Point::new(0, start), Size::new(width, rows)).points(),
            underline,
            strikethrough,
            _font: PhantomData,
        }
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
            let is_underline = self.underline && point.y as u32 == F::CHARACTER_SIZE.height;
            let is_strikethrough = self.strikethrough && point.y as u32 == F::strikethrough_pos();

            let color = BinaryColor::from(is_underline || is_strikethrough);
            Pixel(point, color)
        })
    }
}

/// Renderer to draw a whitespace with additional decoration.
pub struct DecoratedSpaceRenderer<C, F>
where
    C: PixelColor,
    F: MonoFont,
{
    style: MonoTextStyle<C, F>,
    pos: Point,
    width: u32,
    rows: Range<i32>,
    underline: bool,
    strikethrough: bool,
}

impl<C, F> DecoratedSpaceRenderer<C, F>
where
    C: PixelColor,
    F: MonoFont,
{
    /// Creates a new renderer.
    pub fn new(
        style: MonoTextStyle<C, F>,
        pos: Point,
        width: u32,
        rows: Range<i32>,
        underline: bool,
        strikethrough: bool,
    ) -> Self {
        Self {
            style,
            pos,
            width,
            rows,
            underline,
            strikethrough,
        }
    }

    /// Returns the pixel iterator.
    pub fn pixels(&self) -> Pixels<F> {
        Pixels::<F>::new(
            self.width,
            self.rows.clone(),
            self.underline,
            self.strikethrough,
        )
    }
}

impl<C, F> Drawable for DecoratedSpaceRenderer<C, F>
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
