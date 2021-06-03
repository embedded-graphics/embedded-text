//! Line rendering.
use core::convert::Infallible;

use crate::{
    parser::{Parser, Token},
    rendering::{cursor::LineCursor, line_iter::LineElementParser},
    style::TextBoxStyle,
    utils::str_width,
};
use az::SaturatingAs;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::Rgb888,
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
#[derive(Debug)]
pub struct StyledLineRenderer<'a, S>
where
    S: Clone,
{
    cursor: LineCursor,
    state: LineRenderState<'a, S>,
}

#[derive(Debug, Clone)]
pub struct LineRenderState<'a, S>
where
    S: Clone,
{
    pub parser: Parser<'a>,
    pub character_style: S,
    pub style: TextBoxStyle,
    pub carried_token: Option<Token<'a>>,
}

impl<S> LineRenderState<'_, S>
where
    S: Clone,
{
    pub fn is_finished(&self) -> bool {
        self.carried_token.is_none() && self.parser.is_empty()
    }
}

impl<'a, F> StyledLineRenderer<'a, F>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
    <F as CharacterStyle>::Color: From<Rgb888>,
{
    /// Creates a new line renderer.
    pub fn new(cursor: LineCursor, state: LineRenderState<'a, F>) -> Self {
        Self { cursor, state }
    }
}

struct RenderElementHandler<'a, F, D> {
    style: &'a mut F,
    display: &'a mut D,
    pos: Point,
}

impl<'a, F, D> ElementHandler for RenderElementHandler<'a, F, D>
where
    F: CharacterStyle + TextRenderer,
    <F as CharacterStyle>::Color: From<Rgb888>,
    D: DrawTarget<Color = <F as TextRenderer>::Color>,
{
    type Error = D::Error;

    fn measure(&self, st: &str) -> u32 {
        str_width(self.style, st)
    }

    fn whitespace(&mut self, width: u32) -> Result<(), Self::Error> {
        self.pos = self
            .style
            .draw_whitespace(width, self.pos, Baseline::Top, self.display)?;
        Ok(())
    }

    fn printed_characters(&mut self, st: &str, _: u32) -> Result<(), Self::Error> {
        self.pos = self
            .style
            .draw_string(st, self.pos, Baseline::Top, self.display)?;
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

impl<'a, F> Drawable for StyledLineRenderer<'a, F>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
    <F as CharacterStyle>::Color: From<Rgb888>,
{
    type Color = <F as CharacterStyle>::Color;
    type Output = LineRenderState<'a, F>;

    #[inline]
    fn draw<D>(&self, display: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let LineRenderState {
            mut parser,
            mut character_style,
            style,
            carried_token,
        } = self.state.clone();

        let carried = if display.bounding_box().size.height == 0 {
            // We're outside of the view - no need for a separate measure pass.
            let mut elements = LineElementParser::new(
                &mut parser,
                self.cursor.clone(),
                SpaceConfig::new_from_renderer(&character_style),
                carried_token,
                style.alignment,
            );

            elements
                .process(&mut StyleOnlyRenderElementHandler {
                    style: &mut character_style,
                })
                .unwrap()
        } else {
            // We have to resort to trickery to figure out the string that is rendered as the line.
            let mut cloned_parser = parser.clone();
            let lm = style.measure_line(
                &character_style,
                &mut cloned_parser,
                &mut carried_token.clone(),
                self.cursor.line_width(),
            );

            let consumed_bytes = parser.as_str().len() - cloned_parser.as_str().len();
            let line_str = unsafe { parser.as_str().get_unchecked(..consumed_bytes) };

            let (left, space_config) = style.alignment.place_line(line_str, &character_style, lm);

            let mut cursor = self.cursor.clone();
            cursor.move_cursor(left.saturating_as()).ok();

            let pos = cursor.pos();
            let mut elements = LineElementParser::new(
                &mut parser,
                cursor,
                space_config,
                carried_token,
                style.alignment,
            );

            elements.process(&mut RenderElementHandler {
                style: &mut character_style,
                display,
                pos,
            })?
        };

        Ok(LineRenderState {
            parser,
            character_style,
            style,
            carried_token: carried,
        })
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
        parser::Parser,
        rendering::{
            cursor::LineCursor,
            line::{LineRenderState, StyledLineRenderer},
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

        let state = LineRenderState {
            parser,
            character_style,
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
        let state = LineRenderState {
            parser,
            character_style,
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
