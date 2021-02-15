//! Line rendering.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        cursor::LineCursor,
        line_iter::{LineElementParser, RenderElement},
    },
    style::{color::Rgb, height_mode::HeightMode, TextBoxStyle},
    utils::str_width,
};
use core::cell::RefCell;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    text::{CharacterStyle, TextRenderer},
    Drawable,
};

#[cfg(feature = "ansi")]
use super::ansi::Sgr;

#[derive(Debug)]
struct Refs<'a, 'b, F, A, V, H> {
    parser: &'b mut Parser<'a>,
    style: &'b mut TextBoxStyle<F, A, V, H>,
    carried_token: &'b mut Option<Token<'a>>,
}

/// Render a single line of styled text.
#[derive(Debug)]
pub struct StyledLineRenderer<'a, 'b, F, A, V, H> {
    cursor: LineCursor,
    inner: RefCell<Refs<'a, 'b, F, A, V, H>>,
}

impl<'a, 'b, F, A, V, H> StyledLineRenderer<'a, 'b, F, A, V, H>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
    <F as CharacterStyle>::Color: From<Rgb>,
    H: HeightMode,
{
    /// Creates a new line renderer.
    #[inline]
    pub fn new(
        parser: &'b mut Parser<'a>,
        cursor: LineCursor,
        style: &'b mut TextBoxStyle<F, A, V, H>,
        carried_token: &'b mut Option<Token<'a>>,
    ) -> Self {
        Self {
            cursor,
            inner: RefCell::new(Refs {
                parser,
                style,
                carried_token,
            }),
        }
    }

    #[cfg_attr(not(feature = "ansi"), allow(unused))]
    fn render_line<D>(
        display: &mut D,
        elements: impl Iterator<Item = RenderElement<'a>>,
        renderer: &RefCell<&mut F>,
        mut pos: Point,
        min_x: i32,
        max_x: i32,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = <F as CharacterStyle>::Color>,
    {
        // renderer is used in the iterator to measure text so can't borrow early
        for element in elements {
            match element {
                RenderElement::PrintedCharacters(s) => {
                    // this isn't ideal - neither the name `style` nor the fact it's in `elements`
                    pos = renderer.borrow().draw_string(s, pos, display)?;
                }

                RenderElement::Space(space_width, _) => {
                    pos = renderer
                        .borrow()
                        .draw_whitespace(space_width, pos, display)?;
                }

                #[cfg(feature = "ansi")]
                RenderElement::MoveCursor(delta) => {
                    // FIXME: use Ord::clamp if MSRV >= 1.50
                    let new_pos = Point::new((pos.x + delta).max(min_x).min(max_x), pos.y);
                    let from = if delta < 0 { new_pos } else { pos };
                    pos = new_pos;

                    // fill the space and deliberately ignore next position
                    renderer
                        .borrow()
                        .draw_whitespace(delta.abs() as u32, from, display)?;
                }

                #[cfg(feature = "ansi")]
                RenderElement::Sgr(sgr) => sgr.apply(&mut **renderer.borrow_mut()),
            }
        }

        Ok(())
    }

    #[cfg_attr(not(feature = "ansi"), allow(unused))]
    fn skip_line(elements: impl Iterator<Item = RenderElement<'a>>, renderer: &RefCell<&mut F>) {
        // renderer is used in the iterator to measure text so can't borrow early
        for element in elements {
            #[cfg(feature = "ansi")]
            if let RenderElement::Sgr(sgr) = element {
                sgr.apply(&mut **renderer.borrow_mut())
            }
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
            style,
            carried_token,
        } = &mut *inner;

        // FIXME: full copy isn't ideal
        let mut cursor = self.cursor.clone();

        let max_line_width = cursor.line_width();
        let lm = style.measure_line(
            &mut parser.clone(),
            &mut carried_token.clone(),
            max_line_width,
        );

        let (left, space_config) = A::place_line(&style.character_style, max_line_width, lm);

        cursor.advance_unchecked(left);

        let renderer = RefCell::new(&mut style.character_style);

        let elements = LineElementParser::<'_, '_, _, _, A>::new(
            parser,
            cursor.clone(),
            space_config,
            carried_token,
            |s| str_width(&**renderer.borrow(), s),
        );

        if display.bounding_box().size.height == 0 {
            Self::skip_line(elements, &renderer);
        } else {
            let pos = cursor.pos();

            let (min_x, max_x) = (pos.x, pos.x + (max_line_width - left) as i32);
            Self::render_line(display, elements, &renderer, pos, min_x, max_x)?;
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
        parser::Parser,
        rendering::{cursor::LineCursor, line::StyledLineRenderer},
        style::{color::Rgb, height_mode::HeightMode, TabSize, TextBoxStyle, TextBoxStyleBuilder},
        utils::test::size_for,
    };
    use embedded_graphics::{
        geometry::Point,
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
        let cursor = LineCursor::new(
            bounds.size.width,
            TabSize::Spaces(4).into_pixels(&style.character_style),
        );
        let mut carried = None;

        let renderer = StyledLineRenderer::new(&mut parser, cursor, &mut style, &mut carried);
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
}

#[cfg(all(test, feature = "ansi"))]
mod ansi_parser_tests {
    use crate::{
        parser::Parser,
        rendering::{cursor::LineCursor, line::StyledLineRenderer},
        style::{TabSize, TextBoxStyleBuilder},
        utils::test::size_for,
    };
    use embedded_graphics::{
        mock_display::MockDisplay,
        mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
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

        let cursor = LineCursor::new(
            size_for(Font6x9, 7, 1).width,
            TabSize::Spaces(4).into_pixels(&character_style),
        );
        let mut carried = None;
        StyledLineRenderer::new(&mut parser, cursor, &mut style, &mut carried)
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
