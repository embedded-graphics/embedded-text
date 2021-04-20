//! This example demonstrates a simple text "editor" that lets you type and delete characters.
//!
//! The demo uses the "Scrolling" vertical layout which is especially useful for
//! editor type applications.
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_text::prelude::*;
use sdl2::keyboard::{Keycode, Mod};
use std::{collections::HashMap, thread, time::Duration};

trait Selector {
    /// Select inserted characters based on key modifiers.
    ///
    /// Some key combinations don't insert characters, so we have to work with strings.
    fn select_modified(&self, keymod: Mod) -> &str;
}

impl Selector for (&str, &str, &str, &str) {
    #[inline]
    fn select_modified(&self, keymod: Mod) -> &str {
        if keymod.contains(Mod::RALTMOD) {
            self.3
        } else if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
            self.1
        } else if keymod.contains(Mod::CAPSMOD) {
            self.2
        } else {
            self.0
        }
    }
}

fn main() {
    // Special characters are mapped as they appear on Hungarian layouts. Sorry 😅
    let inputs: HashMap<_, _> = [
        // (Keycode, (NO, SHIFT, CAPS, ALT_GR))
        (Keycode::A, ("a", "A", "A", "ä")),
        (Keycode::B, ("b", "B", "B", "{")),
        (Keycode::C, ("c", "C", "C", "&")),
        (Keycode::D, ("d", "D", "D", "Đ")),
        (Keycode::E, ("e", "E", "E", "Ä")),
        (Keycode::F, ("f", "F", "F", "[")),
        (Keycode::G, ("g", "G", "G", "]")),
        (Keycode::H, ("h", "H", "H", "")),
        (Keycode::I, ("i", "I", "I", "Í")),
        (Keycode::J, ("j", "J", "J", "í")),
        (Keycode::K, ("k", "K", "K", "ł")),
        (Keycode::L, ("l", "L", "L", "Ł")),
        (Keycode::M, ("m", "M", "M", "<")),
        (Keycode::N, ("n", "N", "N", "}")),
        (Keycode::O, ("o", "O", "O", "")),
        (Keycode::P, ("p", "P", "P", "")),
        (Keycode::Q, ("q", "Q", "Q", "\\")),
        (Keycode::R, ("r", "R", "R", "")),
        (Keycode::S, ("s", "S", "S", "đ")),
        (Keycode::T, ("t", "T", "T", "")),
        (Keycode::U, ("u", "U", "U", "€")),
        (Keycode::V, ("v", "V", "V", "@")),
        (Keycode::W, ("w", "W", "W", "|")),
        (Keycode::X, ("x", "X", "X", "#")),
        (Keycode::Y, ("y", "Y", "Y", ">")),
        (Keycode::Z, ("z", "Z", "Z", "")),
        (Keycode::Num0, ("0", "§", "0", "")),
        (Keycode::Num1, ("1", "'", "1", "~")),
        (Keycode::Num2, ("2", "\"", "2", "ˇ")),
        (Keycode::Num3, ("3", "+", "3", "^")),
        (Keycode::Num4, ("4", "!", "4", "˘")),
        (Keycode::Num5, ("5", "%", "5", "°")),
        (Keycode::Num6, ("6", "/", "6", "˛")),
        (Keycode::Num7, ("7", "=", "7", "`")),
        (Keycode::Num8, ("8", "(", "8", "˙")),
        (Keycode::Num9, ("9", ")", "9", "´")),
        (Keycode::Space, (" ", " ", " ", " ")),
        (Keycode::Comma, (",", "?", ",", " ")),
        (Keycode::Period, (".", ":", ".", ">")),
        (Keycode::Minus, ("-", "_", "-", "*")),
        (Keycode::Return, ("\n", "\n", "\n", "\n")),
        (Keycode::KpEnter, ("\n", "\n", "\n", "\n")),
    ]
    .iter()
    .cloned()
    .collect();

    // Specify the bounding box.
    let bounds = Rectangle::new(Point::zero(), Size::new(128, 64));

    // Specify the styling options:
    // * Use the 6x8 MonoFont from embedded-graphics.
    // * Draw the text horizontally left aligned (default option, not specified here).
    // * Use `Scrolling` vertical layout - this will make sure the cursor is always in view.
    // * Draw the text with `BinaryColor::On`, which will be displayed as light blue.
    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(BinaryColor::Off)
        .background_color(BinaryColor::On)
        .build();

    let textbox_style = TextBoxStyleBuilder::new()
        .vertical_alignment(Scrolling)
        .build();

    // Set up the window.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let mut window = Window::new("TextBox input demonstration", &output_settings);

    // Text buffer. The contents of this string will be modified while typing.
    let mut text = String::from("Hello, world!");

    'running: loop {
        // Display an underscore for the "cursor"
        let text_and_cursor = format!("{}\u{200b}_", text);

        // Create the text box and apply styling options.
        let text_box = TextBox::with_textbox_style(
            &text_and_cursor,
            bounds,
            character_style,
            textbox_style.clone(),
        );

        // Create a simulated display with the dimensions of the text box.
        let mut display = SimulatorDisplay::new(text_box.bounding_box().size);

        // Draw the text box.
        text_box.draw(&mut display).unwrap();

        // Update the window.
        window.update(&display);

        // Handle key events.
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,

                SimulatorEvent::KeyDown {
                    keycode, keymod, ..
                } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Backspace => {
                        text.pop();
                    }
                    _ => {
                        inputs.get(&keycode).map(|k| {
                            text += k.select_modified(keymod);
                        });
                    }
                },

                _ => {}
            }
        }

        // Wait for a little while.
        thread::sleep(Duration::from_millis(10));
    }
}
