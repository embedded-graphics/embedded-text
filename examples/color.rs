//! This example demonstrates styling a piece of text using text box styling options.

use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    pixelcolor::Rgb565,
    prelude::*,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_text::prelude::*;

fn main() {
    let text = "Hello, World!\n\
    Lorem Ipsum is simply dummy text of the printing and typesetting industry. \
    Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
    an unknown printer took a galley of type and scrambled it to make a type specimen book.";

    // Specify the styling options:
    // * Use the 6x8 MonoFont from embedded-graphics.
    // * Draw the text fully justified.
    // * Draw the text with cyan text color and a gray background color.
    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(Rgb565::CYAN)
        .background_color(Rgb565::new(10, 20, 10))
        .build();

    let textbox_style = TextBoxStyleBuilder::new()
        .character_style(character_style)
        .alignment(Justified)
        .build();

    // Specify the bounding box. Note that in this example the text box will be taller than the text.
    let bounds = Rectangle::new(Point::zero(), Size::new(129, 129));

    // Create the text box and apply styling options.
    let text_box = TextBox::new(text, bounds).into_styled(textbox_style);

    // Create a simulated display with the dimensions of the text box.
    let mut display = SimulatorDisplay::new(text_box.bounding_box().size);

    // Draw the text box.
    text_box.draw(&mut display).unwrap();

    // Set up the window and show the display's contents.
    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    Window::new("Hello TextBox with text background color", &output_settings).show_static(&display);
}
