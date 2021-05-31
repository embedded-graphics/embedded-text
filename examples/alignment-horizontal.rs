//! Example: horizontal text alignment.
//!
//! This example demonstrates drawing a piece of text using the available horizontal alignment options.
//! The example uses different, but equivalent ways to specify the alignment options.

use std::convert::Infallible;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{TextBoxStyle, TextBoxStyleBuilder},
    TextBox,
};

fn main() -> Result<(), Infallible> {
    let mut display = SimulatorDisplay::new(Size::new(128, 184));

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    TextBox::new(
        "Hello, World!\nSome text to demonstrate left alignment.",
        Rectangle::new(Point::zero(), Size::new(128, 48)),
        character_style,
    )
    .draw(&mut display)?;

    TextBox::with_alignment(
        "Hello, World!\nSome text to demonstrate center alignment.",
        Rectangle::new(Point::new(0, 48), Size::new(128, 48)),
        character_style,
        HorizontalAlignment::Center,
    )
    .draw(&mut display)?;

    TextBox::with_textbox_style(
        "Hello, World!\nSome text to demonstrate right alignment.",
        Rectangle::new(Point::new(0, 96), Size::new(128, 48)),
        character_style,
        TextBoxStyle::with_alignment(HorizontalAlignment::Right),
    )
    .draw(&mut display)?;

    TextBox::with_textbox_style(
        "Hello, World!\nSome text to demonstrate fully justified alignment.",
        Rectangle::new(Point::new(0, 144), Size::new(128, 48)),
        character_style,
        TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Justified)
            .build(),
    )
    .draw(&mut display)?;

    // Set up the window and show the display's contents.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .scale(2)
        .build();
    Window::new("Horizontal alignment example", &output_settings).show_static(&display);

    Ok(())
}
