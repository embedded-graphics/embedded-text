//! # Example: Tail plugin.
//!
//! This example demonstrates drawing a piece of text with the Tail plugin. The Tail plugin positions
//! text so that the last lines are visible.

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
    plugin::tail::Tail,
    style::{HeightMode, TextBoxStyleBuilder, VerticalOverdraw},
    TextBox,
};

fn main() -> Result<(), Infallible> {
    let mut display = SimulatorDisplay::new(Size::new(128, 96));

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // Divide the screen into 3 64px wide columns.
    TextBox::new(
        "Short text using the Tail plugin is aligned to the top.",
        Rectangle::new(Point::zero(), Size::new(64, 96)),
        character_style,
    )
    .add_plugin(Tail)
    .draw(&mut display)?;

    TextBox::with_textbox_style(
        "Some longer text to demonstrate that Tail plugin aligns text so that the \
        bottom line is always visible.",
        Rectangle::new(Point::new(64, 0), Size::new(64, 96)),
        character_style,
        TextBoxStyleBuilder::new()
            .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
            .build(),
    )
    .add_plugin(Tail)
    .draw(&mut display)?;

    // Set up the window and show the display's contents.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .scale(2)
        .build();
    Window::new("Vertical alignment example", &output_settings).show_static(&display);

    Ok(())
}
