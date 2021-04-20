//! This example draws text in three columns to demonstrate the common vertical alignments.

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::prelude::*;

fn main() {
    let text = "The quick brown fox jumped over the lazy dog.";

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let base_style = TextBoxStyleBuilder::new().character_style(character_style);

    // Create a 192x129 px simulated display.
    let mut display = SimulatorDisplay::new(Size::new(192, 129));

    // Divide the screen into 3 64px wide columns.
    let bounds_top = Rectangle::new(Point::zero(), Size::new(64, 129));
    let bounds_center = Rectangle::new(Point::new(64, 0), Size::new(64, 129));
    let bounds_bottom = Rectangle::new(Point::new(128, 0), Size::new(64, 129));

    let textbox_style_top = base_style.clone().vertical_alignment(TopAligned).build();
    TextBox::with_textbox_style(text, bounds_top, textbox_style_top)
        .draw(&mut display)
        .unwrap();

    let textbox_style_center = base_style.clone().vertical_alignment(CenterAligned).build();
    TextBox::with_textbox_style(text, bounds_center, textbox_style_center)
        .draw(&mut display)
        .unwrap();

    let textbox_style_bottom = base_style.clone().vertical_alignment(BottomAligned).build();
    TextBox::with_textbox_style(text, bounds_bottom, textbox_style_bottom)
        .draw(&mut display)
        .unwrap();

    // Set up the window and show the display's contents.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello TextBox", &output_settings).show_static(&display);
}
