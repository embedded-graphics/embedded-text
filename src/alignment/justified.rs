//! Fully justified text.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::Token,
    rendering::{
        line::StyledLinePixelIterator, space_config::SpaceConfig, RendererFactory,
        StyledTextBoxIterator,
    },
    style::{color::Rgb, height_mode::HeightMode},
    utils::font_ext::FontExt,
    StyledTextBox,
};
use core::marker::PhantomData;
use embedded_graphics::{fonts::Font, pixelcolor::PixelColor};

/// Marks text to be rendered fully justified.
#[derive(Copy, Clone, Debug)]
pub struct Justified;
impl HorizontalTextAlignment for Justified {
    const STARTING_SPACES: bool = false;
    const ENDING_SPACES: bool = false;
}

/// Internal state information used to store width of whitespace characters when rendering fully
/// justified text.
///
/// The fully justified renderer works by calculating the width of whitespace characters for the
/// current line. Due to integer arithmetic, there can be remainder pixels when a single space
/// width is used. This struct stores two width values so the whole line will always (at least if
/// there's a space in the line) take up all available space.
#[derive(Copy, Clone, Debug)]
pub struct JustifiedSpaceConfig<F: Font + Copy> {
    _font: PhantomData<F>,

    /// The width of the whitespace characters.
    space_width: u32,

    /// Stores how many characters are rendered using the space_width width. This field changes
    /// during rendering.
    space_count: u32,
}

impl<F: Font + Copy> JustifiedSpaceConfig<F> {
    #[inline]
    #[must_use]
    fn new(space_width: u32, extra_pixel_count: u32) -> Self {
        JustifiedSpaceConfig {
            _font: PhantomData,
            space_width,
            space_count: extra_pixel_count,
        }
    }
}

impl<F: Font + Copy> Default for JustifiedSpaceConfig<F> {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new(F::total_char_width(' '), 0)
    }
}

impl<F: Font + Copy> SpaceConfig for JustifiedSpaceConfig<F> {
    type Font = F;

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

impl<'a, C, F, V, H> RendererFactory<'a, C> for StyledTextBox<'a, C, F, Justified, V, H>
where
    C: PixelColor + From<Rgb>,
    F: Font + Copy,
    V: VerticalTextAlignment,
    H: HeightMode,
{
    type Renderer = StyledTextBoxIterator<'a, C, F, Justified, V, H, JustifiedSpaceConfig<F>>;

    #[inline]
    #[must_use]
    fn create_renderer(&self) -> Self::Renderer {
        StyledTextBoxIterator::new(self, |style, carried, cursor, parser| {
            let max_line_width = cursor.line_width();
            let (width, total_whitespace_count, t) =
                style.measure_line(&mut parser.clone(), carried.clone(), max_line_width);

            let space =
                max_line_width - (width - total_whitespace_count * F::total_char_width(' '));
            let stretch_line = t.is_some() && t != Some(Token::NewLine);

            let space_info = if stretch_line && total_whitespace_count != 0 {
                let space_width = space / total_whitespace_count;
                let extra_pixels = space % total_whitespace_count;
                JustifiedSpaceConfig::new(space_width, extra_pixels)
            } else {
                JustifiedSpaceConfig::default()
            };

            StyledLinePixelIterator::new(parser, cursor, space_info, style, carried)
        })
    }
}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        primitives::Rectangle,
    };

    use crate::{alignment::Justified, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn simple_render() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
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
                "......................#.",
                "......................#.",
                "#...#..###..#.##...##.#.",
                "#...#.#...#.##..#.#..##.",
                "#.#.#.#...#.#.....#...#.",
                "#.#.#.#...#.#.....#...#.",
                ".#.#...###..#......####.",
                "........................",
            ])
        );
    }

    #[test]
    fn simple_render_cr() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new("O\rX", Rectangle::new(Point::zero(), Point::new(54, 7)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "#####    ",
                "#   #    ",
                "## ##    ",
                "# # #    ",
                "## ##    ",
                "#   #    ",
                "#####    ",
            ])
        );
    }

    #[test]
    fn wrapping_when_space_is_less_than_space_character() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "A word",
            Rectangle::new(Point::zero(), Point::new(6 * 5 - 1, 7)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                ".###..            ",
                "#...#.            ",
                "#...#.            ",
                "#####.            ",
                "#...#.            ",
                "#...#.            ",
                "#...#.            ",
                "......            ",
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(54, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
                "..................#.....#..................###.."
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(60, 23)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
                "......................................................       "
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(54, 23)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
            ])
        );
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
            Rectangle::new(Point::zero(), Point::new(54, 15)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
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
            ])
        );
    }
}
