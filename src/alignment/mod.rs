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
    ) -> (i32, SpaceConfig) {
        let space_width = str_width(renderer, " ");
        let space_config = SpaceConfig::new(space_width, None);
        if measurement.max_line_width < measurement.width {
            panic!("{} {}", measurement.max_line_width, measurement.width)
        }
        let remaining_space = measurement.max_line_width - measurement.width;
        match self {
            HorizontalAlignment::Left => (0, space_config),
            HorizontalAlignment::Center => ((remaining_space as i32 + 1) / 2, space_config),
            HorizontalAlignment::Right => (remaining_space as i32, space_config),
            HorizontalAlignment::Justified => {
                let space_count = measurement.space_count;
                let space_info = if !measurement.last_line() && space_count != 0 {
                    let space = remaining_space + space_count * space_width;
                    let space_width = space / space_count;
                    let extra_pixels = space % space_count;
                    SpaceConfig::new(space_width, Some(extra_pixels))
                } else {
                    space_config
                };
                (0, space_info)
            }
        }
    }

    pub(crate) const fn leading_spaces(self) -> bool {
        match self {
            HorizontalAlignment::Left => true,
            HorizontalAlignment::Center => false,
            HorizontalAlignment::Right => false,
            HorizontalAlignment::Justified => false,
        }
    }

    pub(crate) const fn trailing_spaces(self) -> bool {
        false
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
