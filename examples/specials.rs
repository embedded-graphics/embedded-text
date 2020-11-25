use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::prelude::*;

fn main() {
    let text = "Hello, World!\nembedded-text supports\rcarriage return.\nNon-breaking \
    spaces\u{A0}are also supported.\nAlso\u{200B}Supports\u{200B}Zero\u{200B}Width\u{200B}Space\u{200B}Characters";

    let textbox_style = TextBoxStyleBuilder::new(Font6x8)
        .alignment(Justified)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .height_mode(FitToText)
        .build();

    let text_box = TextBox::new(text, Rectangle::new(Point::zero(), Point::new(128, 0)))
        .into_styled(textbox_style);

    // Create a window just tall enough to fit the text.
    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(text_box.size());
    text_box.draw(&mut display).unwrap();

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Special character handling example", &output_settings).show_static(&display);
}
