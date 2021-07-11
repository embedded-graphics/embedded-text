//! # Example: static styles
//!
//! This example demonstrates additional text decoration options (underlined and strike-through
//! text, text background).

use std::convert::Infallible;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::Rectangle,
    text::LineHeight,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_text::{
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};

fn main() -> Result<(), Infallible> {
    let text = "Lorem Ipsum is simply dummy text of the printing and typesetting industry.";

    let text_box_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .line_height(LineHeight::Pixels(12))
        .build();

    let underlined_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb888::WHITE)
        .underline_with_color(Rgb888::GREEN)
        .build();

    let strikethrough_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb888::WHITE)
        .strikethrough_with_color(Rgb888::RED)
        .build();

    let background_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb888::WHITE)
        .background_color(Rgb888::CSS_STEEL_BLUE)
        .build();

    let text_box = TextBox::with_textbox_style(
        text,
        Rectangle::new(Point::zero(), Size::new(97, 0)),
        underlined_style,
        text_box_style,
    );

    let text_box2 = TextBox::with_textbox_style(
        text,
        Rectangle::new(Point::new(96, 0), Size::new(97, 0)),
        strikethrough_style,
        text_box_style,
    );

    let text_box3 = TextBox::with_textbox_style(
        text,
        Rectangle::new(Point::new(192, 0), Size::new(97, 0)),
        background_style,
        text_box_style,
    );

    // Create a window for both text boxes.
    let mut display = SimulatorDisplay::new(Size::new(
        text_box.bounding_box().size.width
            + text_box2.bounding_box().size.width
            + text_box3.bounding_box().size.width,
        text_box
            .bounding_box()
            .size
            .height
            .max(text_box2.bounding_box().size.height)
            .max(text_box3.bounding_box().size.height),
    ));

    // Draw the text boxes.
    text_box.draw(&mut display)?;
    text_box2.draw(&mut display)?;
    text_box3.draw(&mut display)?;

    // Set up the window and show the display's contents.
    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    Window::new("Hello TextBox", &output_settings).show_static(&display);

    Ok(())
}
