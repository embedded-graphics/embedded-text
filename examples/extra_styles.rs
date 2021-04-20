//! This example demonstrates additional text decoration options (underlined and strike-through text).

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::prelude::*;

fn main() {
    let text = "Lorem Ipsum is simply dummy text of the printing and typesetting industry.";

    // Specify the common styling options:
    // * Use the 6x8 MonoFont from embedded-graphics.
    // * Draw the text horizontally left aligned (default option, not specified here).
    // * Use `FitToText` height mode to stretch the text box to the exact height of the text.
    // * Draw the text with `BinaryColor::On`, which will be displayed as light blue.
    let base_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On);

    let base_tb_style = TextBoxStyleBuilder::new()
        .vertical_alignment(Scrolling)
        .height_mode(FitToText)
        .line_spacing(2);

    // Specify underlined and strike-through decorations, one for each text box.
    let underlined_style = base_style.underline().build();
    let strikethrough_style = base_style.strikethrough().build();

    let underlined_tb_style = base_tb_style
        .clone()
        .character_style(underlined_style)
        .build();
    let strikethrough_tb_style = base_tb_style
        .clone()
        .character_style(strikethrough_style)
        .build();

    let text_box = TextBox::with_textbox_style(
        text,
        Rectangle::new(Point::zero(), Size::new(97, 0)),
        underlined_tb_style,
    );

    let text_box2 = TextBox::with_textbox_style(
        text,
        Rectangle::new(Point::new(96, 0), Size::new(97, 0)),
        strikethrough_tb_style,
    );

    // Create a window for both text boxes.
    let mut display = SimulatorDisplay::new(Size::new(
        text_box.bounding_box().size.width + text_box2.bounding_box().size.width,
        text_box
            .bounding_box()
            .size
            .height
            .max(text_box2.bounding_box().size.height),
    ));

    // Draw the text boxes.
    text_box.draw(&mut display).unwrap();
    text_box2.draw(&mut display).unwrap();

    // Set up the window and show the display's contents.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello TextBox", &output_settings).show_static(&display);
}
