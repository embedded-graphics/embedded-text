use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::prelude::*;

fn main() {
    let mut display = SimulatorDisplay::new(Size::new(192, 129));

    let text = "The quick brown fox jumped over the lazy dog.";

    let base_style = TextBoxStyleBuilder::new(Font6x8).text_color(BinaryColor::On);

    let textbox_style_top = base_style.vertical_alignment(TopAligned).build();
    let textbox_style_center = base_style.vertical_alignment(CenterAligned).build();
    let textbox_style_bottom = base_style.vertical_alignment(BottomAligned).build();

    // Divide the screen into 3 64px wide columns
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
}
