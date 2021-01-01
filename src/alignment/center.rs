//! Horizontal and vertical center aligned text.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::Token,
    rendering::{cursor::Cursor, space_config::UniformSpaceConfig},
    style::{color::Rgb, height_mode::HeightMode},
    StyledTextBox,
};

use embedded_graphics::{
    geometry::Dimensions,
    text::{CharacterStyle, TextRenderer},
};

/// Marks text to be rendered center aligned.
///
/// This alignment can be used as both horizontal or vertical alignment.
#[derive(Copy, Clone, Debug)]
pub struct CenterAligned;
impl HorizontalTextAlignment for CenterAligned {
    type SpaceConfig = UniformSpaceConfig;

    const STARTING_SPACES: bool = false;
    const ENDING_SPACES: bool = false;

    #[inline]
    fn place_line(
        renderer: &impl TextRenderer,
        max_width: u32,
        text_width: u32,
        _n_spaces: u32,
        _carried_token: Option<Token>,
    ) -> (u32, Self::SpaceConfig) {
        (
            (max_width - text_width + 1) / 2,
            UniformSpaceConfig::new(renderer),
        )
    }
}

impl VerticalTextAlignment for CenterAligned {
    #[inline]
    fn apply_vertical_alignment<'a, F, A, H>(
        cursor: &mut Cursor,
        styled_text_box: &'a StyledTextBox<'a, F, A, Self, H>,
    ) where
        F: TextRenderer + CharacterStyle,
        <F as CharacterStyle>::Color: From<Rgb>,
        A: HorizontalTextAlignment,
        H: HeightMode,
    {
        let text_height = styled_text_box
            .style
            .measure_text_height(styled_text_box.text_box.text, cursor.line_width())
            as i32;

        let box_height = styled_text_box.bounding_box().size.height as i32;
        let offset = (box_height - text_height) / 2;

        cursor.position.y += offset;
    }
}

#[cfg(test)]
mod test_horizontal {
    use embedded_graphics::{
        geometry::Point,
        mock_display::MockDisplay,
        mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        prelude::Size,
        primitives::Rectangle,
        Drawable,
    };

    use crate::{
        alignment::CenterAligned, style::TextBoxStyleBuilder, utils::test::size_for, TextBox,
    };

    fn assert_rendered(text: &str, size: Size, pattern: &[&str]) {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(CenterAligned)
            .build();

        TextBox::new(text, Rectangle::new(Point::zero(), size))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(pattern);
    }

    #[test]
    fn simple_render() {
        assert_rendered(
            "word",
            size_for(Font6x9, 6, 1),
            &[
                "      ........................      ",
                "      ......................#.      ",
                "      ......................#.      ",
                "      #...#...##...#.#....###.      ",
                "      #.#.#..#..#..##.#..#..#.      ",
                "      #.#.#..#..#..#.....#..#.      ",
                "      .#.#....##...#......###.      ",
                "      ........................      ",
                "      ........................      ",
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
            .alignment(CenterAligned)
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
            "      #####       ",
            "      ## ##       ",
            "      # # #       ",
            "      # # #       ",
            "      ## ##       ",
            "      #####       ",
        ]);
    }

    #[test]
    fn simple_word_wrapping() {
        assert_rendered(
            "word wrapping",
            size_for(Font6x9, 9, 2),
            &[
                "               ........................               ",
                "               ......................#.               ",
                "               ......................#.               ",
                "               #...#...##...#.#....###.               ",
                "               #.#.#..#..#..##.#..#..#.               ",
                "               #.#.#..#..#..#.....#..#.               ",
                "               .#.#....##...#......###.               ",
                "               ........................               ",
                "               ........................               ",
                "   ................................................   ",
                "   ................................#...............   ",
                "   ................................................   ",
                "   #...#..#.#....###..###...###...##....###....##..   ",
                "   #.#.#..##.#..#..#..#..#..#..#...#....#..#..#..#.   ",
                "   #.#.#..#.....#..#..#..#..#..#...#....#..#..#..#.   ",
                "   .#.#...#......###..###...###...###...#..#...###.   ",
                "   ...................#.....#....................#.   ",
                "   ...................#.....#..................##..   ",
            ],
        );
    }

    #[test]
    fn word_longer_than_line_wraps_word() {
        assert_rendered(
            "word  somereallylongword",
            size_for(Font6x9, 9, 3),
            &[
                "               ........................               ",
                "               ......................#.               ",
                "               ......................#.               ",
                "               #...#...##...#.#....###.               ",
                "               #.#.#..#..#..##.#..#..#.               ",
                "               #.#.#..#..#..#.....#..#.               ",
                "               .#.#....##...#......###.               ",
                "               ........................               ",
                "               ........................               ",
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
    fn soft_hyphen_centering() {
        assert_rendered(
            "soft\u{AD}hyphen",
            size_for(Font6x9, 6, 2),
            &[
                "   ..............................   ",
                "   ...............#....#.........   ",
                "   ..............#.#...#.........   ",
                "   ..###...##....#....###........   ",
                "   .##....#..#..###....#...#####.   ",
                "   ...##..#..#...#.....#.#.......   ",
                "   .###....##....#......#........   ",
                "   ..............................   ",
                "   ..............................   ",
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

#[cfg(test)]
mod test_vertical {
    use embedded_graphics::{
        geometry::Point,
        mock_display::MockDisplay,
        mono_font::{ascii::Font6x9, MonoTextStyleBuilder},
        pixelcolor::BinaryColor,
        prelude::Size,
        primitives::Rectangle,
        Drawable,
    };

    use crate::{
        alignment::CenterAligned, style::TextBoxStyleBuilder, utils::test::size_for, TextBox,
    };

    fn assert_rendered(text: &str, size: Size, pattern: &[&str]) {
        let mut display = MockDisplay::new();

        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        let style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .vertical_alignment(CenterAligned)
            .build();

        TextBox::new(text, Rectangle::new(Point::zero(), size))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(pattern);
    }

    #[test]
    fn test_center_alignment() {
        assert_rendered(
            "word",
            size_for(Font6x9, 4, 2),
            &[
                "                        ",
                "                        ",
                "                        ",
                "                        ",
                "........................",
                "......................#.",
                "......................#.",
                "#...#...##...#.#....###.",
                "#.#.#..#..#..##.#..#..#.",
                "#.#.#..#..#..#.....#..#.",
                ".#.#....##...#......###.",
                "........................",
                "........................",
                "                        ",
                "                        ",
                "                        ",
                "                        ",
            ],
        );
    }
}
