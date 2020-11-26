//! This example demonstrates support for carriage return, non-breaking space and zero-width space
//! characters to modify text layout behaviour.

use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::prelude::*;

fn main() {
    // Example text sprinkled with special characters. Note that "supports" will not be displayed
    // because it will be overdrawn by the carriage return.
    let text = "Hello, World!\n\
    embedded-text supports\rcarriage return.\n\
    Non-breaking spaces\u{A0}are also supported.\n\
    Also\u{200B}Supports\u{200B}Zero\u{200B}Width\u{200B}Space\u{200B}Characters";

    // Specify the styling options:
    // * Use the 6x8 font from embedded-graphics.
    // * Draw the text fully justified.
    // * Use `FitToText` height mode to stretch the text box to the exact height of the text.
    // * Draw the text with `BinaryColor::On`, which will be displayed as light blue.
    // * Draw the text with `BinaryColor::Off` background color, which will be rendered as dark
    //   blue. This is used to overwrite parts of the text in this example.
    let textbox_style = TextBoxStyleBuilder::new(Font6x8)
        .alignment(Justified)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .height_mode(FitToText)
        .build();

    // Specify the bounding box. Note the 0px height. The `FitToText` height mode will
    // measure and adjust the height of the text box in `into_styled()`.
    let bounds = Rectangle::new(Point::zero(), Point::new(128, 0));

    // Create the text box and apply styling options.
    let text_box = TextBox::new(text, bounds).into_styled(textbox_style);

    // Create a simulated display with the dimensions of the text box.
    let mut display = SimulatorDisplay::new(text_box.size());

    // Draw the text box.
    text_box.draw(&mut display).unwrap();

    // Set up the window and show the display's contents.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Center aligned TextBox example", &output_settings).show_static(&display);
}
