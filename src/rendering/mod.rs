//! Pixel iterators used for text rendering.
#[cfg(feature = "ansi")]
pub mod ansi;
pub mod character;
pub mod cursor;
pub mod line;
pub mod line_iter;
pub mod modified_whitespace;
pub mod space_config;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{cursor::Cursor, line::StyledLinePixelIterator},
    style::{color::Rgb, height_mode::HeightMode},
    StyledTextBox,
};
use embedded_graphics::prelude::*;
use space_config::SpaceConfig;

///
pub trait RendererFactory {
    ///
    type SpaceConfig: SpaceConfig;

    ///
    fn place_line(
        max_width: u32,
        width: u32,
        n_spaces: u32,
        carried_token: Option<Token>,
    ) -> (u32, Self::SpaceConfig);
}

impl<'a, C, F, A, V, H, SP> Drawable for StyledTextBox<'a, C, F, A, V, H>
where
    C: PixelColor + From<Rgb>,
    F: MonoFont,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
    SP: SpaceConfig<Font = F>,
    Self: RendererFactory<SpaceConfig = SP>,
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

            let (left_offset, space_config) =
                Self::place_line(max_line_width, width, total_spaces, t);

            cursor.advance_unchecked(left_offset);

            let iter = StyledLinePixelIterator::new(
                &mut parser,
                &mut cursor,
                space_config,
                &mut style,
                &mut carried,
            );

            display.draw_iter(iter)?;
        }
    }
}
