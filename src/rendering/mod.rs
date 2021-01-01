//! Pixel iterators used for text rendering.
#[cfg(feature = "ansi")]
mod ansi;
pub(crate) mod cursor;
mod line;
pub(crate) mod line_iter;
pub(crate) mod space_config;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::Parser,
    rendering::cursor::Cursor,
    style::{color::Rgb, height_mode::HeightMode},
    StyledTextBox,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    text::{CharacterStyle, TextRenderer},
    Drawable,
};

use self::line::StyledLineRenderer;

impl<'a, F, A, V, H> Drawable for StyledTextBox<'a, F, A, V, H>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle + Clone,
    <F as CharacterStyle>::Color: From<Rgb>,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Color = <F as CharacterStyle>::Color;

    #[inline]
    fn draw<D: DrawTarget<Color = Self::Color>>(&self, display: &mut D) -> Result<(), D::Error> {
        let mut cursor = Cursor::new(
            self.text_box.bounds,
            self.style.character_style.line_height(),
            self.style.line_spacing,
            self.style.tab_size.into_pixels(&self.style.character_style),
        );

        V::apply_vertical_alignment(&mut cursor, self);

        #[cfg(feature = "ansi")]
        let style = &mut self.style.clone();
        #[cfg(not(feature = "ansi"))]
        let style = &self.style;

        let mut carried = None;
        let mut parser = Parser::parse(self.text_box.text);

        loop {
            if carried.is_none() && parser.is_empty() {
                return Ok(());
            }

            StyledLineRenderer::new(&mut parser, &mut cursor, style, &mut carried).draw(display)?;
        }
    }
}
