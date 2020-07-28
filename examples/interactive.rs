//! This example draws text into a bounding box that can be modified by
//! clicking on the display.
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use embedded_graphics::{
    fonts::Font6x8, pixelcolor::BinaryColor, prelude::*, style::PrimitiveStyle,
};
use std::{thread, time::Duration};

use embedded_text::{alignment::*, prelude::*};

fn main() -> Result<(), core::convert::Infallible> {
    let textbox_style = TextBoxStyleBuilder::new(Font6x8)
        .alignment(Justified)
        .text_color(BinaryColor::On)
        .build();

    let rectangle_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    let text = "Hello, World!\nLorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.";

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let mut window = Window::new("TextBox demonstration", &output_settings);

    let mut bounds = Rectangle::new(Point::zero(), Point::new(128, 128));

    'running: loop {
        let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(129, 129));

        TextBox::new(text, bounds)
            .into_styled(textbox_style)
            .draw(&mut display)
            .unwrap();

        bounds
            .into_styled(rectangle_style)
            .draw(&mut display)
            .unwrap();

        window.update(&display);
        for event in window.events() {
            match event {
                SimulatorEvent::MouseButtonDown { point, .. } => {
                    println!("MouseDown: {:?}", point);
                    bounds = Rectangle::new(Point::zero(), point);
                }
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}
