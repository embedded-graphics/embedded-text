//! # Example: editor
//!
//! This example demonstrates a simple text "editor" that lets you type and delete characters.
//!
//! The demo uses the "Scrolling" vertical layout which is especially useful for
//! editor type applications.
use embedded_graphics::{
    geometry::AnchorPoint,
    mono_font::{iso_8859_2::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::{renderer::TextRenderer, Baseline},
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_text::{
    alignment::VerticalAlignment, middleware::Middleware, style::TextBoxStyle, TextBox,
};
use sdl2::keyboard::{Keycode, Mod};
use std::{collections::HashMap, convert::Infallible, thread, time::Duration};

trait StrExt {
    fn first_n_chars<'a>(&'a self, n: usize) -> &'a str;
}

impl StrExt for str {
    fn first_n_chars<'a>(&'a self, n: usize) -> &'a str {
        if let Some((i, (idx, _))) = self.char_indices().enumerate().take(n + 1).last() {
            if i < n as usize {
                self
            } else {
                &self[0..idx]
            }
        } else {
            self
        }
    }
}

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

    pub fn delete_before(&mut self) {
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
            self.text.remove(self.cursor_offset);
        }
    }

    pub fn delete_after(&mut self) {
        if self.cursor_offset > 0 && self.cursor_offset < self.text.chars().count() {
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

    pub fn cursor_middleware<C: PixelColor>(&self, color: C) -> EditorMiddleware<C> {
        EditorMiddleware {
            current_offset: 0,
            cursor_offset: self.cursor_offset,
            color,
            cursor_drawn: false,
        }
    }
}

#[derive(Clone, Copy)]
struct EditorMiddleware<C> {
    current_offset: usize,
    cursor_offset: usize,
    color: C,
    cursor_drawn: bool,
}

impl<C: PixelColor> EditorMiddleware<C> {
    fn draw_cursor<D>(
        &self,
        draw_target: &mut D,
        bounds: Rectangle,
        pos: Point,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let style = PrimitiveStyle::with_stroke(self.color, 1);
        Line::new(
            pos + Point::new(0, 1),
            pos + Point::new(0, bounds.size.height as i32 - 1),
        )
        .into_styled(style)
        .draw(draw_target)
    }
}

impl<'a, C: PixelColor> Middleware<'a, C> for EditorMiddleware<C> {
    fn post_render_text<T, D>(
        &mut self,
        draw_target: &mut D,
        character_style: &T,
        text: &str,
        bounds: Rectangle,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer<Color = C>,
        D: DrawTarget<Color = T::Color>,
    {
        let len = text.chars().count();
        if self.cursor_offset >= self.current_offset
            && self.cursor_offset <= self.current_offset + len
            && !self.cursor_drawn
        {
            let chars_before = self.cursor_offset - self.current_offset;
            let str_before = text.first_n_chars(chars_before);
            let metrics =
                character_style.measure_string(str_before, bounds.top_left, Baseline::Top);
            let pos = metrics.bounding_box.anchor_point(AnchorPoint::TopRight);
            self.draw_cursor(draw_target, bounds, pos)?;
            self.cursor_drawn = true;
        }
        self.current_offset += len;
        Ok(())
    }

    fn post_render_whitespace<T, D>(
        &mut self,
        draw_target: &mut D,
        _character_style: &T,
        width: u32,
        count: u32,
        bounds: Rectangle,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer<Color = C>,
        D: DrawTarget<Color = T::Color>,
    {
        let count = count as usize;

        if self.cursor_offset >= self.current_offset
            && self.cursor_offset <= self.current_offset + count
            && !self.cursor_drawn
        {
            let chars_before = self.cursor_offset - self.current_offset;

            let pos =
                bounds.top_left + Point::new(((width as usize / count) * chars_before) as i32, 0);
            self.draw_cursor(draw_target, bounds, pos)?;
            self.cursor_drawn = true;
        }
        self.current_offset += count;
        Ok(())
    }

    fn post_line_start<T, D>(
        &mut self,
        draw_target: &mut D,
        character_style: &T,
        pos: Point,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer<Color = C>,
        D: DrawTarget<Color = C>,
    {
        if self.cursor_offset == self.current_offset && !self.cursor_drawn {
            let rect = Rectangle::new(Point::zero(), Size::new(0, character_style.line_height()));
            self.draw_cursor(draw_target, rect, pos)?;
            self.cursor_drawn = true;
        }

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
        .text_color(BinaryColor::On)
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
        .add_middleware(input.cursor_middleware(BinaryColor::On))
        .draw(&mut display)?;

        // Update the window.
        window.update(&display);

        // Handle key events.
        for event in window.events() {
            match event {
                SimulatorEvent::KeyDown {
                    keycode, keymod, ..
                } => match keycode {
                    Keycode::Backspace => input.delete_before(),

                    Keycode::Delete => input.delete_after(),
                    Keycode::Left => input.cursor_left(),

                    Keycode::Right => input.cursor_right(),

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
