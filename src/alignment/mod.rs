//! Text alignment options.
use crate::{
    parser::SPEC_CHAR_NBSP,
    rendering::{cursor::Cursor, space_config::SpaceConfig},
    style::LineMeasurement,
    utils::str_width,
    TextBox,
};
use embedded_graphics::{prelude::Dimensions, text::renderer::TextRenderer};

#[cfg(test)]
mod test;

/// Horizontal text alignment options.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum HorizontalAlignment {
    /// Left aligned.
    Left,

    /// Center aligned.
    Center,

    /// Right aligned.
    Right,

    /// Fully justified.
    Justified,
}

impl HorizontalAlignment {
    pub(crate) fn ignores_leading_spaces(self) -> bool {
        match self {
            HorizontalAlignment::Left => false,
            HorizontalAlignment::Center => true,
            HorizontalAlignment::Right => true,
            HorizontalAlignment::Justified => true,
        }
    }

    /// Calculate offset from the left side and whitespace information.
    pub fn place_line(
        self,
        line: &str,
        renderer: &impl TextRenderer,
        measurement: LineMeasurement,
    ) -> (u32, SpaceConfig) {
        match self {
            HorizontalAlignment::Left => (0, SpaceConfig::new_from_renderer(renderer)),
            HorizontalAlignment::Center => (
                (measurement.max_line_width - measurement.width + 1) / 2,
                SpaceConfig::new_from_renderer(renderer),
            ),
            HorizontalAlignment::Right => (
                measurement.max_line_width - measurement.width,
                SpaceConfig::new_from_renderer(renderer),
            ),
            HorizontalAlignment::Justified => {
                let space_width = str_width(renderer, " ");
                let space_chars = [' ', SPEC_CHAR_NBSP];

                let mut space_count = 0;
                let mut partial_space_count = 0;

                for c in line.chars().skip_while(|c| space_chars.contains(c)) {
                    if space_chars.contains(&c) {
                        partial_space_count += 1;
                    } else {
                        space_count += partial_space_count;
                        partial_space_count = 0;
                    }
                }

                let space_info = if !measurement.last_line && space_count != 0 {
                    let space =
                        measurement.max_line_width - measurement.width + space_count * space_width;
                    let space_width = space / space_count;
                    let extra_pixels = space % space_count;
                    SpaceConfig::new(space_width, Some(extra_pixels))
                } else {
                    SpaceConfig::new(space_width, None)
                };
                (0, space_info)
            }
        }
    }
}

/// Vertical text alignment options.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum VerticalAlignment {
    /// Top aligned text.
    ///
    /// The first line of the text will be rendered at the top of the text box.
    Top,

    /// Middle aligned text.
    ///
    /// The text will be vertically centered within the text box.
    Middle,

    /// Bottom aligned text.
    ///
    /// The last line of the text will be aligned to the bottom of the text box.
    Bottom,

    /// Scrolling alignment.
    ///
    /// Aligns the last line of the text to be always visible. If the text fits inside the text box,
    /// it will be top aligned. If the text is longer, it will be bottom aligned.
    Scrolling,
}

impl VerticalAlignment {
    /// Set the cursor's initial vertical position
    pub fn apply_vertical_alignment<'a, 'b, S>(
        self,
        cursor: &mut Cursor,
        styled_text_box: &'b TextBox<'a, S>,
    ) where
        S: TextRenderer,
    {
        let text_height = styled_text_box.style.measure_text_height(
            &styled_text_box.character_style,
            styled_text_box.text,
            cursor.line_width(),
        ) as i32;

        let box_height = styled_text_box.bounding_box().size.height as i32;

        match self {
            VerticalAlignment::Top => {
                // nothing to do here
            }

            VerticalAlignment::Middle => {
                let offset = (box_height - text_height) / 2;

                cursor.y += offset;
            }

            VerticalAlignment::Bottom => {
                let offset = box_height - text_height;

                cursor.y += offset;
            }

            VerticalAlignment::Scrolling => {
                if text_height > box_height {
                    let offset = box_height - text_height;

                    cursor.y += offset
                }
            }
        }
    }
}
