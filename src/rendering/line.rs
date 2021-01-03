//! Line rendering.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        character::GlyphRenderer,
        cursor::Cursor,
        decorated_space::DecoratedSpaceRenderer,
        line_iter::{LineElementParser, RenderElement},
    },
    style::{color::Rgb, height_mode::HeightMode, TextBoxStyle},
};
use core::cell::RefCell;
use core::ops::Range;
use embedded_graphics::{fonts::MonoFont, prelude::*};

#[cfg(feature = "ansi")]
use crate::rendering::ansi::Sgr;

#[derive(Debug)]
struct Refs<'a, 'b, C, F, A, V, H>
where
    C: PixelColor,
    F: MonoFont,
{
    parser: &'b mut Parser<'a>,
    cursor: &'b mut Cursor<F>,
    style: &'b mut TextBoxStyle<C, F, A, V, H>,
    carried_token: &'b mut Option<Token<'a>>,
}

/// Render a single line of styled text.
#[derive(Debug)]
pub struct StyledLineRenderer<'a, 'b, C, F, A, V, H>
where
    C: PixelColor,
    F: MonoFont,
{
    display_range: Range<i32>,
    inner: RefCell<Refs<'a, 'b, C, F, A, V, H>>,
}

impl<'a, 'b, C, F, A, V, H> StyledLineRenderer<'a, 'b, C, F, A, V, H>
where
    C: PixelColor,
    F: MonoFont,
    H: HeightMode,
{
    /// Creates a new line renderer.
    pub fn new(
        parser: &'b mut Parser<'a>,
        cursor: &'b mut Cursor<F>,
        style: &'b mut TextBoxStyle<C, F, A, V, H>,
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

    fn is_anything_displayed(&self) -> bool {
        self.display_range.start < self.display_range.end
    }
}

impl<'a, 'b, C, F, A, V, H> Drawable for StyledLineRenderer<'a, 'b, C, F, A, V, H>
where
    C: PixelColor + From<Rgb>,
    F: MonoFont,
    A: HorizontalTextAlignment,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Color = C;

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
        let (width, total_spaces, t, _) =
            style.measure_line(&mut parser.clone(), carried_token.clone(), max_line_width);

        let (left, space_config) = A::place_line::<F>(max_line_width, width, total_spaces, t);

        cursor.advance_unchecked(left);

        let mut elements = LineElementParser::<'_, '_, F, _, A>::new(
            parser,
            cursor,
            space_config,
            carried_token,
            style.tab_size,
        );

        while let Some(element) = elements.next() {
            // HACK: avoid drawing the underline outside of the text box
            let underlined = if style.underlined {
                elements.cursor.position.y + self.display_range.end - 1
                    < elements.cursor.bottom_right().y
            } else {
                false
            };

            match element {
                RenderElement::PrintedCharacter(c) => {
                    if !self.is_anything_displayed() {
                        continue;
                    }
                    GlyphRenderer::new(
                        c,
                        style.text_style,
                        elements.pos,
                        self.display_range.clone(),
                        underlined,
                        style.strikethrough,
                    )
                    .draw(display)?;
                }

                RenderElement::Space(space_width, _) => {
                    if !self.is_anything_displayed() {
                        continue;
                    }
                    DecoratedSpaceRenderer::new(
                        style.text_style,
                        elements.pos,
                        space_width,
                        self.display_range.clone(),
                        underlined,
                        style.strikethrough,
                    )
                    .draw(display)?;
                }

                #[cfg(feature = "ansi")]
                RenderElement::Sgr(sgr) => match sgr {
                    Sgr::Reset => {
                        style.text_style.text_color = None;
                        style.text_style.background_color = None;
                        style.underlined = false;
                        style.strikethrough = false;
                    }
                    Sgr::ChangeTextColor(color) => {
                        style.text_style.text_color = Some(color.into());
                    }
                    Sgr::DefaultTextColor => {
                        style.text_style.text_color = None;
                    }
                    Sgr::ChangeBackgroundColor(color) => {
                        style.text_style.background_color = Some(color.into());
                    }
                    Sgr::DefaultBackgroundColor => {
                        style.text_style.background_color = None;
                    }
                    Sgr::Underline => {
                        style.underlined = true;
                    }
                    Sgr::UnderlineOff => {
                        style.underlined = false;
                    }
                    Sgr::CrossedOut => {
                        style.strikethrough = true;
                    }
                    Sgr::NotCrossedOut => {
                        style.strikethrough = false;
                    }
                },
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        alignment::{HorizontalTextAlignment, VerticalTextAlignment},
        parser::{Parser, Token},
        rendering::{cursor::Cursor, line::StyledLineRenderer},
        style::{color::Rgb, height_mode::HeightMode, TextBoxStyle, TextBoxStyleBuilder},
    };
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    };
    use embedded_graphics_core::primitives::Rectangle;

    fn test_rendered_text<'a, C, F, A, V, H>(
        text: &'a str,
        bounds: Rectangle,
        mut style: TextBoxStyle<C, F, A, V, H>,
        pattern: &[&str],
    ) where
        C: PixelColor + From<Rgb> + embedded_graphics::mock_display::ColorMapping,
        F: MonoFont,
        A: HorizontalTextAlignment,
        V: VerticalTextAlignment,
        H: HeightMode,
    {
        let mut parser = Parser::parse(text);
        let mut cursor = Cursor::new(bounds, style.line_spacing);
        let mut carried = None;

        let renderer = StyledLineRenderer::new(&mut parser, &mut cursor, &mut style, &mut carried);
        let mut display = MockDisplay::new();

        renderer.draw(&mut display).unwrap();

        display.assert_pattern(pattern);
    }

    #[test]
    fn simple_render() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        test_rendered_text(
            " Some sample text",
            Rectangle::new(Point::zero(), Size::new(6 * 7, 8)),
            style,
            &[
                ".......###....................",
                "......#...#...................",
                "......#......###..##.#...###..",
                ".......###..#...#.#.#.#.#...#.",
                "..........#.#...#.#...#.#####.",
                "......#...#.#...#.#...#.#.....",
                ".......###...###..#...#..###..",
                "..............................",
            ],
        );
    }

    #[test]
    fn render_before_area() {
        let mut parser = Parser::parse(" Some sample text");
        let mut style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let mut cursor = Cursor::new(
            Rectangle::new(Point::new(0, 8), Size::new(6 * 7, 16)),
            style.line_spacing,
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
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        test_rendered_text(
            "Some\u{A0}sample text",
            Rectangle::new(Point::zero(), Size::new(6 * 7, 8)),
            style,
            &[
                ".###......................................",
                "#...#.....................................",
                "#......###..##.#...###.........####..###..",
                ".###..#...#.#.#.#.#...#.......#.........#.",
                "....#.#...#.#...#.#####........###...####.",
                "#...#.#...#.#...#.#...............#.#...#.",
                ".###...###..#...#..###........####...####.",
                "..........................................",
            ],
        );
    }

    #[test]
    fn simple_render_first_word_not_wrapped() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        test_rendered_text(
            "Some sample text",
            Rectangle::new(Point::zero(), Size::new(6 * 2, 8)),
            style,
            &[
                ".###........",
                "#...#.......",
                "#......###..",
                ".###..#...#.",
                "....#.#...#.",
                "#...#.#...#.",
                ".###...###..",
                "............",
            ],
        );
    }

    #[test]
    fn newline_stops_render() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        test_rendered_text(
            "Some \nsample text",
            Rectangle::new(Point::zero(), Size::new(6 * 7, 8)),
            style,
            &[
                ".###..........................",
                "#...#.........................",
                "#......###..##.#...###........",
                ".###..#...#.#.#.#.#...#.......",
                "....#.#...#.#...#.#####.......",
                "#...#.#...#.#...#.#...........",
                ".###...###..#...#..###........",
                "..............................",
            ],
        );
    }

    #[test]
    fn underline_just_inside_of_textbox() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .underlined(true)
            .build();

        test_rendered_text(
            "s",
            Rectangle::new(Point::zero(), Size::new(6, 9)),
            style,
            &[
                "......             ",
                "......             ",
                ".####.             ",
                "#.....             ",
                ".###..             ",
                "....#.             ",
                "####..             ",
                "......             ",
                "######             ",
            ],
        );
    }

    #[test]
    fn underline_outside_of_textbox() {
        let style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .underlined(true)
            .build();

        test_rendered_text(
            "s",
            Rectangle::new(Point::zero(), Size::new(6, 8)),
            style,
            &[
                "......             ",
                "......             ",
                ".####.             ",
                "#.....             ",
                ".###..             ",
                "....#.             ",
                "####..             ",
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
        style::TextBoxStyleBuilder,
    };
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    };
    use embedded_graphics_core::primitives::Rectangle;

    #[test]
    fn ansi_cursor_backwards() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let mut parser = Parser::parse("foo\x1b[2Dsample");
        let mut style = TextBoxStyleBuilder::new(Font6x8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();
        let mut cursor = Cursor::new(Rectangle::new(Point::zero(), Size::new(6 * 7, 8)), 0);
        let mut carried = None;
        StyledLineRenderer::new(&mut parser, &mut cursor, &mut style, &mut carried)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "..##...........................##.........",
            ".#..#...........................#.........",
            ".#.....####..###..##.#..####....#....###..",
            "###...#.........#.#.#.#.#...#...#...#...#.",
            ".#.....###...####.#...#.#...#...#...#####.",
            ".#........#.#...#.#...#.####....#...#.....",
            ".#....####...####.#...#.#......###...###..",
            "........................#.................",
        ]);
    }
}
