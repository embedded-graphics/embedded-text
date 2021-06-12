//! # Example: editor
//!
//! This example demonstrates a simple text "editor" that lets you type and delete characters.
//!
//! The demo uses the "Scrolling" vertical layout which is especially useful for
//! editor type applications.
use embedded_graphics::{
    mono_font::{iso_8859_2::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    text::renderer::TextRenderer,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_text::{
    alignment::VerticalAlignment, middleware::Middleware, style::TextBoxStyle, TextBox,
};
use sdl2::keyboard::{Keycode, Mod};
use std::{collections::HashMap, convert::Infallible, thread, time::Duration};

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

struct EditorInput {
    pub text: String,
    cursor_offset: usize,
}

impl EditorInput {
    pub fn new(text: &str) -> Self {
        Self {
            cursor_offset: text.len(),
            text: text.to_owned(),
        }
    }

    pub fn insert(&mut self, s: &str) {
        self.text.insert_str(self.cursor_offset, s);
        self.cursor_offset += s.len();
    }

    pub fn delete_one(&mut self) {
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
            self.text.remove(self.cursor_offset);
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_offset < self.text.len() {
            self.cursor_offset += 1;
        }
    }
}

impl<'a> Middleware<'a> for EditorInput {
    fn post_render_text<T, D>(
        &mut self,
        _draw_target: &mut D,
        _character_style: &T,
        _text: &str,
        _bounds: Rectangle,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer,
        D: DrawTarget<Color = T::Color>,
    {
        Ok(())
    }

    fn post_render_whitespace<T, D>(
        &mut self,
        _draw_target: &mut D,
        _character_style: &T,
        _width: u32,
        _count: u32,
        _bounds: Rectangle,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer,
        D: DrawTarget<Color = T::Color>,
    {
        Ok(())
    }
}

fn main() -> Result<(), Infallible> {
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

    // Set up the window.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .scale(2)
        .build();
    let mut window = Window::new("Interactive TextBox input demonstration", &output_settings);

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::Off)
        .background_color(BinaryColor::On)
        .build();

    let textbox_style = TextBoxStyle::with_vertical_alignment(VerticalAlignment::Scrolling);
    let mut input = EditorInput::new("Hello, World!");

    'demo: loop {
        // Create a simulated display with the dimensions of the text box.
        let mut display = SimulatorDisplay::new(Size::new(128, 64));

        // Display an underscore for the "cursor"
        // Create the text box and apply styling options.
        TextBox::with_textbox_style(
            &input.text,
            display.bounding_box(),
            character_style,
            textbox_style,
        )
        .add_middleware(&input)
        .draw(&mut display)?;

        // Update the window.
        window.update(&display);

        // Handle key events.
        for event in window.events() {
            match event {
                SimulatorEvent::KeyDown {
                    keycode, keymod, ..
                } => match keycode {
                    Keycode::Backspace => {
                        input.delete_one();
                    }
                    Keycode::Left => {
                        input.cursor_left();
                    }
                    Keycode::Right => {
                        input.cursor_right();
                    }
                    _ => {
                        if let Some(k) = inputs.get(&keycode) {
                            input.insert(k.select_modified(keymod));
                        }
                    }
                },

                SimulatorEvent::Quit => break 'demo,
                _ => {}
            }
        }

        // Wait for a little while.
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}
