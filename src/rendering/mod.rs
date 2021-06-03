//! Pixel iterators used for text rendering.
#[cfg(feature = "ansi")]
mod ansi;
pub(crate) mod cursor;
mod line;
pub(crate) mod line_iter;
pub(crate) mod space_config;

use crate::{
    parser::{Parser, Token},
    rendering::{
        cursor::Cursor,
        line::{LineRenderState, StyledLineRenderer},
    },
    TextBox,
};
use az::SaturatingAs;
use embedded_graphics::{
    draw_target::{DrawTarget, DrawTargetExt},
    pixelcolor::Rgb888,
    prelude::{Point, Size},
    primitives::Rectangle,
    text::renderer::{CharacterStyle, TextRenderer},
    Drawable,
};

impl<'a, F> Drawable for TextBox<'a, F>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
    <F as CharacterStyle>::Color: From<Rgb888>,
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
            self.style.line_height,
            self.style.tab_size.into_pixels(&self.character_style),
        );

        self.style
            .vertical_alignment
            .apply_vertical_alignment(&mut cursor, self);

        let mut state = LineRenderState {
            style: self.style,
            character_style: self.character_style.clone(),
            parser: Parser::parse(self.text),
            carried_token: None,
        };

        cursor.y += self.vertical_offset;

        let mut anything_drawn = false;
        while !state.is_finished() {
            let line_cursor = cursor.line();
            let display_range = self
                .style
                .height_mode
                .calculate_displayed_row_range(&cursor);
            let display_size = Size::new(
                cursor.line_width(),
                display_range.clone().count().saturating_as(),
            );

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

                if state.carried_token == Some(Token::NewLine) {
                    cursor.y += self.style.paragraph_spacing.saturating_as::<i32>();
                }
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
        alignment::HorizontalAlignment,
        style::{HeightMode, TextBoxStyleBuilder, VerticalOverdraw},
        utils::test::size_for,
        TextBox,
    };

    pub fn assert_rendered(
        alignment: HorizontalAlignment,
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
            HorizontalAlignment::Left,
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

    #[test]
    fn vertical_offset() {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "hello",
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, 5, 3)),
            character_style,
        )
        .set_vertical_offset(6)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "..............................",
            ".#...........##....##.........",
            ".#............#.....#.........",
            ".###....##....#.....#.....##..",
            ".#..#..#.##...#.....#....#..#.",
            ".#..#..##.....#.....#....#..#.",
            ".#..#...###..###...###....##..",
            "..............................",
            "..............................",
        ]);
    }

    #[test]
    fn vertical_offset_negative() {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::with_textbox_style(
            "hello",
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, 5, 3)),
            character_style,
            TextBoxStyleBuilder::new()
                .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
                .build(),
        )
        .set_vertical_offset(-4)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            ".#..#..#.##...#.....#....#..#.",
            ".#..#..##.....#.....#....#..#.",
            ".#..#...###..###...###....##..",
            "..............................",
            "..............................",
        ]);
    }
}
