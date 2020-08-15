//! Fully justified text.
use crate::{
    alignment::{HorizontalTextAlignment, VerticalTextAlignment},
    parser::{Parser, Token},
    rendering::{
        cursor::Cursor,
        line::{SpaceConfig, StyledLineIterator},
        StateFactory, StyledTextBoxIterator,
    },
    style::StyledTextBox,
    utils::font_ext::FontExt,
};
use embedded_graphics::{drawable::Pixel, fonts::Font, pixelcolor::PixelColor};

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
    fn default<F: Font>() -> Self {
        JustifiedSpaceConfig::new(F::total_char_width(' '), 0)
    }

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

/// State variable used by the fully justified text renderer.
#[derive(Debug)]
pub enum State<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Starts processing a line.
    NextLine(Option<Token<'a>>, Cursor<F>, Parser<'a>),

    /// Renders the processed line.
    DrawLine(StyledLineIterator<'a, C, F, JustifiedSpaceConfig, Justified>),
}

impl<'a, C, F, V> StateFactory<'a, F> for StyledTextBox<'a, C, F, Justified, V>
where
    C: PixelColor,
    F: Font + Copy,
    V: VerticalTextAlignment,
{
    type PixelIteratorState = State<'a, C, F>;

    #[inline]
    #[must_use]
    fn create_state(&self, cursor: Cursor<F>, parser: Parser<'a>) -> Self::PixelIteratorState {
        State::NextLine(None, cursor, parser)
    }
}

impl<C, F, V> Iterator for StyledTextBoxIterator<'_, C, F, Justified, V>
where
    C: PixelColor,
    F: Font + Copy,
    V: VerticalTextAlignment,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                State::NextLine(ref carried_token, ref cursor, ref mut parser) => {
                    if !cursor.in_display_area() {
                        break None;
                    }

                    if carried_token.is_none() && parser.is_empty() {
                        break None;
                    }

                    let parser_clone = parser.clone();
                    let max_line_width = cursor.line_width();
                    let (width, total_whitespace_count, t) =
                        self.style
                            .measure_line(parser, carried_token.clone(), max_line_width);

                    let space = max_line_width
                        - (width - total_whitespace_count * F::total_char_width(' '));
                    let stretch_line = t.is_some() && t != Some(Token::NewLine);

                    let space_info = if stretch_line && total_whitespace_count != 0 {
                        let space_width = space / total_whitespace_count;
                        let extra_pixels = space % total_whitespace_count;
                        JustifiedSpaceConfig::new(space_width, extra_pixels)
                    } else {
                        JustifiedSpaceConfig::default::<F>()
                    };

                    self.state = State::DrawLine(StyledLineIterator::new(
                        parser_clone,
                        *cursor,
                        space_info,
                        self.style.text_style,
                        carried_token.clone(),
                    ));
                }

                State::DrawLine(ref mut line_iterator) => {
                    if let pixel @ Some(_) = line_iterator.next() {
                        break pixel;
                    }

                    self.state = State::NextLine(
                        line_iterator.remaining_token(),
                        line_iterator.cursor,
                        line_iterator.parser.clone(),
                    );
                }
            }
        }
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
