//! Vertical overdraw options.

/// Implementors of this trait specify how drawing vertically outside the bounding box is handled.
pub trait VerticalOverdraw {}

/// Only render full rows of text.
pub struct FullRowsOnly;
impl VerticalOverdraw for FullRowsOnly {}

/// Render partially visible rows, but only inside the bounding box.
pub struct Hidden;
impl VerticalOverdraw for Hidden {}

/// Display text even if it's outside the bounding box.
pub struct Visible;
impl VerticalOverdraw for Visible {}

#[cfg(test)]
mod test {
    use embedded_graphics::{
        fonts::Font6x8, mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
        primitives::Rectangle,
    };

    use crate::{
        alignment::{BottomAligned, LeftAligned},
        style::TextBoxStyleBuilder,
        TextBox,
    };

    #[test]
    fn default_is_full_rows_only() {
        // This test verifies that FullRowsOnly does not draw partial rows

        let mut display = MockDisplay::new();
        let style = TextBoxStyleBuilder::new(Font6x8)
            .alignment(LeftAligned)
            .text_color(BinaryColor::On)
            .build();

        TextBox::new(
            "word and other words",
            Rectangle::new(Point::new(0, 0), Point::new(54, 14)),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                      #                       #",
                "                      #                       #",
                "#   #  ###  # ##   ## #        ###  # ##   ## #",
                "#   # #   # ##  # #  ##           # ##  # #  ##",
                "# # # #   # #     #   #        #### #   # #   #",
                "# # # #   # #     #   #       #   # #   # #   #",
                " # #   ###  #      ####        #### #   #  ####",
                "                                               ",
            ])
        );
    }
}
