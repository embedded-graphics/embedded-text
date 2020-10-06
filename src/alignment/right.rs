//! Right aligned text.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    rendering::{
        line::StyledLinePixelIterator, space_config::UniformSpaceConfig, RendererFactory,
        StyledTextBoxIterator,
    },
    style::{color::Rgb, height_mode::HeightMode},
    StyledTextBox,
};
use embedded_graphics::{fonts::Font, pixelcolor::PixelColor};

/// Marks text to be rendered right aligned.
#[derive(Copy, Clone, Debug)]
pub struct RightAligned;
impl HorizontalTextAlignment for RightAligned {
    const STARTING_SPACES: bool = false;
    const ENDING_SPACES: bool = false;
}

impl<'a, C, F, V, H> RendererFactory<'a, C> for StyledTextBox<'a, C, F, RightAligned, V, H>
where
    C: PixelColor + From<Rgb>,
    F: Font + Copy,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Renderer = StyledTextBoxIterator<'a, C, F, RightAligned, V, H, UniformSpaceConfig<F>>;

    #[inline]
    #[must_use]
    fn create_renderer(&self) -> Self::Renderer {
        StyledTextBoxIterator::new(self, |style, carried, mut cursor, parser| {
            let max_line_width = cursor.line_width();
            let (width, _, _, _) =
                style.measure_line(&mut parser.clone(), carried.clone(), max_line_width);
            cursor.advance_unchecked(max_line_width - width);

            StyledLinePixelIterator::new(
                parser,
                cursor,
                UniformSpaceConfig::default(),
                style,
                carried,
            )
        })
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

        TextBox::new("word", Rectangle::new(Point::zero(), Point::new(54, 7)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                               ......................#.",
                "                               ......................#.",
                "                               #...#..###..#.##...##.#.",
                "                               #...#.#...#.##..#.#..##.",
                "                               #.#.#.#...#.#.....#...#.",
                "                               #.#.#.#...#.#.....#...#.",
                "                               .#.#...###..#......####.",
                "                               ........................",
            ])
        );
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(RightAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new("O\rX", Rectangle::new(Point::zero(), Point::new(54, 7)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                                                 ##### ",
                "                                                 #   # ",
                "                                                 ## ## ",
                "                                                 # # # ",
                "                                                 ## ## ",
                "                                                 #   # ",
                "                                                 ##### ",
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(54, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
                "       ..................#.....#..................###.."
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(54, 23)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(54, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
            ])
        );
    }
}
