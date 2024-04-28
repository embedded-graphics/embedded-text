//! Misc utilities

use embedded_graphics::{
    prelude::Point,
    text::{renderer::TextRenderer, Baseline},
};

/// Measure the width of a piece of string.
pub fn str_width(renderer: &impl TextRenderer, s: &str) -> u32 {
    renderer
        .measure_string(s, Point::zero(), Baseline::Top)
        .next_position
        .x as u32
}

/// Measure the width of a piece of string and the offset between
/// the left edge of the bounding box and the left edge of the text.
///
/// The offset is particularly useful when the first glyph on
/// the line has a negative left side bearing.
pub fn str_width_and_left_offset(renderer: &impl TextRenderer, s: &str) -> (u32, u32) {
    let tm = renderer.measure_string(s, Point::zero(), Baseline::Top);
    (
        tm.next_position.x as u32,
        tm.bounding_box.top_left.x.min(0).abs() as u32,
    )
}

#[cfg(test)]
pub mod test {
    use az::SaturatingAs;
    use embedded_graphics::{
        draw_target::DrawTarget,
        geometry::Point,
        mono_font::{ascii::FONT_6X9, MonoFont, MonoTextStyle},
        pixelcolor::{BinaryColor, PixelColor},
        prelude::Size,
        primitives::{Line, PrimitiveStyle, Rectangle, StyledDrawable},
        text::{
            renderer::{CharacterStyle, TextMetrics, TextRenderer},
            Baseline,
        },
        Drawable, Pixel,
    };

    use super::str_width;

    pub fn size_for(font: &MonoFont, chars: u32, lines: u32) -> Size {
        font.character_size.x_axis() * chars + font.character_size.y_axis() * lines
    }

    /// A font where each glyph is 4x10 pixels, except for the
    /// glyph 'j' that is 3x10 with a negative left side bearing of 2 pixels
    #[derive(Copy, Clone)]
    pub struct TestFont<C> {
        text_color: C,
        background_color: C,
        letter_spacing: u32,
    }

    enum LineElement {
        Char(char),
        Spacing,
        Done,
    }

    fn left_side_bearing(c: char) -> i32 {
        match c {
            'j' => -2,
            _ => 0,
        }
    }

    fn char_width(c: char) -> u32 {
        match c {
            'j' => 3,
            _ => 4,
        }
    }

    impl<C> TestFont<C> {
        pub fn new(text_color: C, background_color: C) -> Self {
            Self {
                text_color,
                background_color,
                letter_spacing: 1,
            }
        }

        fn line_elements<'t>(
            &self,
            mut position: Point,
            text: &'t str,
        ) -> impl Iterator<Item = (Point, LineElement)> + 't
where {
            let mut chars = text.chars();
            let mut next_char = chars.next();
            let mut spacing = next_char.map(left_side_bearing);
            let letter_spacing = self.letter_spacing as i32;

            core::iter::from_fn(move || {
                if let Some(offset) = spacing {
                    let p = position;
                    position.x += offset;
                    spacing = None;
                    Some((p, LineElement::Spacing))
                } else if let Some(c) = next_char {
                    let p = position;
                    position.x += char_width(c) as i32;
                    next_char = chars.next();
                    spacing = next_char.map(|c| letter_spacing + left_side_bearing(c));
                    Some((p, LineElement::Char(c)))
                } else {
                    Some((position, LineElement::Done))
                }
            })
        }
    }

    impl<C> CharacterStyle for TestFont<C>
    where
        C: PixelColor,
    {
        type Color = C;
    }

    impl<C> TextRenderer for TestFont<C>
    where
        C: PixelColor,
    {
        type Color = C;

        fn draw_string<D>(
            &self,
            text: &str,
            position: Point,
            _baseline: Baseline,
            target: &mut D,
        ) -> Result<Point, D::Error>
        where
            D: DrawTarget<Color = Self::Color>,
        {
            let style = PrimitiveStyle::with_stroke(self.text_color, 1);
            let bg_style = PrimitiveStyle::with_fill(self.background_color);
            let letter_spacing = self.letter_spacing;
            for (p, element) in self.line_elements(position, text) {
                match element {
                    LineElement::Char('j') => {
                        // draw the 'j' character background, occyping the space behind the stem
                        Rectangle::new(p + Point::new(2, 0), Size::new(1, 10))
                            .draw_styled(&bg_style, target)?;
                        // draw the 'j' character
                        Pixel(p + Point::new(2, 0), self.text_color).draw(target)?;
                        Line::new(p + Point::new(2, 2), p + Point::new(2, 8))
                            .draw_styled(&style, target)?;
                        Line::new(p + Point::new(0, 9), p + Point::new(1, 9))
                            .draw_styled(&style, target)?;
                    }
                    LineElement::Char(_) => {
                        // draw the background for other characters
                        Rectangle::new(p, Size::new(4, 10)).draw_styled(&bg_style, target)?;
                        // draw a 4x7 rectangle for other characters
                        Rectangle::new(p, Size::new(4, 7)).draw_styled(&style, target)?
                    }
                    LineElement::Spacing => {
                        // draw a 1x10 rectangle for letter spacing
                        Rectangle::new(p, Size::new(letter_spacing, 10))
                            .draw_styled(&bg_style, target)?
                    }
                    LineElement::Done => return Ok(p),
                }
            }
            Ok(position)
        }

        fn draw_whitespace<D>(
            &self,
            width: u32,
            position: Point,
            _baseline: Baseline,
            target: &mut D,
        ) -> Result<Point, D::Error>
        where
            D: DrawTarget<Color = Self::Color>,
        {
            let bg_style = PrimitiveStyle::with_fill(self.background_color);
            Rectangle::new(position, Size::new(width, 10)).draw_styled(&bg_style, target)?;
            return Ok(Point::new(position.x + width as i32, position.y));
        }

        fn measure_string(&self, text: &str, position: Point, _baseline: Baseline) -> TextMetrics {
            // the bounding box position can be to the left of the position,
            // when the first character has a negative left side bearing
            // e.g. letter 'j'
            let mut bb_left = position.x;
            let mut bb_right = position.x;
            for (p, element) in self.line_elements(position, text) {
                bb_left = bb_left.min(p.x);
                bb_right = bb_right.max(p.x);
                if let LineElement::Done = element {
                    break;
                }
            }
            let bb_width = bb_right - position.x;
            let bb_size = Size::new(bb_width.saturating_as(), self.line_height());
            TextMetrics {
                bounding_box: Rectangle::new(Point::new(bb_left, position.y), bb_size),
                next_position: position + bb_size.x_axis(),
            }
        }

        fn line_height(&self) -> u32 {
            10
        }
    }

    #[test]
    fn width_of_nbsp_is_single_space() {
        let renderer = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
        assert_eq!(str_width(&renderer, " "), str_width(&renderer, "\u{a0}"));
    }
}
