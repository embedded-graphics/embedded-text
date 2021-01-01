//! Line rendering.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        cursor::Cursor,
        line_iter::{LineElementParser, RenderElement},
    },
    style::{color::Rgb, height_mode::HeightMode, TextBoxStyle},
    utils::str_width,
};
use core::{cell::RefCell, ops::Range};
use embedded_graphics::{
    draw_target::{DrawTarget, DrawTargetExt},
    geometry::{Point, Size},
    primitives::Rectangle,
    text::{CharacterStyle, TextRenderer},
    Drawable,
};

#[cfg(feature = "ansi")]
use super::ansi::Sgr;

#[derive(Debug)]
struct Refs<'a, 'b, F, A, V, H> {
    parser: &'b mut Parser<'a>,
    cursor: &'b mut Cursor,
    #[cfg(feature = "ansi")]
    style: &'b mut TextBoxStyle<F, A, V, H>,
    #[cfg(not(feature = "ansi"))]
    style: &'b TextBoxStyle<F, A, V, H>,
    carried_token: &'b mut Option<Token<'a>>,
}

/// Render a single line of styled text.
#[derive(Debug)]
pub struct StyledLineRenderer<'a, 'b, F, A, V, H> {
    display_range: Range<i32>,
    inner: RefCell<Refs<'a, 'b, F, A, V, H>>,
}

impl<'a, 'b, F, A, V, H> StyledLineRenderer<'a, 'b, F, A, V, H>
where
    F: TextRenderer,
    H: HeightMode,
{
    /// Creates a new line renderer.
    #[inline]
    pub fn new(
        parser: &'b mut Parser<'a>,
        cursor: &'b mut Cursor,
        #[cfg(feature = "ansi")] style: &'b mut TextBoxStyle<F, A, V, H>,
        #[cfg(not(feature = "ansi"))] style: &'b TextBoxStyle<F, A, V, H>,
        carried_token: &'b mut Option<Token<'a>>,
    ) -> Self {
        Self {
            display_range: H::calculate_displayed_row_range(&cursor),
            inner: RefCell::new(Refs {
                parser,
                cursor,
                style,
                carried_token,
            }),
        }
    }
}

impl<F, A, V, H> Drawable for StyledLineRenderer<'_, '_, F, A, V, H>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
    <F as CharacterStyle>::Color: From<Rgb>,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Color = <F as CharacterStyle>::Color;

    #[inline]
    fn draw<D>(&self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut inner = self.inner.borrow_mut();
        let Refs {
            parser,
            cursor,
            style,
            carried_token,
        } = &mut *inner;

        let max_line_width = cursor.line_width();
        let (width, total_spaces, t) =
            style.measure_line(&mut parser.clone(), carried_token.clone(), max_line_width);

        let (left, space_config) = A::place_line(
            &style.character_style,
            max_line_width,
            width,
            total_spaces,
            t,
        );

        cursor.advance_unchecked(left);

        let display_size = Size::new(max_line_width, self.display_range.clone().count() as u32);

        let mut pos = cursor.position;

        #[cfg(feature = "ansi")]
        let (min_x, max_x) = (
            cursor.position.x,
            cursor.position.x + cursor.line_width() as i32,
        );

        #[cfg(feature = "ansi")]
        let renderer = RefCell::new(&mut style.character_style);
        #[cfg(not(feature = "ansi"))]
        let renderer = RefCell::new(&style.character_style);

        let mut elements = LineElementParser::<'_, '_, _, _, A>::new(
            parser,
            cursor,
            space_config,
            carried_token,
            |s| str_width(&**renderer.borrow(), s),
        );

        if display_size.height == 0 {
            while let Some(element) = elements.next() {
                match element {
                    #[cfg(feature = "ansi")]
                    RenderElement::Sgr(sgr) => sgr.apply(&mut **renderer.borrow_mut()),

                    _ => {}
                }
            }
        } else {
            // FIXME: this should probably be done by H. Also, for a full range no clipping is necessary
            let mut display = display.clipped(&Rectangle::new(
                pos + Point::new(0, self.display_range.start),
                display_size,
            ));

            while let Some(element) = elements.next() {
                match element {
                    RenderElement::PrintedCharacters(s) => {
                        // this isn't ideal - neither the name `style` nor the fact it's in `elements`
                        pos = renderer.borrow().draw_string(s, pos, &mut display)?;
                    }

                    RenderElement::Space(space_width, _) => {
                        pos = renderer
                            .borrow()
                            .draw_whitespace(space_width, pos, &mut display)?;
                    }

                    #[cfg(feature = "ansi")]
                    RenderElement::MoveCursor(delta) => {
                        let new_pos_x = (pos.x + delta).max(min_x).min(max_x);
                        let from = if delta < 0 {
                            pos.y_axis() + Point::new(new_pos_x, 0)
                        } else {
                            pos
                        };
                        pos.x = new_pos_x;

                        // fill the space and deliberately ignore next position
                        renderer.borrow().draw_whitespace(
                            delta.abs() as u32,
                            from,
                            &mut display,
                        )?;
                    }

                    #[cfg(feature = "ansi")]
                    RenderElement::Sgr(sgr) => sgr.apply(&mut **renderer.borrow_mut()),
                }
            }
        }

        match carried_token {
            Some(Token::CarriageReturn) => {
                cursor.carriage_return();
            }

            _ => {
                cursor.new_line();
                cursor.carriage_return();
            }
        }

        Ok(())
    }
}

#[cfg(feature = "ansi")]
impl Sgr {
    fn apply<F>(self, renderer: &mut F)
    where
        F: CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb>,
    {
        use embedded_graphics::text::DecorationColor;
        match self {
            Sgr::Reset => {
                renderer.set_text_color(None);
                renderer.set_background_color(None);
                renderer.set_underline_color(DecorationColor::None);
                renderer.set_strikethrough_color(DecorationColor::None);
            }
            Sgr::ChangeTextColor(color) => {
                renderer.set_text_color(Some(color.into()));
            }
            Sgr::DefaultTextColor => {
                renderer.set_text_color(None);
            }
            Sgr::ChangeBackgroundColor(color) => {
                renderer.set_background_color(Some(color.into()));
            }
            Sgr::DefaultBackgroundColor => {
                renderer.set_background_color(None);
            }
            Sgr::Underline => {
                renderer.set_underline_color(DecorationColor::TextColor);
            }
            Sgr::UnderlineOff => {
                renderer.set_underline_color(DecorationColor::None);
            }
            Sgr::CrossedOut => {
                renderer.set_strikethrough_color(DecorationColor::TextColor);
            }
            Sgr::NotCrossedOut => {
                renderer.set_strikethrough_color(DecorationColor::None);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        alignment::{HorizontalTextAlignment, VerticalTextAlignment},
        parser::{Parser, Token},
        prelude::Exact,
        rendering::{cursor::Cursor, line::StyledLineRenderer},
        style::{
            color::Rgb, height_mode::HeightMode, vertical_overdraw::Hidden, TabSize, TextBoxStyle,
            TextBoxStyleBuilder,
        },
        utils::test::size_for,
    };
    use embedded_graphics::{
        geometry::{Point, Size},
        mock_display::MockDisplay,
        mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        primitives::Rectangle,
        text::{CharacterStyle, TextRenderer},
        Drawable,
    };

    fn test_rendered_text<'a, F, A, V, H>(
        text: &'a str,
        bounds: Rectangle,
        mut style: TextBoxStyle<F, A, V, H>,
        pattern: &[&str],
    ) where
        F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb> + embedded_graphics::mock_display::ColorMapping,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        let mut parser = Parser::parse(text);
        let mut cursor = Cursor::new(
            bounds,
            style.character_style.line_height(),
            style.line_spacing,
            TabSize::Spaces(4).into_pixels(&style.character_style),
        );
        let mut carried = None;

        let renderer = StyledLineRenderer::new(&mut parser, &mut cursor, &mut style, &mut carried);
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        renderer.draw(&mut display).unwrap();

        display.assert_pattern(pattern);
    }

    #[test]
    fn simple_render() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .build();

        test_rendered_text(
            " Some sample text",
            Rectangle::new(Point::zero(), size_for(Font6x9, 7, 1)),
            style,
            &[
                "..............................",
                "........##....................",
                ".......#..#...................",
                "........#.....##..##.#....##..",
                ".........#...#..#.#.#.#..#.##.",
                ".......#..#..#..#.#.#.#..##...",
                "........##....##..#...#...###.",
                "..............................",
                "..............................",
            ],
        );
    }

    #[test]
    fn render_before_area() {
        let mut parser = Parser::parse(" Some sample text");

        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .build();

        let mut cursor = Cursor::new(
            Rectangle::new(Point::new(0, 8), size_for(Font6x9, 7, 2)),
            style.character_style.line_height(),
            style.line_spacing,
            TabSize::Spaces(4).into_pixels(&style.character_style),
        );
        cursor.position.y -= 8;

        let mut carried = None;
        let renderer = StyledLineRenderer::new(&mut parser, &mut cursor, &mut style, &mut carried);

        let mut display = MockDisplay::new();

        renderer.draw(&mut display).unwrap();

        // Nothing is drawn and we don't get a panic either.
        display.assert_pattern(&[]);

        // even though nothing was drawn, the text should be consumed
        assert_eq!(Some(Token::Break(None)), carried);
    }

    #[test]
    fn simple_render_nbsp() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .build();

        test_rendered_text(
            "Some\u{A0}sample text",
            Rectangle::new(Point::zero(), size_for(Font6x9, 7, 1)),
            style,
            &[
                "..........................................",
                "..##......................................",
                ".#..#.....................................",
                "..#.....##..##.#....##..........###...###.",
                "...#...#..#.#.#.#..#.##........##....#..#.",
                ".#..#..#..#.#.#.#..##............##..#..#.",
                "..##....##..#...#...###........###....###.",
                "..........................................",
                "..........................................",
            ],
        );
    }

    #[test]
    fn simple_render_first_word_not_wrapped() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .build();

        test_rendered_text(
            "Some sample text",
            Rectangle::new(Point::zero(), size_for(Font6x9, 2, 1)),
            style,
            &[
                "............",
                "..##........",
                ".#..#.......",
                "..#.....##..",
                "...#...#..#.",
                ".#..#..#..#.",
                "..##....##..",
                "............",
                "............",
            ],
        );
    }

    #[test]
    fn newline_stops_render() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .build();

        test_rendered_text(
            "Some \nsample text",
            Rectangle::new(Point::zero(), size_for(Font6x9, 7, 1)),
            style,
            &[
                "..............................",
                "..##..........................",
                ".#..#.........................",
                "..#.....##..##.#....##........",
                "...#...#..#.#.#.#..#.##.......",
                ".#..#..#..#.#.#.#..##.........",
                "..##....##..#...#...###.......",
                "..............................",
                "..............................",
            ],
        );
    }

    #[test]
    fn underline_just_inside_of_textbox() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .underline()
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .build();

        test_rendered_text(
            "s",
            Rectangle::new(Point::zero(), size_for(Font6x9, 1, 1)),
            style,
            &[
                "......             ",
                "......             ",
                "......             ",
                "..###.             ",
                ".##...             ",
                "...##.             ",
                ".###..             ",
                "......             ",
                "######             ",
            ],
        );
    }

    #[test]
    fn underline_outside_of_textbox() {
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .underline()
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .height_mode(Exact(Hidden))
            .build();

        test_rendered_text(
            "s",
            Rectangle::new(Point::zero(), size_for(Font6x9, 1, 1) - Size::new(0, 1)),
            style,
            &[
                "......             ",
                "......             ",
                "......             ",
                "..###.             ",
                ".##...             ",
                "...##.             ",
                ".###..             ",
                "......             ",
            ],
        );
    }
}

#[cfg(all(test, feature = "ansi"))]
mod ansi_parser_tests {
    use crate::{
        parser::Parser,
        rendering::{cursor::Cursor, line::StyledLineRenderer},
        style::{TabSize, TextBoxStyleBuilder},
        utils::test::size_for,
    };
    use embedded_graphics::{
        geometry::Point,
        mock_display::MockDisplay,
        mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        primitives::Rectangle,
        text::TextRenderer,
        Drawable,
    };

    #[test]
    fn ansi_cursor_backwards() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let mut parser = Parser::parse("foo\x1b[2Dsample");

        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .build();

        let mut cursor = Cursor::new(
            Rectangle::new(Point::zero(), size_for(Font6x9, 7, 1)),
            character_style.line_height(),
            0,
            TabSize::Spaces(4).into_pixels(&character_style),
        );
        let mut carried = None;
        StyledLineRenderer::new(&mut parser, &mut cursor, &mut style, &mut carried)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "..........................................",
            "...#...........................##.........",
            "..#.#...........................#.........",
            "..#.....###...###.##.#...###....#.....##..",
            ".###...##....#..#.#.#.#..#..#...#....#.##.",
            "..#......##..#..#.#.#.#..#..#...#....##...",
            "..#....###....###.#...#..###...###....###.",
            ".........................#................",
            ".........................#................",
        ]);
    }
}
