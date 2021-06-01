//! Example: interactive demonstration.
//!
//! This example draws text into a bounding box that can be modified by
//! clicking and dragging on the display.
//!
//! Press H or V to switch between different horizontal alignment modes.
//! Press M to cycle through different height modes.

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
    text::Text,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_text::{
    alignment::{HorizontalAlignment, VerticalAlignment},
    style::{HeightMode, TextBoxStyle, VerticalOverdraw},
    TextBox,
};
use sdl2::keyboard::Keycode;
use std::{convert::Infallible, thread, time::Duration};

enum ProcessedEvent {
    Nothing,
    Quit,
    NextHorizontal,
    NextVertical,
    NextMode,
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
                SimulatorEvent::KeyDown { keycode, .. } if keycode == Keycode::H => {
                    ProcessedEvent::NextHorizontal
                }
                SimulatorEvent::KeyDown { keycode, .. } if keycode == Keycode::V => {
                    ProcessedEvent::NextVertical
                }
                SimulatorEvent::KeyDown { keycode, .. } if keycode == Keycode::M => {
                    ProcessedEvent::NextMode
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

    let text = "Press H to change horizontal alignment.\n\
    Press V to change vertical alignment.\n\
    Press M to change height mode.\n\n\
    Lorem Ipsum is simply dummy text of the printing and typesetting industry. \
    Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
    an unknown printer took a galley of type and scrambled it to make a type specimen book.\n\
    super\u{AD}cali\u{AD}fragi\u{AD}listic\u{AD}espeali\u{AD}docious";

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let mut textbox_style = TextBoxStyle::default();

    let mut bounds = Rectangle::new(Point::new(1, 34), Size::new(128, 200));

    'demo: loop {
        // Create a simulated display.
        let mut display = SimulatorDisplay::new(Size::new(255, 255));

        // Create the text box and apply styling options.
        let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);
        text_box.draw(&mut display)?;

        // Draw the bounding box of the text box.
        text_box
            .bounds
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .stroke_alignment(StrokeAlignment::Outside)
                    .stroke_color(BinaryColor::On)
                    .stroke_width(1)
                    .build(),
            )
            .draw(&mut display)?;

        // Display the name of the current alignment modes above the text box.
        Text::new(
            &format!(
                "Horizontal: {:?}\nVertical: {:?}\nHeight mode: {:?}",
                textbox_style.alignment,
                textbox_style.vertical_alignment,
                textbox_style.height_mode
            ),
            Point::new(0, 8),
            character_style,
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
                ProcessedEvent::NextHorizontal => {
                    textbox_style.alignment = match textbox_style.alignment {
                        HorizontalAlignment::Left => HorizontalAlignment::Center,
                        HorizontalAlignment::Center => HorizontalAlignment::Right,
                        HorizontalAlignment::Right => HorizontalAlignment::Justified,
                        HorizontalAlignment::Justified => HorizontalAlignment::Left,
                    }
                }
                ProcessedEvent::NextVertical => {
                    textbox_style.vertical_alignment = match textbox_style.vertical_alignment {
                        VerticalAlignment::Top => VerticalAlignment::Middle,
                        VerticalAlignment::Middle => VerticalAlignment::Bottom,
                        VerticalAlignment::Bottom => VerticalAlignment::Scrolling,
                        VerticalAlignment::Scrolling => VerticalAlignment::Top,
                    }
                }
                ProcessedEvent::NextMode => {
                    textbox_style.height_mode = match textbox_style.height_mode {
                        HeightMode::Exact(VerticalOverdraw::FullRowsOnly) => {
                            HeightMode::Exact(VerticalOverdraw::Visible)
                        }
                        HeightMode::Exact(VerticalOverdraw::Visible) => {
                            HeightMode::Exact(VerticalOverdraw::Hidden)
                        }
                        HeightMode::Exact(VerticalOverdraw::Hidden) => {
                            HeightMode::ShrinkToText(VerticalOverdraw::FullRowsOnly)
                        }
                        HeightMode::ShrinkToText(VerticalOverdraw::FullRowsOnly) => {
                            HeightMode::ShrinkToText(VerticalOverdraw::Visible)
                        }
                        HeightMode::ShrinkToText(VerticalOverdraw::Visible) => {
                            HeightMode::ShrinkToText(VerticalOverdraw::Hidden)
                        }
                        HeightMode::ShrinkToText(VerticalOverdraw::Hidden) => HeightMode::FitToText,
                        HeightMode::FitToText => HeightMode::Exact(VerticalOverdraw::FullRowsOnly),
                    }
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
