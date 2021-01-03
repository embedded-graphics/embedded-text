//! Fully justified text.
use crate::{
    alignment::HorizontalTextAlignment, parser::Token, rendering::space_config::SpaceConfig,
};
use embedded_graphics::fonts::MonoFont;

/// Marks text to be rendered fully justified.
#[derive(Copy, Clone, Debug)]
pub struct Justified;
impl HorizontalTextAlignment for Justified {
    type SpaceConfig = JustifiedSpaceConfig;

    const STARTING_SPACES: bool = false;
    const ENDING_SPACES: bool = false;

    #[inline]
    fn place_line<F: MonoFont>(
        max_width: u32,
        text_width: u32,
        n_spaces: u32,
        carried_token: Option<Token>,
    ) -> (u32, Self::SpaceConfig) {
        let space =
            max_width - (text_width - n_spaces * F::CHARACTER_SIZE.width + F::CHARACTER_SPACING);
        let stretch_line = carried_token.is_some() && carried_token != Some(Token::NewLine);

        let space_info = if stretch_line && n_spaces != 0 {
            let space_width = space / n_spaces;
            let extra_pixels = space % n_spaces;
            JustifiedSpaceConfig::new(space_width, extra_pixels)
        } else {
            JustifiedSpaceConfig::new(F::CHARACTER_SIZE.width + F::CHARACTER_SPACING, 0)
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
    fn new(space_width: u32, extra_pixel_count: u32) -> Self {
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
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    };
    use embedded_graphics_core::primitives::Rectangle;

    use crate::{alignment::Justified, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn simple_render() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new("word", Rectangle::new(Point::zero(), Size::new(55, 8)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "......................#.",
            "......................#.",
            "#...#..###..#.##...##.#.",
            "#...#.#...#.##..#.#..##.",
            "#.#.#.#...#.#.....#...#.",
            "#.#.#.#...#.#.....#...#.",
            ".#.#...###..#......####.",
            "........................",
        ]);
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new("O\rX", Rectangle::new(Point::zero(), Size::new(55, 8)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "#####    ",
            "#   #    ",
            "## ##    ",
            "# # #    ",
            "## ##    ",
            "#   #    ",
            "#####    ",
        ]);
    }

    #[test]
    fn wrapping_when_space_is_less_than_space_character() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new("A word", Rectangle::new(Point::zero(), Size::new(6 * 5, 8)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            ".###..            ",
            "#...#.            ",
            "#...#.            ",
            "#####.            ",
            "#...#.            ",
            "#...#.            ",
            "#...#.            ",
            "......            ",
        ]);
    }

    #[test]
    fn simple_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word wrapping",
            Rectangle::new(Point::zero(), Size::new(55, 16)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "......................#.                        ",
            "......................#.                        ",
            "#...#..###..#.##...##.#.                        ",
            "#...#.#...#.##..#.#..##.                        ",
            "#.#.#.#...#.#.....#...#.                        ",
            "#.#.#.#...#.#.....#...#.                        ",
            ".#.#...###..#......####.                        ",
            "........................                        ",
            "................................#...............",
            "................................................",
            "#...#.#.##...###..####..####...##...#.##...####.",
            "#...#.##..#.....#.#...#.#...#...#...##..#.#...#.",
            "#.#.#.#......####.#...#.#...#...#...#...#.#...#.",
            "#.#.#.#.....#...#.####..####....#...#...#..####.",
            ".#.#..#......####.#.....#......###..#...#.....#.",
            "..................#.....#..................###..",
        ]);
    }

    #[test]
    fn justified_alignment() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word and other word last line",
            Rectangle::new(Point::zero(), Size::new(61, 24)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "......................#....................................#.",
            "......................#....................................#.",
            "#...#..###..#.##...##.#.....................###..#.##...##.#.",
            "#...#.#...#.##..#.#..##........................#.##..#.#..##.",
            "#.#.#.#...#.#.....#...#.....................####.#...#.#...#.",
            "#.#.#.#...#.#.....#...#....................#...#.#...#.#...#.",
            ".#.#...###..#......####.....................####.#...#..####.",
            ".............................................................",
            ".......#....#..............................................#.",
            ".......#....#..............................................#.",
            ".###..###...#.##...###..#.##.........#...#..###..#.##...##.#.",
            "#...#..#....##..#.#...#.##..#........#...#.#...#.##..#.#..##.",
            "#...#..#....#...#.#####.#............#.#.#.#...#.#.....#...#.",
            "#...#..#..#.#...#.#.....#............#.#.#.#...#.#.....#...#.",
            ".###....##..#...#..###..#.............#.#...###..#......####.",
            ".............................................................",
            ".##................#...........##.....#...............       ",
            "..#................#............#.....................       ",
            "..#....###...####.###...........#....##...#.##...###..       ",
            "..#.......#.#......#............#.....#...##..#.#...#.       ",
            "..#....####..###...#............#.....#...#...#.#####.       ",
            "..#...#...#.....#..#..#.........#.....#...#...#.#.....       ",
            ".###...####.####....##.........###...###..#...#..###..       ",
            "......................................................       ",
        ]);
    }

    #[test]
    fn word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word somereallylongword",
            Rectangle::new(Point::zero(), Size::new(55, 24)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "......................#.                              ",
            "......................#.                              ",
            "#...#..###..#.##...##.#.                              ",
            "#...#.#...#.##..#.#..##.                              ",
            "#.#.#.#...#.#.....#...#.                              ",
            "#.#.#.#...#.#.....#...#.                              ",
            ".#.#...###..#......####.                              ",
            "........................                              ",
            "...........................................##....##...",
            "............................................#.....#...",
            ".####..###..##.#...###..#.##...###...###....#.....#...",
            "#.....#...#.#.#.#.#...#.##..#.#...#.....#...#.....#...",
            ".###..#...#.#...#.#####.#.....#####..####...#.....#...",
            "....#.#...#.#...#.#.....#.....#.....#...#...#.....#...",
            "####...###..#...#..###..#......###...####..###...###..",
            "......................................................",
            ".......##...........................................#.",
            "........#...........................................#.",
            "#...#...#....###..#.##...####.#...#..###..#.##...##.#.",
            "#...#...#...#...#.##..#.#...#.#...#.#...#.##..#.#..##.",
            "#...#...#...#...#.#...#.#...#.#.#.#.#...#.#.....#...#.",
            ".####...#...#...#.#...#..####.#.#.#.#...#.#.....#...#.",
            "....#..###...###..#...#.....#..#.#...###..#......####.",
            ".###.....................###..........................",
        ]);
    }

    #[test]
    fn first_word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "somereallylongword",
            Rectangle::new(Point::zero(), Size::new(55, 16)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        display.assert_pattern(&[
            "...........................................##....##...",
            "............................................#.....#...",
            ".####..###..##.#...###..#.##...###...###....#.....#...",
            "#.....#...#.#.#.#.#...#.##..#.#...#.....#...#.....#...",
            ".###..#...#.#...#.#####.#.....#####..####...#.....#...",
            "....#.#...#.#...#.#.....#.....#.....#...#...#.....#...",
            "####...###..#...#..###..#......###...####..###...###..",
            "......................................................",
            ".......##...........................................#.",
            "........#...........................................#.",
            "#...#...#....###..#.##...####.#...#..###..#.##...##.#.",
            "#...#...#...#...#.##..#.#...#.#...#.#...#.##..#.#..##.",
            "#...#...#...#...#.#...#.#...#.#.#.#.#...#.#.....#...#.",
            ".####...#...#...#.#...#..####.#.#.#.#...#.#.....#...#.",
            "....#..###...###..#...#.....#..#.#...###..#......####.",
            ".###.....................###..........................",
        ]);
    }

    #[test]
    fn soft_hyphen_rendering() {
        let text = "soft\u{AD}hyphen";

        let mut display = MockDisplay::new();

        let bounds = Rectangle::new(Point::new(0, 0), Size::new(36, 31));
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new(text, bounds)
            .into_styled(textbox_style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "              ##   #                 ",
            "             #  #  #                 ",
            " ####  ###   #    ###                ",
            "#     #   # ###    #    #####        ",
            " ###  #   #  #     #                 ",
            "    # #   #  #     #  #              ",
            "####   ###   #      ##               ",
            "                                     ",
            "#                 #                  ",
            "#                 #                  ",
            "# ##  #   # ####  # ##   ###  # ##   ",
            "##  # #   # #   # ##  # #   # ##  #  ",
            "#   # #   # #   # #   # ##### #   #  ",
            "#   #  #### ####  #   # #     #   #  ",
            "#   #     # #     #   #  ###  #   #  ",
            "       ###  #                        ",
        ]);
    }

    #[test]
    fn tab_rendering() {
        // Expect \t to render as 3 space characters, ignored by the justified alignment.
        let text = "a\ttab + two te xt words";

        let mut display = MockDisplay::new();

        let bounds = Rectangle::new(Point::new(0, 0), Size::new(60, 31));
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new(text, bounds)
            .into_styled(textbox_style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "                         #          #                        ",
            "                         #          #                   #    ",
            " ###                    ###    ###  # ##                #    ",
            "    #                    #        # ##  #             #####  ",
            " ####                    #     #### #   #               #    ",
            "#   #                    #  # #   # #   #               #    ",
            " ####                     ##   #### ####                     ",
            "                                                             ",
            " #                          #                          #     ",
            " #                          #                          #     ",
            "###   #   #  ###           ###    ###           #   # ###    ",
            " #    #   # #   #           #    #   #           # #   #     ",
            " #    # # # #   #           #    #####            #    #     ",
            " #  # # # # #   #           #  # #               # #   #  #  ",
            "  ##   # #   ###             ##   ###           #   #   ##   ",
            "                                                             ",
            "                      #                                      ",
            "                      #                                      ",
            "#   #  ###  # ##   ## #  ####                                ",
            "#   # #   # ##  # #  ## #                                    ",
            "# # # #   # #     #   #  ###                                 ",
            "# # # #   # #     #   #     #                                ",
            " # #   ###  #      #### ####                                 ",
        ]);
    }
}
