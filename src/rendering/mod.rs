//! Pixel iterators used for text rendering.
#[cfg(feature = "ansi")]
mod ansi;
pub(crate) mod cursor;
mod line;
pub(crate) mod line_iter;
pub(crate) mod space_config;

use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        cursor::Cursor,
        line::{LineRenderState, StyledLineRenderer},
    },
    style::{color::Rgb, height_mode::HeightMode},
    TextBox,
};
use embedded_graphics::{
    draw_target::{DrawTarget, DrawTargetExt},
    prelude::{Point, Size},
    primitives::Rectangle,
    text::renderer::{CharacterStyle, TextRenderer},
    Drawable,
};

impl<'a, F, A, V, H> Drawable for TextBox<'a, F, A, V, H>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle + Clone,
    <F as CharacterStyle>::Color: From<Rgb>,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Color = <F as CharacterStyle>::Color;
    type Output = &'a str;

    #[inline]
    fn draw<D: DrawTarget<Color = Self::Color>>(
        &self,
        display: &mut D,
    ) -> Result<&'a str, D::Error> {
        let mut cursor = Cursor::new(
            self.bounds,
            self.character_style.line_height(),
            self.style.line_spacing,
            self.style.tab_size.into_pixels(&self.character_style),
        );

        V::apply_vertical_alignment(&mut cursor, self);

        let mut state = LineRenderState {
            style: self.style.clone(),
            character_style: self.character_style.clone(),
            parser: Parser::parse(self.text),
            carried_token: None,
        };

        let mut anything_drawn = false;
        while !state.is_finished() {
            let line_cursor = cursor.line();
            let display_range = H::calculate_displayed_row_range(&cursor);
            let display_size = Size::new(cursor.line_width(), display_range.clone().count() as u32);

            if display_range.start == display_range.end {
                if anything_drawn {
                    let carried_bytes = if let Some(Token::Word(word)) = state.carried_token {
                        word.len()
                    } else {
                        0
                    };

                    let remaining_bytes = state.parser.as_str().len();
                    let consumed_bytes = self.text.len() - remaining_bytes - carried_bytes;
                    return Ok(self.text.get(consumed_bytes..).unwrap());
                }
            } else {
                anything_drawn = true;
            }

            // FIXME: cropping isn't necessary for whole lines, but make sure not to blow up the
            // binary size as well.
            let mut display = display.clipped(&Rectangle::new(
                line_cursor.pos() + Point::new(0, display_range.start),
                display_size,
            ));
            state = StyledLineRenderer::new(line_cursor, state).draw(&mut display)?;

            if state.carried_token != Some(Token::CarriageReturn) {
                cursor.new_line();
            }
        }

        Ok("")
    }
}

#[cfg(test)]
pub mod test {
    use embedded_graphics::{
        mock_display::MockDisplay,
        mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        prelude::*,
        primitives::Rectangle,
    };

    use crate::{
        alignment::{HorizontalTextAlignment, LeftAligned},
        style::TextBoxStyleBuilder,
        utils::test::size_for,
        TextBox,
    };

    pub fn assert_rendered<A: HorizontalTextAlignment>(
        alignment: A,
        text: &str,
        size: Size,
        pattern: &[&str],
    ) {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new().alignment(alignment).build();

        TextBox::with_textbox_style(
            text,
            Rectangle::new(Point::zero(), size),
            character_style,
            style,
        )
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(pattern);
    }

    #[test]
    fn nbsp_doesnt_break() {
        assert_rendered(
            LeftAligned,
            "a b c\u{a0}d e f",
            size_for(&FONT_6X9, 5, 3),
            &[
                "..................            ",
                ".............#....            ",
                ".............#....            ",
                "..###........###..            ",
                ".#..#........#..#.            ",
                ".#..#........#..#.            ",
                "..###........###..            ",
                "..................            ",
                "..................            ",
                "..............................",
                "................#.............",
                "................#.............",
                "..###.........###.........##..",
                ".#...........#..#........#.##.",
                ".#...........#..#........##...",
                "..###.........###.........###.",
                "..............................",
                "..............................",
                "......                        ",
                "...#..                        ",
                "..#.#.                        ",
                "..#...                        ",
                ".###..                        ",
                "..#...                        ",
                "..#...                        ",
                "......                        ",
                "......                        ",
            ],
        );
    }
}
