//! Pixel iterators used for text rendering.
#[cfg(feature = "ansi")]
pub mod ansi;
pub mod character;
pub mod cursor;
pub mod decorated_space;
pub mod line;
pub mod line_iter;
pub mod space_config;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::Parser,
    rendering::cursor::Cursor,
    style::{color::Rgb, height_mode::HeightMode},
    StyledTextBox,
};
use embedded_graphics::prelude::*;

use self::line::StyledLineRenderer;

impl<'a, C, F, A, V, H> Drawable for StyledTextBox<'a, C, F, A, V, H>
where
    C: PixelColor + From<Rgb>,
    F: MonoFont,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Color = C;

    #[inline]
    fn draw<D: DrawTarget<Color = C>>(&self, display: &mut D) -> Result<(), D::Error> {
        let mut cursor = Cursor::new(self.text_box.bounds, self.style.line_spacing);

        V::apply_vertical_alignment(&mut cursor, self);

        let mut style = self.style;
        let mut carried = None;
        let mut parser = Parser::parse(self.text_box.text);

        loop {
            if carried.is_none() && parser.is_empty() {
                return Ok(());
            }

            let max_line_width = cursor.line_width();
            let (width, total_spaces, t, _) =
                style.measure_line(&mut parser.clone(), carried.clone(), max_line_width);

            let (left, space_config) = A::place_line::<F>(max_line_width, width, total_spaces, t);

            cursor.advance_unchecked(left);

            StyledLineRenderer::new(
                &mut parser,
                &mut cursor,
                &mut style,
                &mut carried,
                space_config,
            )
            .draw(display)?;
        }
    }
}
