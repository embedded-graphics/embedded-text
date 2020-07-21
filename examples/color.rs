use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

use embedded_graphics::{fonts::Font6x8, pixelcolor::Rgb565, prelude::*, primitives::Rectangle};

use embedded_text::{alignment::*, style::TextBoxStyleBuilder, TextBox};

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(129, 129));

    let textbox_style = TextBoxStyleBuilder::new(Font6x8)
        .alignment(Justified)
        .text_color(Rgb565::RED)
        .background_color(Rgb565::GREEN)
        .build();

    TextBox::new(
        "Hello, World!\nLorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.",
        Rectangle::new(Point::zero(), Point::new(128, 128)),
    )
    .into_styled(textbox_style)
    .draw(&mut display)
    .unwrap();

    let output_settings = OutputSettingsBuilder::new().build();
    Window::new("Hello TextBox with text background color", &output_settings).show_static(&display);
    Ok(())
}
