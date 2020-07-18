use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};

use embedded_graphics::{
    fonts::Font6x8, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};

use embedded_text::{style::TextBoxStyleBuilder, TextBox};

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(129, 129));

    let textbox_style = TextBoxStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    TextBox::new(
        "Hello, World!\nThis is some longer text to demonstrate a TextBox, also with  a verymuchultramegahyperlong word.\nHow    does    weird  spaces behave?",
        Rectangle::new(Point::zero(), Point::new(128, 128)),
    )
    .into_styled(textbox_style)
    // align text to the display
    .draw(&mut display)
    .unwrap();

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello TextBox", &output_settings).show_static(&display);
    Ok(())
}
