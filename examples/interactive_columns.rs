//! This example draws text into a bounding box that can be modified by
//! clicking and dragging on the display.
//!
//! Press spacebar to switch between horizontal alignment modes.
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use embedded_graphics::{
    mono_font::{ascii::Font6x9, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::PrimitiveStyle,
    text::Text,
};
use embedded_text::{prelude::*, style::vertical_overdraw::FullRowsOnly};
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

fn demo_loop<'a, A>(window: &mut Window, bounds: &mut Rectangle, alignment: A) -> bool
where
    A: HorizontalTextAlignment + core::fmt::Debug,
    StyledTextBox<'a, MonoTextStyle<BinaryColor, Font6x9>, A, TopAligned, Exact<FullRowsOnly>>:
        Drawable<Color = BinaryColor, Output = &'a str>,
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
        // * Use the horizontal alignmnet mode that was given to the `demo_loop()` function.
        // * Draw the text with `BinaryColor::On`, which will be displayed as light blue.
        let character_style = MonoTextStyleBuilder::new()
            .font(Font6x9)
            .text_color(BinaryColor::On)
            .build();

        let textbox_style = TextBoxStyleBuilder::new()
            .character_style(character_style)
            .alignment(alignment)
            .build();

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
        let text_box1 = TextBox::new(text, bounds1).into_styled(textbox_style);
        let remaining_text = text_box1.draw(&mut display).unwrap();

        let text_box2 = TextBox::new(remaining_text, bounds2).into_styled(textbox_style);
        text_box2.draw(&mut display).unwrap();

        // Draw the bounding box of the text box.
        bounds
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        // Display the name of the horizontal alignment mode above the text box.
        let horizontal_alignment_text = format!("Alignment: {:?}", alignment);
        Text::new(&horizontal_alignment_text, Point::new(0, 6))
            .into_styled(character_style)
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
        if !demo_loop(&mut window, &mut bounds, Justified) {
            break 'running;
        }
        if !demo_loop(&mut window, &mut bounds, LeftAligned) {
            break 'running;
        }
        if !demo_loop(&mut window, &mut bounds, CenterAligned) {
            break 'running;
        }
        if !demo_loop(&mut window, &mut bounds, RightAligned) {
            break 'running;
        }
    }
}
