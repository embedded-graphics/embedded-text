//! Text alignment options.
use crate::{
    rendering::{cursor::Cursor, space_config::SpaceConfig},
    style::LineMeasurement,
    utils::str_width,
};
use embedded_graphics::text::renderer::TextRenderer;

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
    /// Calculate offset from the left side and whitespace information.
    pub(crate) fn place_line(
        self,
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
                let space_count = measurement.space_count;
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
}

impl VerticalAlignment {
    /// Set the cursor's initial vertical position
    pub(crate) fn apply_vertical_alignment(
        self,
        cursor: &mut Cursor,
        text_height: i32,
        box_height: i32,
    ) {
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
        }
    }
}
