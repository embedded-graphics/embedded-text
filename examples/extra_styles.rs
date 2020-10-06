use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::prelude::*;

fn main() -> Result<(), core::convert::Infallible> {
    let text = "Lorem Ipsum is simply dummy text of the printing and typesetting industry.";

    let underlined_style = TextBoxStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .height_mode(FitToText)
        .underlined(true)
        .line_spacing(2)
        .build();
    let strikethrough_style = TextBoxStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .height_mode(FitToText)
        .strikethrough(true)
        .line_spacing(2)
        .build();

    let text_box = TextBox::new(text, Rectangle::new(Point::zero(), Point::new(96, 0)))
        .into_styled(underlined_style);

    let text_box2 = TextBox::new(text, Rectangle::new(Point::new(96, 0), Point::new(192, 0)))
        .into_styled(strikethrough_style);

    // Create a window just tall enough to fit the text.
    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(
        text_box.size().width + text_box2.size().width,
        text_box.size().height.max(text_box2.size().height),
    ));
    text_box.draw(&mut display).unwrap();
    text_box2.draw(&mut display).unwrap();

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello TextBox", &output_settings).show_static(&display);
    Ok(())
}
