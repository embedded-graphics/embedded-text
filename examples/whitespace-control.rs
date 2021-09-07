//! # Example: whitespace control.
//!
//! This example demonstrates the different leading/trailing whitespace options and their effect.

use embedded_graphics::{
    geometry::AnchorPoint,
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::Rectangle,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_text::{style::TextBoxStyleBuilder, TextBox};
use std::convert::Infallible;

fn main() -> Result<(), Infallible> {
    // Set up the window.
    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    let mut window = Window::new("Interactive TextBox demonstration", &output_settings);

    let text = "  Hello, World!\n  \
        Lorem Ipsum is simply dummy text of the printing and typesetting industry.   \
        Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
        an unknown printer took a galley of type and scrambled it to make a type specimen book.";

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb888::WHITE)
        .background_color(Rgb888::CSS_STEEL_BLUE)
        .build();

    // Create a simulated display.
    let mut display = SimulatorDisplay::new(Size::new(255, 255));

    // Create bounding boxes
    let bounds = Rectangle::new(Point::zero(), Size::new(255, 255));

    // Create and draw the text boxes.
    TextBox::with_textbox_style(
        text,
        bounds.resized(Size::new(128, 255), AnchorPoint::TopLeft),
        character_style,
        TextBoxStyleBuilder::default().build(),
    )
    .draw(&mut display)?;

    TextBox::with_textbox_style(
        text,
        bounds.resized(Size::new(128, 255), AnchorPoint::TopRight),
        character_style,
        TextBoxStyleBuilder::default()
            .leading_spaces(true)
            .trailing_spaces(true)
            .build(),
    )
    .draw(&mut display)?;

    // Update the window.
    window.show_static(&display);

    Ok(())
}
