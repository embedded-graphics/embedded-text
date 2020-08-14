use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};

use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};

use embedded_text::alignment::vertical;
use embedded_text::prelude::*;

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(192, 129));

    let text = "The quick brown fox jumped over the lazy dog.";

    let textbox_style_top = TextBoxStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .vertical_alignment(vertical::Top)
        .build();

    let textbox_style_center = TextBoxStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .vertical_alignment(vertical::Center)
        .build();

    let textbox_style_bottom = TextBoxStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .vertical_alignment(vertical::Bottom)
        .build();

    let bounds_top = Rectangle::new(Point::zero(), Point::new(63, 128));
    let bounds_center = Rectangle::new(Point::new(64, 0), Point::new(127, 128));
    let bounds_bottom = Rectangle::new(Point::new(128, 0), Point::new(191, 128));

    TextBox::new(text, bounds_top)
        .into_styled(textbox_style_top)
        .draw(&mut display)
        .unwrap();

    TextBox::new(text, bounds_center)
        .into_styled(textbox_style_center)
        .draw(&mut display)
        .unwrap();

    TextBox::new(text, bounds_bottom)
        .into_styled(textbox_style_bottom)
        .draw(&mut display)
        .unwrap();

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello TextBox", &output_settings).show_static(&display);
    Ok(())
}
