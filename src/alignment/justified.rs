//! Fully justified text.
use crate::{
    alignment::HorizontalTextAlignment, parser::SPEC_CHAR_NBSP,
    rendering::space_config::SpaceConfig, style::LineMeasurement, utils::str_width,
};
use embedded_graphics::text::renderer::TextRenderer;

/// Marks text to be rendered fully justified.
#[derive(Copy, Clone, Debug)]
pub struct Justified;
impl HorizontalTextAlignment for Justified {
    type SpaceConfig = JustifiedSpaceConfig;

    #[inline]
    fn place_line(
        line: &str,
        renderer: &impl TextRenderer,
        measurement: LineMeasurement,
    ) -> (u32, Self::SpaceConfig) {
        let space_width = str_width(renderer, " ");
        let space_chars = [' ', SPEC_CHAR_NBSP];

        let mut space_count = 0;
        let mut partial_space_count = 0;

        for c in line.chars().skip_while(|c| space_chars.contains(c)) {
            if space_chars.contains(&c) {
                partial_space_count += 1;
            } else {
                space_count += partial_space_count;
                partial_space_count = 0;
            }
        }

        let space_info = if !measurement.last_line && space_count != 0 {
            let space = measurement.max_line_width - measurement.width + space_count * space_width;
            let space_width = space / space_count;
            let extra_pixels = space % space_count;
            JustifiedSpaceConfig::new(space_width, extra_pixels)
        } else {
            JustifiedSpaceConfig::new(space_width, 0)
        };
        (0, space_info)
    }
}

/// Internal state information used to store width of whitespace characters when rendering fully
/// justified text.
///
/// The fully justified renderer works by calculating the width of whitespace characters for the
/// current line. Due to integer arithmetic, there can be remainder pixels when a single space
/// width is used. This struct stores two width values so the whole line will always (at least if
/// there's a space in the line) take up all available space.
#[derive(Copy, Clone, Debug)]
pub struct JustifiedSpaceConfig {
    /// The width of the whitespace characters.
    space_width: u32,

    /// Stores how many characters are rendered using the space_width width. This field changes
    /// during rendering.
    space_count: u32,
}

impl JustifiedSpaceConfig {
    #[inline]
    #[must_use]
    pub(crate) fn new(space_width: u32, extra_pixel_count: u32) -> Self {
        JustifiedSpaceConfig {
            space_width,
            space_count: extra_pixel_count,
        }
    }
}

impl SpaceConfig for JustifiedSpaceConfig {
    #[inline]
    fn peek_next_width(&self, whitespace_count: u32) -> u32 {
        whitespace_count * self.space_width + self.space_count.min(whitespace_count)
    }

    #[inline]
    fn consume(&mut self, n: u32) -> u32 {
        let w = self.peek_next_width(n);
        self.space_count = self.space_count.saturating_sub(n);
        w
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        geometry::Point,
        mock_display::MockDisplay,
        mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        primitives::Rectangle,
        Drawable,
    };

    use crate::{
        alignment::Justified, rendering::test::assert_rendered, style::TextBoxStyleBuilder,
        utils::test::size_for, TextBox,
    };

    #[test]
    fn simple_render() {
        assert_rendered(
            Justified,
            "word",
            size_for(&FONT_6X9, 6, 1),
            &[
                "........................",
                "......................#.",
                "......................#.",
                "#...#...##...#.#....###.",
                "#.#.#..#..#..##.#..#..#.",
                "#.#.#..#..#..#.....#..#.",
                ".#.#....##...#......###.",
                "........................",
                "........................",
            ],
        );
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(Justified)
            .build();

        TextBox::with_textbox_style(
            "O\rX",
            Rectangle::new(Point::zero(), size_for(&FONT_6X9, 1, 1)),
            style,
        )
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "         ",
            "#####    ",
            "## ##    ",
            "# # #    ",
            "# # #    ",
            "## ##    ",
            "#####    ",
        ]);
    }

    #[test]
    fn wrapping_when_space_is_less_than_space_character() {
        assert_rendered(
            Justified,
            "A word",
            size_for(&FONT_6X9, 5, 1),
            &[
                "......            ",
                "..#...            ",
                ".#.#..            ",
                "#...#.            ",
                "#####.            ",
                "#...#.            ",
                "#...#.            ",
                "......            ",
                "......            ",
            ],
        );
    }

    #[test]
    fn simple_word_wrapping() {
        assert_rendered(
            Justified,
            "word wrapping",
            size_for(&FONT_6X9, 9, 2),
            &[
                "........................                        ",
                "......................#.                        ",
                "......................#.                        ",
                "#...#...##...#.#....###.                        ",
                "#.#.#..#..#..##.#..#..#.                        ",
                "#.#.#..#..#..#.....#..#.                        ",
                ".#.#....##...#......###.                        ",
                "........................                        ",
                "........................                        ",
                "................................................",
                "................................#...............",
                "................................................",
                "#...#..#.#....###..###...###...##....###....##..",
                "#.#.#..##.#..#..#..#..#..#..#...#....#..#..#..#.",
                "#.#.#..#.....#..#..#..#..#..#...#....#..#..#..#.",
                ".#.#...#......###..###...###...###...#..#...###.",
                "...................#.....#....................#.",
                "...................#.....#..................##..",
            ],
        );
    }

    #[test]
    fn justified_alignment() {
        assert_rendered(
            Justified,
            "word and other word last line",
            size_for(&FONT_6X9, 10, 3),
            &[
                "............................................................",
                "......................#...................................#.",
                "......................#...................................#.",
                "#...#...##...#.#....###.....................###..###....###.",
                "#.#.#..#..#..##.#..#..#....................#..#..#..#..#..#.",
                "#.#.#..#..#..#.....#..#....................#..#..#..#..#..#.",
                ".#.#....##...#......###.....................###..#..#...###.",
                "............................................................",
                "............................................................",
                "............................................................",
                "........#....#............................................#.",
                "........#....#............................................#.",
                "..##...###...###....##...#.#........#...#...##...#.#....###.",
                ".#..#...#....#..#..#.##..##.#.......#.#.#..#..#..##.#..#..#.",
                ".#..#...#.#..#..#..##....#..........#.#.#..#..#..#.....#..#.",
                "..##.....#...#..#...###..#...........#.#....##...#......###.",
                "............................................................",
                "............................................................",
                "......................................................      ",
                ".##.................#..........##.....#...............      ",
                "..#.................#...........#.....................      ",
                "..#.....###...###..###..........#....##....###....##..      ",
                "..#....#..#..##.....#...........#.....#....#..#..#.##.      ",
                "..#....#..#....##...#.#.........#.....#....#..#..##...      ",
                ".###....###..###.....#.........###...###...#..#...###.      ",
                "......................................................      ",
                "......................................................      ",
            ],
        );
    }

    #[test]
    fn word_longer_than_line_wraps_word() {
        assert_rendered(
            Justified,
            "word somereallylongword",
            size_for(&FONT_6X9, 9, 3),
            &[
                "........................                              ",
                "......................#.                              ",
                "......................#.                              ",
                "#...#...##...#.#....###.                              ",
                "#.#.#..#..#..##.#..#..#.                              ",
                "#.#.#..#..#..#.....#..#.                              ",
                ".#.#....##...#......###.                              ",
                "........................                              ",
                "........................                              ",
                "......................................................",
                "...........................................##....##...",
                "............................................#.....#...",
                "..###...##..##.#....##...#.#....##....###...#.....#...",
                ".##....#..#.#.#.#..#.##..##.#..#.##..#..#...#.....#...",
                "...##..#..#.#.#.#..##....#.....##....#..#...#.....#...",
                ".###....##..#...#...###..#......###...###..###...###..",
                "......................................................",
                "......................................................",
                "......................................................",
                ".......##...........................................#.",
                "........#...........................................#.",
                ".#..#...#.....##...###....##..#...#...##...#.#....###.",
                ".#..#...#....#..#..#..#..#..#.#.#.#..#..#..##.#..#..#.",
                ".#..#...#....#..#..#..#..#..#.#.#.#..#..#..#.....#..#.",
                "..###..###....##...#..#...###..#.#....##...#......###.",
                ".#..#.......................#.........................",
                "..##......................##..........................",
            ],
        );
    }

    #[test]
    fn first_word_longer_than_line_wraps_word() {
        assert_rendered(
            Justified,
            "somereallylongword",
            size_for(&FONT_6X9, 9, 2),
            &[
                "......................................................",
                "...........................................##....##...",
                "............................................#.....#...",
                "..###...##..##.#....##...#.#....##....###...#.....#...",
                ".##....#..#.#.#.#..#.##..##.#..#.##..#..#...#.....#...",
                "...##..#..#.#.#.#..##....#.....##....#..#...#.....#...",
                ".###....##..#...#...###..#......###...###..###...###..",
                "......................................................",
                "......................................................",
                "......................................................",
                ".......##...........................................#.",
                "........#...........................................#.",
                ".#..#...#.....##...###....##..#...#...##...#.#....###.",
                ".#..#...#....#..#..#..#..#..#.#.#.#..#..#..##.#..#..#.",
                ".#..#...#....#..#..#..#..#..#.#.#.#..#..#..#.....#..#.",
                "..###..###....##...#..#...###..#.#....##...#......###.",
                ".#..#.......................#.........................",
                "..##......................##..........................",
            ],
        );
    }

    #[test]
    fn soft_hyphen_rendering() {
        assert_rendered(
            Justified,
            "soft\u{AD}hyphen",
            size_for(&FONT_6X9, 6, 2),
            &[
                "..............................      ",
                "...............#....#.........      ",
                "..............#.#...#.........      ",
                "..###...##....#....###........      ",
                ".##....#..#..###....#...#####.      ",
                "...##..#..#...#.....#.#.......      ",
                ".###....##....#......#........      ",
                "..............................      ",
                "..............................      ",
                "....................................",
                ".#.................#................",
                ".#.................#................",
                ".###...#..#..###...###....##...###..",
                ".#..#..#..#..#..#..#..#..#.##..#..#.",
                ".#..#..#..#..#..#..#..#..##....#..#.",
                ".#..#...###..###...#..#...###..#..#.",
                ".......#..#..#......................",
                "........##...#......................",
            ],
        );
    }

    #[test]
    fn tab_rendering() {
        // Expect \t to render as 3 space characters, ignored by the justified alignment.
        assert_rendered(
            Justified,
            "a\ttab + two te xt words",
            size_for(&FONT_6X9, 10, 3),
            &[
                "............................................................",
                "..........................#..........#......................",
                "..........................#..........#..................#...",
                "..###....................###....###..###................#...",
                ".#..#.....................#....#..#..#..#.............#####.",
                ".#..#.....................#.#..#..#..#..#...............#...",
                "..###......................#....###..###................#...",
                "............................................................",
                "............................................................",
                "............................................................",
                "..#..........................#..........................#...",
                "..#..........................#..........................#...",
                ".###..#...#...##............###....##............#..#..###..",
                "..#...#.#.#..#..#............#....#.##............##....#...",
                "..#.#.#.#.#..#..#............#.#..##..............##....#.#.",
                "...#...#.#....##..............#....###...........#..#....#..",
                "............................................................",
                "............................................................",
                "..............................                              ",
                "......................#.......                              ",
                "......................#.......                              ",
                "#...#...##...#.#....###...###.                              ",
                "#.#.#..#..#..##.#..#..#..##...                              ",
                "#.#.#..#..#..#.....#..#....##.                              ",
                ".#.#....##...#......###..###..                              ",
                "..............................                              ",
                "..............................                              ",
            ],
        );
    }
}
