//! Right aligned text.
use crate::{
    alignment::HorizontalTextAlignment, rendering::space_config::UniformSpaceConfig,
    style::LineMeasurement,
};
use embedded_graphics::text::TextRenderer;

/// Marks text to be rendered right aligned.
#[derive(Copy, Clone, Debug)]
pub struct RightAligned;
impl HorizontalTextAlignment for RightAligned {
    type SpaceConfig = UniformSpaceConfig;

    const STARTING_SPACES: bool = false;
    const ENDING_SPACES: bool = false;

    #[inline]
    fn place_line(
        _line: &str,
        renderer: &impl TextRenderer,
        max_width: u32,
        measurement: LineMeasurement,
    ) -> (u32, Self::SpaceConfig) {
        (
            max_width - measurement.width,
            UniformSpaceConfig::new(renderer),
        )
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        geometry::Point,
        mock_display::MockDisplay,
        mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        primitives::Rectangle,
        Drawable,
    };

    use crate::{
        alignment::RightAligned, rendering::test::assert_rendered, style::TextBoxStyleBuilder,
        utils::test::size_for, TextBox,
    };

    #[test]
    fn simple_render() {
        assert_rendered(
            RightAligned,
            "word",
            size_for(Font6x9, 6, 1),
            &[
                "            ........................",
                "            ......................#.",
                "            ......................#.",
                "            #...#...##...#.#....###.",
                "            #.#.#..#..#..##.#..#..#.",
                "            #.#.#..#..#..#.....#..#.",
                "            .#.#....##...#......###.",
                "            ........................",
                "            ........................",
            ],
        );
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(RightAligned)
            .build();

        TextBox::new(
            "O\rX",
            Rectangle::new(Point::zero(), size_for(Font6x9, 3, 1)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "                  ",
            "            ##### ",
            "            ## ## ",
            "            # # # ",
            "            # # # ",
            "            ## ## ",
            "            ##### ",
        ]);
    }

    #[test]
    fn simple_word_wrapping() {
        assert_rendered(
            RightAligned,
            "word wrapping",
            size_for(Font6x9, 9, 2),
            &[
                "                              ........................",
                "                              ......................#.",
                "                              ......................#.",
                "                              #...#...##...#.#....###.",
                "                              #.#.#..#..#..##.#..#..#.",
                "                              #.#.#..#..#..#.....#..#.",
                "                              .#.#....##...#......###.",
                "                              ........................",
                "                              ........................",
                "      ................................................",
                "      ................................#...............",
                "      ................................................",
                "      #...#..#.#....###..###...###...##....###....##..",
                "      #.#.#..##.#..#..#..#..#..#..#...#....#..#..#..#.",
                "      #.#.#..#.....#..#..#..#..#..#...#....#..#..#..#.",
                "      .#.#...#......###..###...###...###...#..#...###.",
                "      ...................#.....#....................#.",
                "      ...................#.....#..................##..",
            ],
        );
    }

    #[test]
    fn word_longer_than_line_wraps_word() {
        assert_rendered(
            RightAligned,
            "word  somereallylongword",
            size_for(Font6x9, 9, 3),
            &[
                "                              ........................",
                "                              ......................#.",
                "                              ......................#.",
                "                              #...#...##...#.#....###.",
                "                              #.#.#..#..#..##.#..#..#.",
                "                              #.#.#..#..#..#.....#..#.",
                "                              .#.#....##...#......###.",
                "                              ........................",
                "                              ........................",
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
            RightAligned,
            "somereallylongword",
            size_for(Font6x9, 9, 2),
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
            RightAligned,
            "soft\u{AD}hyphen",
            size_for(Font6x9, 6, 2),
            &[
                "      ..............................",
                "      ...............#....#.........",
                "      ..............#.#...#.........",
                "      ..###...##....#....###........",
                "      .##....#..#..###....#...#####.",
                "      ...##..#..#...#.....#.#.......",
                "      .###....##....#......#........",
                "      ..............................",
                "      ..............................",
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
}
