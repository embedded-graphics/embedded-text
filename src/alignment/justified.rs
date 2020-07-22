//! Fully justified text
use crate::{
    alignment::TextAlignment,
    parser::Token,
    rendering::{
        character::StyledCharacterIterator, whitespace::EmptySpaceIterator, StateFactory,
        StyledTextBoxIterator,
    },
    style::StyledTextBox,
    utils::{font_ext::FontExt, rect_ext::RectExt},
};
use embedded_graphics::prelude::*;

use core::str::Chars;

/// Marks text to be rendered fully justified
#[derive(Copy, Clone, Debug)]
pub struct Justified;
impl TextAlignment for Justified {}

/// Internal state information used to store width of whitespace characters when rendering fully
/// justified text.
///
/// The fully justified renderer works by calculating the width of whitespace characters for the
/// current line. Due to integer arithmetic, there can be remainder pixels when a single space
/// width is used. This struct stores two width values so the whole line will always (at least if
/// there's a space in the line) take up all available space.
#[derive(Copy, Clone, Debug)]
pub struct SpaceInfo {
    /// The width of the first space_count whitespace characters
    pub space_width: u32,

    /// Stores how many characters are rendered using the space_width width. This field changes
    /// during rendering
    pub space_count: u32,

    /// Width of space characters after space_count number of spaces have been rendered
    pub remaining_space_width: u32,
}

impl SpaceInfo {
    #[inline]
    #[must_use]
    fn default<F: Font>() -> Self {
        SpaceInfo::new(F::char_width(' '), 0)
    }

    #[inline]
    #[must_use]
    fn new(space_width: u32, extra_pixel_count: u32) -> Self {
        SpaceInfo {
            space_width: space_width + 1,
            space_count: extra_pixel_count,
            remaining_space_width: space_width,
        }
    }

    #[inline]
    fn space_width(&mut self) -> u32 {
        if self.space_count == 0 {
            self.remaining_space_width
        } else {
            self.space_count -= 1;
            self.space_width
        }
    }

    #[inline]
    fn peek_space_width(&self, whitespace_count: u32) -> u32 {
        let above_limit = whitespace_count.saturating_sub(self.space_count);
        self.space_width * self.space_count + above_limit * self.remaining_space_width
    }
}

/// State variable used by the fully justified text renderer
#[derive(Debug)]
pub enum JustifiedState<'a, C, F>
where
    C: PixelColor,
    F: Font + Copy,
{
    /// This state processes the next token in the text.
    NextWord(bool, SpaceInfo),

    /// This state handles a line break after a newline character or word wrapping.
    LineBreak(Chars<'a>),

    /// This state measures the next line to calculate the position of the first word.
    MeasureLine(Chars<'a>),

    /// This state processes the next character in a word.
    DrawWord(Chars<'a>, SpaceInfo),

    /// This state renders a character, then passes the rest of the character iterator to DrawWord.
    DrawCharacter(Chars<'a>, StyledCharacterIterator<C, F>, SpaceInfo),

    /// This state renders whitespace.
    DrawWhitespace(u32, EmptySpaceIterator<C, F>, SpaceInfo),
}

impl<'a, C, F> StateFactory for StyledTextBox<'a, C, F, Justified>
where
    C: PixelColor,
    F: Font + Copy,
{
    type PixelIteratorState = JustifiedState<'a, C, F>;

    #[inline]
    #[must_use]
    fn create_state() -> Self::PixelIteratorState {
        JustifiedState::MeasureLine("".chars())
    }
}

impl<C, F> Iterator for StyledTextBoxIterator<'_, C, F, Justified>
where
    C: PixelColor,
    F: Font + Copy,
{
    type Item = Pixel<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.cursor.in_display_area() {
                break None;
            }

            match self.state {
                JustifiedState::LineBreak(ref remaining) => {
                    self.cursor.carriage_return();
                    self.cursor.new_line();
                    self.state = JustifiedState::MeasureLine(remaining.clone());
                }

                JustifiedState::MeasureLine(ref remaining) => {
                    let max_line_width = RectExt::size(self.cursor.bounds).width;

                    // initial width is the width of the characters carried over to this row
                    let (mut total_width, fits) = F::max_fitting(remaining.clone(), max_line_width);

                    let mut total_whitespace_count = 0;
                    let mut stretch_line = false;

                    // in some rare cases, the carried over text may not fit into a single line
                    if fits {
                        let mut last_whitespace_width = 0;
                        let mut last_whitespace_count = 0;
                        let mut total_whitespace_width = 0;

                        for token in self.parser.clone() {
                            if total_width >= max_line_width {
                                break;
                            }
                            match token {
                                Token::NewLine => {
                                    break;
                                }

                                Token::Whitespace(_) if total_width == 0 => {
                                    // eat spaces at the start of line
                                }

                                Token::Whitespace(n) => {
                                    last_whitespace_count = n;
                                    last_whitespace_width =
                                        (n * F::char_width(' ')).min(max_line_width - total_width);
                                }

                                Token::Word(w) => {
                                    let word_width = w.chars().map(F::char_width).sum::<u32>();
                                    let new_total_width = total_width + word_width;
                                    let new_whitespace_width =
                                        total_whitespace_width + last_whitespace_width;

                                    if new_whitespace_width + new_total_width > max_line_width {
                                        // including the word would wrap the line, stop here instead
                                        stretch_line = true;
                                        break;
                                    }

                                    total_width = new_total_width;
                                    total_whitespace_width = new_whitespace_width;
                                    total_whitespace_count += last_whitespace_count;

                                    last_whitespace_count = 0;
                                    last_whitespace_width = 0;
                                }
                            }
                        }
                    }

                    let space_info = if stretch_line && total_whitespace_count != 0 {
                        let total_space_width = max_line_width - total_width;
                        let space_width =
                            (total_space_width / total_whitespace_count).max(F::char_width(' '));
                        let extra_pixels = total_space_width - space_width * total_whitespace_count;
                        SpaceInfo::new(space_width, extra_pixels)
                    } else {
                        SpaceInfo::default::<F>()
                    };

                    self.state = if remaining.clone().next().is_none() {
                        JustifiedState::NextWord(true, space_info)
                    } else {
                        JustifiedState::DrawWord(remaining.clone(), space_info)
                    }
                }

                JustifiedState::NextWord(first_word, mut space_info) => {
                    if let Some(token) = self.parser.next() {
                        match token {
                            Token::Word(w) => {
                                // measure w to see if it fits in current line
                                if first_word
                                    || self
                                        .cursor
                                        .fits_in_line(w.chars().map(F::char_width).sum::<u32>())
                                {
                                    self.state = JustifiedState::DrawWord(w.chars(), space_info);
                                } else {
                                    self.state = JustifiedState::LineBreak(w.chars());
                                }
                            }

                            Token::Whitespace(n) => {
                                // TODO character spacing!
                                // word wrapping, also applied for whitespace sequences
                                let mut lookahead = self.parser.clone();
                                if let Some(Token::Word(w)) = lookahead.next() {
                                    // only render whitespace if next is word and next doesn't wrap
                                    let n_width = w.chars().map(F::char_width).sum::<u32>()
                                        + space_info.peek_space_width(n);

                                    let pos = self.cursor.position;
                                    self.state = if self.cursor.fits_in_line(n_width) {
                                        let width = space_info.space_width();
                                        self.cursor.advance(width);
                                        JustifiedState::DrawWhitespace(
                                            n - 1,
                                            EmptySpaceIterator::new(
                                                width,
                                                pos,
                                                self.style.text_style,
                                            ),
                                            space_info,
                                        )
                                    } else {
                                        JustifiedState::NextWord(first_word, space_info)
                                    }
                                } else {
                                    // don't render
                                }
                            }

                            Token::NewLine => {
                                self.state = JustifiedState::LineBreak("".chars());
                            }
                        }
                    } else {
                        break None;
                    }
                }

                JustifiedState::DrawWord(ref mut chars_iterator, space_info) => {
                    let mut copy = chars_iterator.clone();
                    self.state = if let Some(c) = copy.next() {
                        // TODO character spacing!
                        let current_pos = self.cursor.position;

                        if self.cursor.advance_char(c) {
                            JustifiedState::DrawCharacter(
                                copy,
                                StyledCharacterIterator::new(c, current_pos, self.style.text_style),
                                space_info,
                            )
                        } else {
                            // word wrapping
                            JustifiedState::LineBreak(chars_iterator.clone())
                        }
                    } else {
                        JustifiedState::NextWord(false, space_info)
                    }
                }

                JustifiedState::DrawWhitespace(n, ref mut iterator, mut space_info) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    let width = space_info.space_width();
                    let pos = self.cursor.position;
                    self.state = if n == 0 {
                        // no more spaces to draw
                        JustifiedState::NextWord(false, space_info)
                    } else if self.cursor.advance(width) {
                        // draw next space
                        JustifiedState::DrawWhitespace(
                            n - 1,
                            EmptySpaceIterator::new(width, pos, self.style.text_style),
                            space_info,
                        )
                    } else {
                        // word wrapping, also applied for whitespace sequences
                        // eat the spaces from the start of next line
                        JustifiedState::LineBreak("".chars())
                    }
                }

                JustifiedState::DrawCharacter(ref chars_iterator, ref mut iterator, space_info) => {
                    if let pixel @ Some(_) = iterator.next() {
                        break pixel;
                    }

                    self.state = JustifiedState::DrawWord(chars_iterator.clone(), space_info);
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

        TextBox::new("word", Rectangle::new(Point::zero(), Point::new(54, 54)))
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
    fn simple_word_wrapping() {
        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(Justified)
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
            Rectangle::new(Point::zero(), Point::new(60, 54)),
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
            Rectangle::new(Point::zero(), Point::new(54, 54)),
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
            Rectangle::new(Point::zero(), Point::new(54, 54)),
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
