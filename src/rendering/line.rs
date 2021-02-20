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
use super::space_config::UniformSpaceConfig;

/// Render a single line of styled text.
#[derive(Debug)]
pub struct StyledLineRenderer<'a, F, A, V, H> {
    cursor: LineCursor,
    state: LineRenderState<'a, F, A, V, H>,
}

#[derive(Debug, Clone)]
pub struct LineRenderState<'a, F, A, V, H> {
    pub parser: Parser<'a>,
    pub style: TextBoxStyle<F, A, V, H>,
    pub carried_token: Option<Token<'a>>,
}

impl<F, A, V, H> LineRenderState<'_, F, A, V, H> {
    pub fn is_finished(&self) -> bool {
        self.carried_token.is_none() && self.parser.is_empty()
    }
}

impl<'a, F, A, V, H> StyledLineRenderer<'a, F, A, V, H>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
    <F as CharacterStyle>::Color: From<Rgb>,
    H: HeightMode,
{
    /// Creates a new line renderer.
    #[inline]
    pub fn new(cursor: LineCursor, state: LineRenderState<'a, F, A, V, H>) -> Self {
        Self { cursor, state }
    }

    #[cfg_attr(not(feature = "ansi"), allow(unused))]
    fn render_line<D>(
        display: &mut D,
        elements: impl Iterator<Item = RenderElement<'a>>,
        renderer: &RefCell<&mut F>,
        mut pos: Point,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = <F as CharacterStyle>::Color>,
    {
        // renderer is used in the iterator to measure text so can't borrow early
        for element in elements {
            match element {
                RenderElement::PrintedCharacters(s, _) => {
                    // this isn't ideal - neither the name `style` nor the fact it's in `elements`
                    pos = renderer.borrow().draw_string(s, pos, display)?;
                }

                RenderElement::Space(space_width) => {
                    pos = renderer
                        .borrow()
                        .draw_whitespace(space_width, pos, display)?;
                }

                #[cfg(feature = "ansi")]
                RenderElement::MoveCursor(delta) => {
                    // LineElementIterator ensures this new_pos is valid.
                    let new_pos = Point::new(pos.x + delta, pos.y);
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

impl<'a, F, A, V, H> Drawable for StyledLineRenderer<'a, F, A, V, H>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle + Clone,
    <F as CharacterStyle>::Color: From<Rgb>,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Color = <F as CharacterStyle>::Color;
    type Output = LineRenderState<'a, F, A, V, H>;

    #[inline]
    fn draw<D>(&self, display: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let LineRenderState {
            mut parser,
            mut style,
            mut carried_token,
        } = self.state.clone();

        if display.bounding_box().size.height == 0 {
            // We're outside of the view - no need for a separate measure pass.
            let renderer = RefCell::new(&mut style.character_style);
            let mut elements = LineElementParser::<'_, '_, _, _, A>::new(
                &mut parser,
                self.cursor.clone(),
                UniformSpaceConfig::new(&**renderer.borrow()),
                &mut carried_token,
                |s| str_width(&**renderer.borrow(), s),
            );
            Self::skip_line(elements.iter(), &renderer);
        } else {
            // We have to resort to trickery to figure out the string that is rendered as the line.
            let mut cloned_parser = parser.clone();
            let lm = style.measure_line(
                &mut cloned_parser,
                &mut carried_token.clone(),
                self.cursor.line_width(),
            );

            let consumed_bytes = parser.as_str().len() - cloned_parser.as_str().len();
            let line_str = unsafe { parser.as_str().get_unchecked(..consumed_bytes) };

            let (left, space_config) = A::place_line(line_str, &style.character_style, lm);

            let mut cursor = self.cursor.clone();
            cursor.move_cursor(left as i32).ok();

            let renderer = RefCell::new(&mut style.character_style);
            let pos = cursor.pos();
            let mut elements = LineElementParser::<'_, '_, _, _, A>::new(
                &mut parser,
                cursor,
                space_config,
                &mut carried_token,
                |s| str_width(&**renderer.borrow(), s),
            );
            Self::render_line(display, elements.iter(), &renderer, pos)?;
        }

        Ok(LineRenderState {
            parser,
            style,
            carried_token,
        })
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
        rendering::{
            cursor::LineCursor,
            line::{LineRenderState, StyledLineRenderer},
        },
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
        style: TextBoxStyle<F, A, V, H>,
        pattern: &[&str],
    ) where
        F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle + Clone,
        <F as CharacterStyle>::Color: From<Rgb> + embedded_graphics::mock_display::ColorMapping,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        let parser = Parser::parse(text);
        let cursor = LineCursor::new(
            bounds.size.width,
            TabSize::Spaces(4).into_pixels(&style.character_style),
        );

        let state = LineRenderState {
            parser,
            style,
            carried_token: None,
        };

        let renderer = StyledLineRenderer::new(cursor, state);
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
        rendering::{
            cursor::LineCursor,
            line::{LineRenderState, StyledLineRenderer},
        },
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

        let parser = Parser::parse("foo\x1b[2Dsample");

        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .build();

        let cursor = LineCursor::new(
            size_for(Font6x9, 7, 1).width,
            TabSize::Spaces(4).into_pixels(&character_style),
        );
        let state = LineRenderState {
            parser,
            style,
            carried_token: None,
        };
        StyledLineRenderer::new(cursor, state)
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
