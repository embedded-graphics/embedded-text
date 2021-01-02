//! Right aligned text.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::Token,
    rendering::{space_config::UniformSpaceConfig, RendererFactory},
    style::{color::Rgb, height_mode::HeightMode},
    StyledTextBox,
};
use embedded_graphics::prelude::MonoFont;
use embedded_graphics_core::pixelcolor::PixelColor;

/// Marks text to be rendered right aligned.
#[derive(Copy, Clone, Debug)]
pub struct RightAligned;
impl HorizontalTextAlignment for RightAligned {
    const STARTING_SPACES: bool = false;
    const ENDING_SPACES: bool = false;
}

impl<'a, C, F, V, H> RendererFactory for StyledTextBox<'a, C, F, RightAligned, V, H>
where
    C: PixelColor + From<Rgb>,
    F: MonoFont,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type SpaceConfig = UniformSpaceConfig<F>;

    fn place_line(
        max_width: u32,
        width: u32,
        _n_spaces: u32,
        _carried_token: Option<Token>,
    ) -> (u32, Self::SpaceConfig) {
        (max_width - width, UniformSpaceConfig::default())
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        primitives::Rectangle,
    };

    use crate::{alignment::RightAligned, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn simple_render() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new("word", Rectangle::new(Point::zero(), Size::new(55, 8)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "                               ......................#.",
            "                               ......................#.",
            "                               #...#..###..#.##...##.#.",
            "                               #...#.#...#.##..#.#..##.",
            "                               #.#.#.#...#.#.....#...#.",
            "                               #.#.#.#...#.#.....#...#.",
            "                               .#.#...###..#......####.",
            "                               ........................",
        ]);
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);

        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new("O\rX", Rectangle::new(Point::zero(), Size::new(55, 8)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "                                                 ##### ",
            "                                                 #   # ",
            "                                                 ## ## ",
            "                                                 # # # ",
            "                                                 ## ## ",
            "                                                 #   # ",
            "                                                 ##### ",
        ]);
    }

    #[test]
    fn simple_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
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
            "                               ......................#.",
            "                               ......................#.",
            "                               #...#..###..#.##...##.#.",
            "                               #...#.#...#.##..#.#..##.",
            "                               #.#.#.#...#.#.....#...#.",
            "                               #.#.#.#...#.#.....#...#.",
            "                               .#.#...###..#......####.",
            "                               ........................",
            "       ................................#...............",
            "       ................................................",
            "       #...#.#.##...###..####..####...##...#.##...####.",
            "       #...#.##..#.....#.#...#.#...#...#...##..#.#...#.",
            "       #.#.#.#......####.#...#.#...#...#...#...#.#...#.",
            "       #.#.#.#.....#...#.####..####....#...#...#..####.",
            "       .#.#..#......####.#.....#......###..#...#.....#.",
            "       ..................#.....#..................###..",
        ]);
    }

    #[test]
    fn word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
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
            "                               ......................#.",
            "                               ......................#.",
            "                               #...#..###..#.##...##.#.",
            "                               #...#.#...#.##..#.#..##.",
            "                               #.#.#.#...#.#.....#...#.",
            "                               #.#.#.#...#.#.....#...#.",
            "                               .#.#...###..#......####.",
            "                               ........................",
            " ...........................................##....##...",
            " ............................................#.....#...",
            " .####..###..##.#...###..#.##...###...###....#.....#...",
            " #.....#...#.#.#.#.#...#.##..#.#...#.....#...#.....#...",
            " .###..#...#.#...#.#####.#.....#####..####...#.....#...",
            " ....#.#...#.#...#.#.....#.....#.....#...#...#.....#...",
            " ####...###..#...#..###..#......###...####..###...###..",
            " ......................................................",
            " .......##...........................................#.",
            " ........#...........................................#.",
            " #...#...#....###..#.##...####.#...#..###..#.##...##.#.",
            " #...#...#...#...#.##..#.#...#.#...#.#...#.##..#.#..##.",
            " #...#...#...#...#.#...#.#...#.#.#.#.#...#.#.....#...#.",
            " .####...#...#...#.#...#..####.#.#.#.#...#.#.....#...#.",
            " ....#..###...###..#...#.....#..#.#...###..#......####.",
            " .###.....................###..........................",
        ]);
    }

    #[test]
    fn first_word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
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
            " ...........................................##....##...",
            " ............................................#.....#...",
            " .####..###..##.#...###..#.##...###...###....#.....#...",
            " #.....#...#.#.#.#.#...#.##..#.#...#.....#...#.....#...",
            " .###..#...#.#...#.#####.#.....#####..####...#.....#...",
            " ....#.#...#.#...#.#.....#.....#.....#...#...#.....#...",
            " ####...###..#...#..###..#......###...####..###...###..",
            " ......................................................",
            " .......##...........................................#.",
            " ........#...........................................#.",
            " #...#...#....###..#.##...####.#...#..###..#.##...##.#.",
            " #...#...#...#...#.##..#.#...#.#...#.#...#.##..#.#..##.",
            " #...#...#...#...#.#...#.#...#.#.#.#.#...#.#.....#...#.",
            " .####...#...#...#.#...#..####.#.#.#.#...#.#.....#...#.",
            " ....#..###...###..#...#.....#..#.#...###..#......####.",
            " .###.....................###..........................",
        ]);
    }

    #[test]
    fn soft_hyphen_rendering() {
        let text = "soft\u{AD}hyphen";

        let mut display = MockDisplay::new();

        let bounds = Rectangle::new(Point::new(0, 0), Size::new(36, 31));
        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new(text, bounds)
            .into_styled(textbox_style)
            .draw(&mut display)
            .unwrap();

        display.assert_pattern(&[
            "                    ##   #         ",
            "                   #  #  #         ",
            "       ####  ###   #    ###        ",
            "      #     #   # ###    #    #####",
            "       ###  #   #  #     #         ",
            "          # #   #  #     #  #      ",
            "      ####   ###   #      ##       ",
            "                                   ",
            "#                 #                ",
            "#                 #                ",
            "# ##  #   # ####  # ##   ###  # ## ",
            "##  # #   # #   # ##  # #   # ##  #",
            "#   # #   # #   # #   # ##### #   #",
            "#   #  #### ####  #   # #     #   #",
            "#   #     # #     #   #  ###  #   #",
            "       ###  #                      ",
        ]);
    }
}
