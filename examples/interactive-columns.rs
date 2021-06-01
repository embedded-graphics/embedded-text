//! This example draws text into a bounding box that can be modified by
//! clicking and dragging on the display.
//!
//! Press spacebar to switch between horizontal alignment modes.
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_text::TextBox;
use std::{convert::Infallible, thread, time::Duration};

enum ProcessedEvent {
    Nothing,
    Quit,
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
                SimulatorEvent::Quit => ProcessedEvent::Quit,
                _ => ProcessedEvent::Nothing,
            }
        }
    }
}

fn main() -> Result<(), Infallible> {
    // Set up the window.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .scale(2)
        .build();
    let mut window = Window::new("Interactive TextBox demonstration", &output_settings);

    let text = "Hello, World!\n\
        Lorem Ipsum is simply dummy text of the printing and typesetting industry. \
        Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
        an unknown printer took a galley of type and scrambled it to make a type specimen book.\n\
        super\u{AD}cali\u{AD}fragi\u{AD}listic\u{AD}espeali\u{AD}docious";

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let mut bounds = Rectangle::new(Point::new(1, 1), Size::new(128, 200));

    'demo: loop {
        // Create a simulated display.
        let mut display = SimulatorDisplay::new(Size::new(255, 255));

        // Create bounding boxes
        let size = Size::new(bounds.size.width / 2 - 1, bounds.size.height);
        let bounds1 = Rectangle::new(bounds.top_left, size);
        let bounds2 = Rectangle::new(
            Point::new(
                bounds1.bottom_right().unwrap_or_default().x + 3,
                bounds1.top_left.y,
            ),
            size,
        );

        // Create and draw the text boxes.
        let remaining_text = TextBox::new(text, bounds1, character_style).draw(&mut display)?;
        TextBox::new(remaining_text, bounds2, character_style).draw(&mut display)?;

        // Draw the bounding box of the text box.
        bounds
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .stroke_alignment(StrokeAlignment::Outside)
                    .stroke_color(BinaryColor::On)
                    .stroke_width(1)
                    .build(),
            )
            .draw(&mut display)?;

        // Update the window.
        window.update(&display);

        // Handle key and mouse events.
        for event in window.events() {
            match ProcessedEvent::new(event) {
                ProcessedEvent::Resize(bottom_right) => {
                    // Make sure we don't move the text box
                    bounds = Rectangle::with_corners(
                        bounds.top_left,
                        bottom_right.component_max(bounds.top_left),
                    );
                }
                ProcessedEvent::Quit => break 'demo,
                ProcessedEvent::Nothing => {}
            }
        }

        // Wait for a little while.
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}
