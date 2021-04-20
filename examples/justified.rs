//! This example demonstrates drawing a piece of text fully justified.

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
    let text = "Hello, World!\n\
    Lorem Ipsum is simply dummy text of the printing and typesetting industry. \
    Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
    an unknown printer took a galley of type and scrambled it to make a type specimen book.";

    // Specify the styling options:
    // * Use the 6x8 MonoFont from embedded-graphics.
    // * Draw the text fully justified.
    // * Use `FitToText` height mode to stretch the text box to the exact height of the text.
    // * Draw the text with `BinaryColor::On`, which will be displayed as light blue.
    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let textbox_style = TextBoxStyleBuilder::new()
        .character_style(character_style)
        .alignment(Justified)
        .height_mode(FitToText)
        .build();

    // Specify the bounding box. Note the 0px height. The `FitToText` height mode will
    // measure and adjust the height of the text box in `into_styled()`.
    let bounds = Rectangle::new(Point::zero(), Size::new(129, 0));

    // Create the text box and apply styling options.
    let text_box = TextBox::new(text, bounds).into_styled(textbox_style);

    // Create a simulated display with the dimensions of the text box.
    let mut display = SimulatorDisplay::new(text_box.bounding_box().size);

    // Draw the text box.
    text_box.draw(&mut display).unwrap();

    // Set up the window and show the display's contents.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Fully justified TextBox example", &output_settings).show_static(&display);
}
