//! Pixel iterators used for text rendering.

pub(crate) mod cursor;
pub(crate) mod line;
pub(crate) mod line_iter;
pub(crate) mod space_config;

use crate::{
    parser::Parser,
    plugin::{PluginMarker as Plugin, ProcessingState},
    rendering::{
        cursor::Cursor,
        line::{LineRenderState, StyledLineRenderer},
    },
    style::TextBoxStyle,
    TextBox,
};
use az::SaturatingAs;
use embedded_graphics::{
    draw_target::{DrawTarget, DrawTargetExt},
    prelude::{Dimensions, Point, Size},
    primitives::Rectangle,
    text::renderer::{CharacterStyle, TextRenderer},
    Drawable,
};
use line_iter::LineEndType;

/// Text box properties.
///
/// This struct holds information about the text box.
#[derive(Clone)]
pub struct TextBoxProperties<'a, S> {
    /// The used text box style.
    pub box_style: &'a TextBoxStyle,

    /// The character style.
    pub char_style: &'a S,

    /// The height of the text.
    pub text_height: i32,

    /// The bounds of the text box.
    pub bounding_box: Rectangle,
}

impl<'a, F, M> Drawable for TextBox<'a, F, M>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
    M: Plugin<'a, <F as TextRenderer>::Color> + Plugin<'a, <F as CharacterStyle>::Color>,
    <F as CharacterStyle>::Color: Default,
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

        let text_height = self
            .style
            .measure_text_height_impl(
                self.plugin.clone(),
                &self.character_style,
                self.text,
                cursor.line_width(),
            )
            .saturating_as::<i32>();

        let box_height = self.bounding_box().size.height.saturating_as::<i32>();

        self.style.vertical_alignment.apply_vertical_alignment(
            &mut cursor,
            text_height,
            box_height,
        );

        cursor.y += self.vertical_offset;

        let props = TextBoxProperties {
            box_style: &self.style,
            char_style: &self.character_style,
            text_height,
            bounding_box: self.bounding_box(),
        };

        self.plugin.on_start_render(&mut cursor, props);

        let mut state = LineRenderState {
            text_renderer: self.character_style.clone(),
            parser: Parser::parse(self.text),
            end_type: LineEndType::EndOfText,
            plugin: &self.plugin,
        };

        state.plugin.set_state(ProcessingState::Render);

        let mut anything_drawn = false;
        loop {
            state.plugin.new_line();

            let display_range = self
                .style
                .height_mode
                .calculate_displayed_row_range(&cursor);
            let display_range_start = display_range.start.saturating_as::<i32>();
            let display_range_count = display_range.count() as u32;
            let display_size = Size::new(cursor.line_width(), display_range_count);

            let line_start = cursor.line_start();

            // FIXME: cropping isn't necessary for whole lines, but make sure not to blow up the
            // binary size as well. We could also use a different way to consume invisible text.
            let mut display = display.clipped(&Rectangle::new(
                line_start + Point::new(0, display_range_start),
                display_size,
            ));
            if display_range_count == 0 {
                // Display range can be empty if we are above, or below the visible text section
                if anything_drawn {
                    // We are below, so we won't be drawing anything else
                    let remaining_bytes = state.parser.as_str().len();
                    let consumed_bytes = self.text.len() - remaining_bytes;

                    state.plugin.post_render(
                        &mut display,
                        &self.character_style,
                        None,
                        Rectangle::new(line_start, Size::new(0, cursor.line_height())),
                    )?;
                    state.plugin.on_rendering_finished();
                    return Ok(self.text.get(consumed_bytes..).unwrap());
                }
            } else {
                anything_drawn = true;
            }

            StyledLineRenderer {
                cursor: cursor.line(),
                state: &mut state,
                style: &self.style,
            }
            .draw(&mut display)?;

            match state.end_type {
                LineEndType::EndOfText => {
                    state.plugin.on_rendering_finished();
                    break;
                }
                LineEndType::CarriageReturn => {}
                _ => {
                    cursor.new_line();

                    if state.end_type == LineEndType::NewLine {
                        cursor.y += self.style.paragraph_spacing.saturating_as::<i32>();
                    }
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
        mono_font::{
            ascii::{FONT_6X10, FONT_6X9},
            MonoTextStyleBuilder,
        },
        pixelcolor::BinaryColor,
        prelude::*,
        primitives::Rectangle,
        text::renderer::TextRenderer,
    };

    use crate::{
        alignment::HorizontalAlignment,
        style::{HeightMode, TextBoxStyle, TextBoxStyleBuilder, VerticalOverdraw},
        utils::test::{size_for, TestFont},
        TextBox,
    };

    #[track_caller]
    pub fn assert_rendered(
        alignment: HorizontalAlignment,
        text: &str,
        size: Size,
        pattern: &[&str],
    ) {
        assert_styled_rendered(
            TextBoxStyleBuilder::new().alignment(alignment).build(),
            text,
            size,
            pattern,
        );
    }

    #[track_caller]
    pub fn assert_styled_rendered(style: TextBoxStyle, text: &str, size: Size, pattern: &[&str]) {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

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

    #[test]
    fn rendering_not_stopped_prematurely() {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::with_textbox_style(
            "hello\nbuggy\nworld",
            Rectangle::new(Point::zero(), size_for(&FONT_6X10, 5, 3)),
            character_style,
            TextBoxStyleBuilder::new()
                .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
                .build(),
        )
        .set_vertical_offset(-20)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "..............................",
            "...................##.......#.",
            "....................#.......#.",
            "#...#..###..#.##....#....##.#.",
            "#...#.#...#.##..#...#...#..##.",
            "#.#.#.#...#.#.......#...#...#.",
            "#.#.#.#...#.#.......#...#..##.",
            ".#.#...###..#......###...##.#.",
            "..............................",
            "..............................",
        ]);
    }

    #[test]
    fn space_wrapping_issue() {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::with_textbox_style(
            "Hello,      s",
            Rectangle::new(Point::zero(), size_for(&FONT_6X10, 10, 2)),
            character_style,
            TextBoxStyleBuilder::new()
                .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
                .trailing_spaces(true)
                .build(),
        )
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "............................................................",
            "#...#........##....##.......................................",
            "#...#.........#.....#.......................................",
            "#...#..###....#.....#....###................................",
            "#####.#...#...#.....#...#...#...............................",
            "#...#.#####...#.....#...#...#...............................",
            "#...#.#.......#.....#...#...#...##..........................",
            "#...#..###...###...###...###....#...........................",
            "...............................#............................",
            "............................................................",
            "............                                                ",
            "............                                                ",
            "............                                                ",
            ".......###..                                                ",
            "......#.....                                                ",
            ".......###..                                                ",
            "..........#.                                                ",
            "......####..                                                ",
            "............                                                ",
            "............                                                ",
        ]);
    }

    #[test]
    fn rendering_justified_text_with_negative_left_side_bearing() {
        let mut display: MockDisplay<BinaryColor> = MockDisplay::new();
        display.set_allow_overdraw(true);

        let text = "j000 0 j00 00j00 0";
        let character_style = TestFont::new(BinaryColor::On, BinaryColor::Off);
        let size = Size::new(50, 0);

        TextBox::with_textbox_style(
            text,
            Rectangle::new(Point::zero(), size),
            character_style,
            TextBoxStyleBuilder::new()
                .alignment(HorizontalAlignment::Justified)
                .height_mode(HeightMode::FitToText)
                .build(),
        )
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "..#.####.####.####.........####........#.####.####",
            "....#..#.#..#.#..#.........#..#..........#..#.#..#",
            "..#.#..#.#..#.#..#.........#..#........#.#..#.#..#",
            "..#.#..#.#..#.#..#.........#..#........#.#..#.#..#",
            "..#.#..#.#..#.#..#.........#..#........#.#..#.#..#",
            "..#.#..#.#..#.#..#.........#..#........#.#..#.#..#",
            "..#.####.####.####.........####........#.####.####",
            "..#....................................#..........",
            "..#....................................#..........",
            "##...................................##...........",
            "####.####.#.####.####....####                     ",
            "#..#.#..#...#..#.#..#....#..#                     ",
            "#..#.#..#.#.#..#.#..#....#..#                     ",
            "#..#.#..#.#.#..#.#..#....#..#                     ",
            "#..#.#..#.#.#..#.#..#....#..#                     ",
            "#..#.#..#.#.#..#.#..#....#..#                     ",
            "####.####.#.####.####....####                     ",
            "..........#..................                     ",
            "..........#..................                     ",
            "........##...................                     ",
        ]);
    }
}
