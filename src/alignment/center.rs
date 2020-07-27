//! Center aligned text
use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{
        line::{LineConfiguration, StyledLineIterator, UniformSpaceConfig},
        StateFactory, StyledTextBoxIterator,
    },
    style::StyledTextBox,
    utils::{
        font_ext::{FontExt, LineMeasurement},
        rect_ext::RectExt,
    },
};
use embedded_graphics::{drawable::Pixel, fonts::Font, pixelcolor::PixelColor};

/// Marks text to be rendered center aligned
#[derive(Copy, Clone, Debug)]
pub struct CenterAligned;
impl TextAlignment for CenterAligned {}

/// State variable used by the center aligned text renderer
#[derive(Debug)]
pub enum CenterAlignedState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// Starts processing a line
    NextLine(Option<Token<'a>>),

    /// Renders the processed line
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
    fn create_state() -> Self::PixelIteratorState {
        CenterAlignedState::NextLine(None)
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
                CenterAlignedState::NextLine(ref carried_token) => {
                    if !self.cursor.in_display_area() {
                        break None;
                    }

                    if self.parser.peek().is_none() && carried_token.is_none() {
                        break None;
                    }

                    let max_line_width = self.cursor.bounds.size().width;

                    // initial width is the width of the characters carried over to this row
                    let measurement = if let Some(Token::Word(w)) = carried_token.clone() {
                        F::measure_line(w.chars(), max_line_width)
                    } else {
                        LineMeasurement::empty()
                    };

                    let mut total_width = measurement.width;

                    // in some rare cases, the carried over text may not fit into a single line
                    if measurement.fits_line {
                        let mut last_whitespace_width = 0;
                        let mut first_word = true;

                        for token in self.parser.clone() {
                            match token {
                                Token::NewLine => {
                                    break;
                                }

                                Token::Whitespace(_) if total_width == 0 => {
                                    // eat spaces at the start of line
                                }

                                Token::Whitespace(n) => {
                                    last_whitespace_width = (n * F::total_char_width(' '))
                                        .min(max_line_width - total_width);
                                }

                                Token::Word(w) => {
                                    let word_measurement = F::measure_line(
                                        w.chars(),
                                        max_line_width - total_width - last_whitespace_width,
                                    );
                                    if word_measurement.fits_line {
                                        total_width +=
                                            last_whitespace_width + word_measurement.width;
                                        last_whitespace_width = 0;
                                        first_word = false;
                                    } else {
                                        if first_word {
                                            total_width =
                                                F::measure_line(w.chars(), max_line_width).width;
                                        }
                                        break;
                                    }
                                }
                            }
                            if total_width >= max_line_width {
                                break;
                            }
                        }
                    }

                    self.cursor.carriage_return();
                    self.cursor.advance((max_line_width - total_width + 1) / 2);

                    self.state = CenterAlignedState::DrawLine(StyledLineIterator::new(
                        self.parser.clone(),
                        self.cursor.position,
                        self.cursor.bounds.size().width,
                        LineConfiguration {
                            starting_spaces: false,
                            ending_spaces: false,
                            space_config: UniformSpaceConfig(F::total_char_width(' ')),
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
                    self.state = CenterAlignedState::NextLine(line_iterator.remaining_token());
                    self.cursor.new_line();
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
