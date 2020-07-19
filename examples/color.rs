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
        "Hello, World!\nThis is some longer text to demonstrate a TextBox, also with a verymuchultramegahyperlong word.\nHow    does    weird  spaces behave?\nAlso test word wrapping\n ",
        Rectangle::new(Point::zero(), Point::new(128, 128)),
    )
    .into_styled(textbox_style)
    // align text to the display
    .draw(&mut display)
    .unwrap();

    let output_settings = OutputSettingsBuilder::new().build();
    Window::new("Hello TextBox", &output_settings).show_static(&display);
    Ok(())
}
