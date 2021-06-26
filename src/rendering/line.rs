//! Line rendering.
use core::convert::Infallible;

use crate::{
    middleware::{Middleware, MiddlewareWrapper, ProcessingState},
    parser::Parser,
    rendering::{
        cursor::LineCursor,
        line_iter::{LineElementParser, LineEndType},
    },
    style::TextBoxStyle,
    utils::str_width,
};
use az::SaturatingAs;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::Rgb888,
    prelude::Size,
    primitives::Rectangle,
    text::{
        renderer::{CharacterStyle, TextRenderer},
        Baseline,
    },
    Drawable,
};

#[cfg(feature = "ansi")]
use super::ansi::Sgr;
use super::{line_iter::ElementHandler, space_config::SpaceConfig};

/// Render a single line of styled text.
pub(crate) struct StyledLineRenderer<'a, S, M>
where
    S: TextRenderer + Clone,
    M: Middleware<'a, <S as TextRenderer>::Color>,
{
    cursor: LineCursor,
    state: LineRenderState<'a, S, M>,
}

#[derive(Clone)]
pub(crate) struct LineRenderState<'a, S, M>
where
    S: TextRenderer + Clone,
    M: Middleware<'a, S::Color>,
{
    pub parser: Parser<'a>,
    pub character_style: S,
    pub style: TextBoxStyle,
    pub end_type: LineEndType,
    pub middleware: MiddlewareWrapper<'a, M, S::Color>,
}

impl<'a, F, M> StyledLineRenderer<'a, F, M>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
    <F as CharacterStyle>::Color: From<Rgb888>,
    M: Middleware<'a, <F as TextRenderer>::Color>,
{
    /// Creates a new line renderer.
    pub fn new(cursor: LineCursor, state: LineRenderState<'a, F, M>) -> Self {
        Self { cursor, state }
    }
}

struct RenderElementHandler<'a, 'b, F, D, M>
where
    F: TextRenderer,
    D: DrawTarget<Color = F::Color>,
{
    style: &'b mut F,
    display: &'b mut D,
    pos: Point,
    middleware: &'b MiddlewareWrapper<'a, M, F::Color>,
}

impl<'a, 'b, 'c, F, D, M> ElementHandler for RenderElementHandler<'a, 'c, F, D, M>
where
    F: CharacterStyle + TextRenderer,
    <F as CharacterStyle>::Color: From<Rgb888>,
    D: DrawTarget<Color = <F as TextRenderer>::Color>,
    M: Middleware<'b, <F as TextRenderer>::Color>,
{
    type Error = D::Error;

    fn measure(&self, st: &str) -> u32 {
        str_width(self.style, st)
    }

    fn whitespace(&mut self, st: &str, _space_count: u32, width: u32) -> Result<(), Self::Error> {
        let top_left = self.pos;
        self.pos = self
            .style
            .draw_whitespace(width, self.pos, Baseline::Top, self.display)?;

        let size = Size::new(width, self.style.line_height().saturating_as());
        let bounds = Rectangle::new(top_left, size);

        self.middleware.middleware.borrow_mut().post_render(
            self.display,
            self.style,
            st,
            bounds,
        )?;

        Ok(())
    }

    fn printed_characters(&mut self, st: &str, width: u32) -> Result<(), Self::Error> {
        let top_left = self.pos;
        self.pos = self
            .style
            .draw_string(st, self.pos, Baseline::Top, self.display)?;

        let size = Size::new(width, self.style.line_height().saturating_as());
        let bounds = Rectangle::new(top_left, size);

        self.middleware.middleware.borrow_mut().post_render(
            self.display,
            self.style,
            st,
            bounds,
        )?;

        Ok(())
    }

    fn move_cursor(&mut self, by: i32) -> Result<(), Self::Error> {
        // LineElementIterator ensures this new pos is valid.
        self.pos = Point::new(self.pos.x + by, self.pos.y);
        Ok(())
    }

    #[cfg(feature = "ansi")]
    fn sgr(&mut self, sgr: Sgr) -> Result<(), Self::Error> {
        sgr.apply(self.style);
        Ok(())
    }
}

struct StyleOnlyRenderElementHandler<'a, F> {
    style: &'a mut F,
}

impl<'a, F> ElementHandler for StyleOnlyRenderElementHandler<'a, F>
where
    F: CharacterStyle + TextRenderer,
    <F as CharacterStyle>::Color: From<Rgb888>,
{
    type Error = Infallible;

    fn measure(&self, st: &str) -> u32 {
        str_width(self.style, st)
    }

    #[cfg(feature = "ansi")]
    fn sgr(&mut self, sgr: Sgr) -> Result<(), Self::Error> {
        sgr.apply(self.style);
        Ok(())
    }
}

impl<'a, F, M> Drawable for StyledLineRenderer<'a, F, M>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
    <F as CharacterStyle>::Color: From<Rgb888>,
    M: Middleware<'a, <F as TextRenderer>::Color> + Middleware<'a, <F as CharacterStyle>::Color>,
{
    type Color = <F as CharacterStyle>::Color;
    type Output = LineRenderState<'a, F, M>;

    #[inline]
    fn draw<D>(&self, display: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let LineRenderState {
            mut parser,
            mut character_style,
            style,
            middleware,
            ..
        } = self.state.clone();

        let mut cloned_parser = parser.clone();
        middleware.set_state(ProcessingState::Measure);
        let lm = style.measure_line(
            &middleware,
            &character_style,
            &mut cloned_parser,
            self.cursor.line_width(),
        );
        middleware.set_state(ProcessingState::Render);

        let (end_type, end_pos) = if display.bounding_box().size.height == 0 {
            // We're outside of the view. Use simpler render element handler and space config.
            let mut elements = LineElementParser::new(
                &mut parser,
                &middleware,
                self.cursor.clone(),
                SpaceConfig::new_from_renderer(&character_style),
                style.alignment,
            );

            let end_type = elements
                .process(&mut StyleOnlyRenderElementHandler {
                    style: &mut character_style,
                })
                .unwrap();

            (end_type, elements.cursor.pos())
        } else {
            let (left, space_config) = style.alignment.place_line(&character_style, lm);

            let mut cursor = self.cursor.clone();
            cursor.move_cursor(left.saturating_as()).ok();

            let pos = cursor.pos();
            let mut elements = LineElementParser::new(
                &mut parser,
                &middleware,
                cursor,
                space_config,
                style.alignment,
            );

            let end_type = elements.process(&mut RenderElementHandler {
                style: &mut character_style,
                display,
                pos,
                middleware: &middleware,
            })?;

            (end_type, elements.cursor.pos())
        };

        let next_state = LineRenderState {
            parser,
            character_style,
            style,
            end_type,
            middleware,
        };

        if next_state.end_type == LineEndType::EndOfText {
            next_state.middleware.middleware.borrow_mut().post_render(
                display,
                &next_state.character_style,
                "",
                Rectangle::new(
                    end_pos,
                    Size::new(0, next_state.character_style.line_height()),
                ),
            )?;
        }

        Ok(next_state)
    }
}

#[cfg(feature = "ansi")]
impl Sgr {
    fn apply<F>(self, renderer: &mut F)
    where
        F: CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb888>,
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
        middleware::{MiddlewareWrapper, NoMiddleware},
        parser::Parser,
        rendering::{
            cursor::LineCursor,
            line::{LineRenderState, StyledLineRenderer},
            line_iter::LineEndType,
        },
        style::{TabSize, TextBoxStyle, TextBoxStyleBuilder},
        utils::test::size_for,
    };
    use embedded_graphics::{
        geometry::Point,
        mock_display::MockDisplay,
        mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
        pixelcolor::{BinaryColor, Rgb888},
        primitives::Rectangle,
        text::renderer::{CharacterStyle, TextRenderer},
        Drawable,
    };

    fn test_rendered_text<'a, S>(
        text: &'a str,
        bounds: Rectangle,
        character_style: S,
        style: TextBoxStyle,
        pattern: &[&str],
    ) where
        S: TextRenderer<Color = <S as CharacterStyle>::Color> + CharacterStyle,
        <S as CharacterStyle>::Color: From<Rgb888> + embedded_graphics::mock_display::ColorMapping,
    {
        let parser = Parser::parse(text);
        let cursor = LineCursor::new(
            bounds.size.width,
            TabSize::Spaces(4).into_pixels(&character_style),
        );

        let middleware = MiddlewareWrapper::new(NoMiddleware::new());

        let state = LineRenderState {
            parser,
            character_style,
            style,
            end_type: LineEndType::EndOfText,
            middleware,
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
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new().build();

        test_rendered_text(
            "Some sample text",
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, 7, 1)),
            character_style,
            style,
            &[
                "........................",
                "..##....................",
                ".#..#...................",
                "..#.....##..##.#....##..",
                "...#...#..#.#.#.#..#.##.",
                ".#..#..#..#.#.#.#..##...",
                "..##....##..#...#...###.",
                "........................",
                "........................",
            ],
        );
    }

    #[test]
    fn simple_render_nbsp() {
        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new().build();

        test_rendered_text(
            "Some\u{A0}sample text",
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, 7, 1)),
            character_style,
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
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new().build();

        test_rendered_text(
            "Some sample text",
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, 2, 1)),
            character_style,
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
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new().build();

        test_rendered_text(
            "Some \nsample text",
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, 7, 1)),
            character_style,
            style,
            &[
                "........................",
                "..##....................",
                ".#..#...................",
                "..#.....##..##.#....##..",
                "...#...#..#.#.#.#..#.##.",
                ".#..#..#..#.#.#.#..##...",
                "..##....##..#...#...###.",
                "........................",
                "........................",
            ],
        );
    }
}

#[cfg(all(test, feature = "ansi"))]
mod ansi_parser_tests {
    use crate::{
        middleware::{MiddlewareWrapper, NoMiddleware},
        parser::Parser,
        rendering::{
            cursor::LineCursor,
            line::{LineRenderState, StyledLineRenderer},
            line_iter::LineEndType,
        },
        style::{TabSize, TextBoxStyleBuilder},
        utils::test::size_for,
    };
    use embedded_graphics::{
        mock_display::MockDisplay,
        mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        Drawable,
    };

    #[test]
    fn ansi_cursor_backwards() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let parser = Parser::parse("foo\x1b[2Dsample");

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new().build();

        let cursor = LineCursor::new(
            size_for(&FONT_6X9, 7, 1).width,
            TabSize::Spaces(4).into_pixels(&character_style),
        );

        let middleware = MiddlewareWrapper::new(NoMiddleware::new());
        let state = LineRenderState {
            parser,
            character_style,
            style,
            end_type: LineEndType::EndOfText,
            middleware,
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
