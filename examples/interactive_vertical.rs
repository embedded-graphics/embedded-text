//! This example draws text into a bounding box that can be modified by
//! clicking and dragging on the display.
//!
//! Press spacebar to switch between vertical alignment modes.
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::PrimitiveStyle,
    text::Text,
};
use embedded_text::prelude::*;
use sdl2::keyboard::Keycode;
use std::{thread, time::Duration};

enum ProcessedEvent {
    Nothing,
    Quit,
    Next,
    Resize(Point),
}

impl ProcessedEvent {
    /// Translates simulator events to logical events used by the example.
    pub fn new(event: SimulatorEvent) -> Self {
        unsafe {
            // This is fine for a demo
            static mut MOUSE_DOWN: bool = false;

            match event {
                SimulatorEvent::MouseButtonDown { point, .. } => {
                    println!("MouseButtonDown: {:?}", point);
                    MOUSE_DOWN = true;
                    ProcessedEvent::Resize(point)
                }
                SimulatorEvent::MouseButtonUp { .. } => {
                    println!("MouseButtonUp");
                    MOUSE_DOWN = false;
                    ProcessedEvent::Nothing
                }
                SimulatorEvent::MouseMove { point, .. } => {
                    if MOUSE_DOWN {
                        println!("MouseMove: {:?}", point);
                        ProcessedEvent::Resize(point)
                    } else {
                        ProcessedEvent::Nothing
                    }
                }
                SimulatorEvent::KeyDown { keycode, .. } if keycode == Keycode::Space => {
                    ProcessedEvent::Next
                }
                SimulatorEvent::Quit => ProcessedEvent::Quit,
                _ => ProcessedEvent::Nothing,
            }
        }
    }
}

fn demo_loop<'a, V>(window: &mut Window, bounds: &mut Rectangle, alignment: V) -> bool
where
    V: VerticalTextAlignment + std::fmt::Debug,
{
    let text = "Hello, World!\n\
    Lorem Ipsum is simply dummy text of the printing and typesetting industry. \
    Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
    an unknown printer took a galley of type and scrambled it to make a type specimen book.\n\
    super\u{AD}cali\u{AD}fragi\u{AD}listic\u{AD}espeali\u{AD}docious";

    loop {
        // Create a simulated display.
        let mut display = SimulatorDisplay::new(Size::new(255, 255));

        // Specify the styling options:
        // * Use the 6x8 MonoFont from embedded-graphics.
        // * Draw the text horizontally left aligned (default option, not specified here).
        // * Draw the text with `BinaryColor::On`, which will be displayed as light blue.
        // * Use the vertical alignmnet mode that was given to the `demo_loop()` function.
        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X9)
            .text_color(BinaryColor::On)
            .build();

        let textbox_style = TextBoxStyleBuilder::new()
            .vertical_alignment(alignment)
            .build();

        // Create the text box and apply styling options.
        let text_box = TextBox::with_textbox_style(text, *bounds, character_style, textbox_style);

        // Draw the text box.
        text_box.draw(&mut display).unwrap();

        // Draw the bounding box of the text box.
        text_box
            .bounds
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        // Display the name of the vertical alignment mode above the text box.
        let vertical_alignment_text = format!("Vertical Alignment: {:?}", alignment);
        Text::new(&vertical_alignment_text, Point::new(0, 6), character_style)
            .draw(&mut display)
            .unwrap();

        // Update the window.
        window.update(&display);

        // Handle key and mouse events.
        for event in window.events() {
            match ProcessedEvent::new(event) {
                ProcessedEvent::Resize(bottom_right) => {
                    // Make sure we don't move the text box
                    let new_bottom_right = Point::new(
                        bottom_right.x.max(bounds.top_left.x),
                        bottom_right.y.max(bounds.top_left.y),
                    );
                    *bounds = Rectangle::with_corners(bounds.top_left, new_bottom_right);
                }
                ProcessedEvent::Quit => return false,
                ProcessedEvent::Next => return true,
                ProcessedEvent::Nothing => {}
            }
        }

        // Wait for a little while.
        thread::sleep(Duration::from_millis(10));
    }
}

fn main() {
    // Set up the window.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let mut window = Window::new("TextBox demonstration", &output_settings);

    // Specify the bounding box. Leave 8px of space above.
    let mut bounds = Rectangle::new(Point::new(0, 8), Size::new(128, 200));

    'running: loop {
        if !demo_loop(&mut window, &mut bounds, TopAligned) {
            break 'running;
        }
        if !demo_loop(&mut window, &mut bounds, CenterAligned) {
            break 'running;
        }
        if !demo_loop(&mut window, &mut bounds, BottomAligned) {
            break 'running;
        }
        if !demo_loop(&mut window, &mut bounds, Scrolling) {
            break 'running;
        }
    }
}
