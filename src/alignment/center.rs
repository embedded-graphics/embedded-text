//! Center aligned text.
use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{
        cursor::Cursor,
        line::{StyledLineIterator, UniformSpaceConfig},
        StateFactory, StyledTextBoxIterator,
    },
    style::StyledTextBox,
    utils::font_ext::{FontExt, LineMeasurement},
};
use embedded_graphics::{drawable::Pixel, fonts::Font, pixelcolor::PixelColor};

/// Marks text to be rendered center aligned.
#[derive(Copy, Clone, Debug)]
pub struct CenterAligned;
impl TextAlignment for CenterAligned {}

/// State variable used by the center aligned text renderer.
#[derive(Debug)]
pub enum CenterAlignedState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Starts processing a line.
    NextLine(Option<Token<'a>>, Cursor<F>),

    /// Renders the processed line.
    DrawLine(StyledLineIterator<'a, C, F, UniformSpaceConfig>),
}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, CenterAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = CenterAlignedState<'a, C, F>;

    #[inline]
    #[must_use]
    fn create_state(&self) -> Self::PixelIteratorState {
        CenterAlignedState::NextLine(None, Cursor::new(self.text_box.bounds))
    }
}

impl<C, F> Iterator for StyledTextBoxIterator<'_, C, F, CenterAligned>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                CenterAlignedState::NextLine(ref carried_token, ref mut cursor) => {
                    if !cursor.in_display_area() {
                        break None;
                    }

                    if carried_token.is_none() && self.parser.is_empty() {
                        break None;
                    }

                    let max_line_width = cursor.line_width();

                    // initial width is the width of the characters carried over to this row
                    let measurement = if let Some(Token::Word(ref w)) = carried_token {
                        F::measure_line(w, max_line_width)
                    } else {
                        LineMeasurement::empty()
                    };

                    let mut space = max_line_width - measurement.width;

                    // in some rare cases, the carried over text may not fit into a single line
                    if measurement.fits_line {
                        let mut last_whitespace_width = 0;

                        for token in self.parser.clone() {
                            match token {
                                Token::NewLine => {
                                    break;
                                }

                                Token::Whitespace(_) if space == max_line_width => {
                                    // eat spaces at the start of line
                                }

                                Token::Whitespace(n) => {
                                    last_whitespace_width =
                                        (n * F::total_char_width(' ')).min(space);
                                }

                                Token::Word(w) => {
                                    let space_with_last_ws = space - last_whitespace_width;
                                    let word_measurement = F::measure_line(w, space_with_last_ws);

                                    if !word_measurement.fits_line {
                                        if space == max_line_width {
                                            space -= F::measure_line(w, max_line_width).width;
                                        }
                                        break;
                                    }

                                    space = space_with_last_ws - word_measurement.width;
                                }
                            }
                        }
                    }

                    cursor.advance((space + 1) / 2);

                    self.state = CenterAlignedState::DrawLine(StyledLineIterator::new(
                        self.parser.clone(),
                        *cursor,
                        UniformSpaceConfig {
                            starting_spaces: false,
                            ending_spaces: false,
                            space_width: F::total_char_width(' '),
                        },
                        self.style.text_style,
                        carried_token.clone(),
                    ));
                }

                CenterAlignedState::DrawLine(ref mut line_iterator) => {
                    if let pixel @ Some(_) = line_iterator.next() {
                        break pixel;
                    }

                    self.parser = line_iterator.parser.clone();
                    let carried_token = line_iterator.remaining_token();
                    let mut cursor = line_iterator.cursor;
                    cursor.new_line();
                    cursor.carriage_return();
                    self.state = CenterAlignedState::NextLine(carried_token, cursor);
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

    use crate::{alignment::CenterAligned, style::TextBoxStyleBuilder, TextBox};

    #[test]
    fn simple_render() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new("word", Rectangle::new(Point::zero(), Point::new(54, 54)))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                ......................#.               ",
                "                ......................#.               ",
                "                #...#..###..#.##...##.#.               ",
                "                #...#.#...#.##..#.#..##.               ",
                "                #.#.#.#...#.#.....#...#.               ",
                "                #.#.#.#...#.#.....#...#.               ",
                "                .#.#...###..#......####.               ",
                "                ........................               ",
            ])
        );
    }

    #[test]
    fn simple_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word wrapping",
            Rectangle::new(Point::zero(), Point::new(54, 54)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                ......................#.               ",
                "                ......................#.               ",
                "                #...#..###..#.##...##.#.               ",
                "                #...#.#...#.##..#.#..##.               ",
                "                #.#.#.#...#.#.....#...#.               ",
                "                #.#.#.#...#.#.....#...#.               ",
                "                .#.#...###..#......####.               ",
                "                ........................               ",
                "    ................................#...............   ",
                "    ................................................   ",
                "    #...#.#.##...###..####..####...##...#.##...####.   ",
                "    #...#.##..#.....#.#...#.#...#...#...##..#.#...#.   ",
                "    #.#.#.#......####.#...#.#...#...#...#...#.#...#.   ",
                "    #.#.#.#.....#...#.####..####....#...#...#..####.   ",
                "    .#.#..#......####.#.....#......###..#...#.....#.   ",
                "    ..................#.....#..................###..   "
            ])
        );
    }

    #[test]
    fn word_longer_than_line_wraps_word() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "word somereallylongword",
            Rectangle::new(Point::zero(), Point::new(54, 54)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                ......................#.               ",
                "                ......................#.               ",
                "                #...#..###..#.##...##.#.               ",
                "                #...#.#...#.##..#.#..##.               ",
                "                #.#.#.#...#.#.....#...#.               ",
                "                #.#.#.#...#.#.....#...#.               ",
                "                .#.#...###..#......####.               ",
                "                ........................               ",
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
            .alignment(CenterAligned)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        TextBox::new(
            "somereallylongword",
            Rectangle::new(Point::zero(), Point::new(54, 54)),
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
