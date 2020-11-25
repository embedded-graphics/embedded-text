use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::prelude::*;

fn main() {
    let text = "Hello, World!\nLorem Ipsum is simply dummy text of the printing and typesetting \
    industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
    an unknown printer took a galley of type and scrambled it to make a type specimen book.";

    let textbox_style = TextBoxStyleBuilder::new(Font6x8)
        .alignment(CenterAligned)
        .height_mode(FitToText)
        .text_color(BinaryColor::On)
        .build();

    // Create the text box. Note that the size is set to 129x0. The `FitToText` height mode will
    // measure and adjust the height of the text box in `into_styled()`.
    let text_box = TextBox::new(text, Rectangle::new(Point::zero(), Point::new(128, 0)))
        .into_styled(textbox_style);

    // Create a window just tall enough to fit the text.
    let mut display = SimulatorDisplay::new(text_box.size());
    text_box.draw(&mut display).unwrap();

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello center aligned TextBox", &output_settings).show_static(&display);
}
