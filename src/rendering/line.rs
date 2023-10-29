//! Line rendering.

use crate::{
    parser::{ChangeTextStyle, Parser},
    plugin::{PluginMarker as Plugin, PluginWrapper, ProcessingState},
    rendering::{
        cursor::LineCursor,
        line_iter::{ElementHandler, LineElementParser, LineEndType},
    },
    style::TextBoxStyle,
    utils::str_width,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    prelude::{PixelColor, Size},
    primitives::Rectangle,
    text::{
        renderer::{CharacterStyle, TextRenderer},
        Baseline, DecorationColor,
    },
};

impl<C> ChangeTextStyle<C>
where
    C: PixelColor + Default,
{
    pub(crate) fn apply<S: CharacterStyle<Color = C>>(self, text_renderer: &mut S) {
        match self {
            ChangeTextStyle::Reset => {
                text_renderer.set_text_color(Some(C::default()));
                text_renderer.set_background_color(None);
                text_renderer.set_underline_color(DecorationColor::None);
                text_renderer.set_strikethrough_color(DecorationColor::None);
            }
            ChangeTextStyle::TextColor(color) => text_renderer.set_text_color(color),
            ChangeTextStyle::BackgroundColor(color) => text_renderer.set_background_color(color),
            ChangeTextStyle::Underline(color) => text_renderer.set_underline_color(color),
            ChangeTextStyle::Strikethrough(color) => text_renderer.set_strikethrough_color(color),
        }
    }
}

/// Render a single line of styled text.
pub(crate) struct StyledLineRenderer<'a, 'b, 'c, S, M>
where
    S: TextRenderer + Clone,
    M: Plugin<'a, <S as TextRenderer>::Color>,
{
    pub(crate) cursor: LineCursor,
    pub(crate) state: &'c mut LineRenderState<'a, 'b, S, M>,
    pub(crate) style: &'c TextBoxStyle,
}

#[derive(Clone)]
pub(crate) struct LineRenderState<'a, 'b, S, M>
where
    S: TextRenderer + Clone,
    M: Plugin<'a, S::Color>,
{
    pub parser: Parser<'a, S::Color>,
    pub text_renderer: S,
    pub end_type: LineEndType,
    pub plugin: &'b PluginWrapper<'a, M, S::Color>,
}

struct RenderElementHandler<'a, 'b, F, D, M>
where
    F: TextRenderer,
    D: DrawTarget<Color = F::Color>,
{
    text_renderer: &'b mut F,
    display: &'b mut D,
    pos: Point,
    plugin: &'b PluginWrapper<'a, M, F::Color>,
}

impl<'a, 'b, F, D, M> RenderElementHandler<'a, 'b, F, D, M>
where
    F: CharacterStyle + TextRenderer,
    D: DrawTarget<Color = <F as TextRenderer>::Color>,
    M: Plugin<'a, <F as TextRenderer>::Color>,
{
    fn post_print(&mut self, width: u32, st: &str) -> Result<(), D::Error> {
        let bounds = Rectangle::new(self.pos, Size::new(width, self.text_renderer.line_height()));

        self.pos += Point::new(width as i32, 0);

        self.plugin
            .post_render(self.display, self.text_renderer, Some(st), bounds)
    }
}

impl<'a, 'c, F, D, M> ElementHandler for RenderElementHandler<'a, 'c, F, D, M>
where
    F: CharacterStyle + TextRenderer,
    D: DrawTarget<Color = <F as TextRenderer>::Color>,
    M: Plugin<'a, <F as TextRenderer>::Color>,
    <F as CharacterStyle>::Color: Default,
{
    type Error = D::Error;
    type Color = <F as CharacterStyle>::Color;

    fn measure(&self, st: &str) -> u32 {
        str_width(self.text_renderer, st)
    }

    fn whitespace(&mut self, st: &str, _space_count: u32, width: u32) -> Result<(), Self::Error> {
        if width > 0 {
            self.text_renderer
                .draw_whitespace(width, self.pos, Baseline::Top, self.display)?;
        }

        self.post_print(width, st)
    }

    fn printed_characters(&mut self, st: &str, width: Option<u32>) -> Result<(), Self::Error> {
        let render_width =
            self.text_renderer
                .draw_string(st, self.pos, Baseline::Top, self.display)?;

        let width = width.unwrap_or((render_width - self.pos).x as u32);

        self.post_print(width, st)
    }

    fn move_cursor(&mut self, by: i32) -> Result<(), Self::Error> {
        // LineElementIterator ensures this new pos is valid.
        self.pos += Point::new(by, 0);
        Ok(())
    }

    fn change_text_style(
        &mut self,
        change: ChangeTextStyle<<F as CharacterStyle>::Color>,
    ) -> Result<(), Self::Error> {
        change.apply(self.text_renderer);
        Ok(())
    }
}

impl<'a, 'b, 'c, F, M> StyledLineRenderer<'a, 'b, 'c, F, M>
where
    F: TextRenderer<Color = <F as CharacterStyle>::Color> + CharacterStyle,
    M: Plugin<'a, <F as TextRenderer>::Color> + Plugin<'a, <F as CharacterStyle>::Color>,
    <F as CharacterStyle>::Color: Default,
{
    #[inline]
    pub(crate) fn draw<D>(mut self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = <F as CharacterStyle>::Color>,
    {
        let LineRenderState {
            ref mut parser,
            ref mut text_renderer,
            plugin,
            ..
        } = self.state;

        let lm = {
            // Ensure the clone lives for as short as possible.
            let mut cloned_parser = parser.clone();
            let measure_plugin = plugin.clone();
            measure_plugin.set_state(ProcessingState::Measure);
            self.style.measure_line(
                &measure_plugin,
                text_renderer,
                &mut cloned_parser,
                self.cursor.line_width(),
            )
        };

        let (left, space_config) = self.style.alignment.place_line(text_renderer, lm);

        self.cursor.move_cursor(left).ok();

        let mut render_element_handler = RenderElementHandler {
            text_renderer,
            display,
            pos: self.cursor.pos(),
            plugin: *plugin,
        };
        let end_type =
            LineElementParser::new(parser, plugin, self.cursor, space_config, self.style)
                .process(&mut render_element_handler)?;

        if end_type == LineEndType::EndOfText {
            let end_pos = render_element_handler.pos;
            plugin.post_render(
                display,
                text_renderer,
                None,
                Rectangle::new(end_pos, Size::new(0, text_renderer.line_height())),
            )?;
        }

        self.state.end_type = end_type;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        parser::Parser,
        plugin::{NoPlugin, PluginWrapper},
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
        pixelcolor::BinaryColor,
        primitives::Rectangle,
        text::renderer::{CharacterStyle, TextRenderer},
    };

    fn test_rendered_text<'a, S>(
        text: &'a str,
        bounds: Rectangle,
        character_style: S,
        style: TextBoxStyle,
        pattern: &[&str],
    ) where
        S: TextRenderer<Color = <S as CharacterStyle>::Color> + CharacterStyle,
        <S as CharacterStyle>::Color: embedded_graphics::mock_display::ColorMapping + Default,
    {
        let parser = Parser::parse(text);
        let cursor = LineCursor::new(
            bounds.size.width,
            TabSize::Spaces(4).into_pixels(&character_style),
        );

        let plugin = PluginWrapper::new(NoPlugin::new());

        let mut state = LineRenderState {
            parser,
            text_renderer: character_style,
            end_type: LineEndType::EndOfText,
            plugin: &plugin,
        };

        let renderer = StyledLineRenderer {
            cursor,
            state: &mut state,
            style: &style,
        };
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
