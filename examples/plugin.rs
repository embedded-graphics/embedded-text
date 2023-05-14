//! # Example: plugin
//!
//! This example demonstrates a simple plugin that simulates typing input.
//! The plugin itself limits the number of characters printed. The number of printed characters
//! is incremented in each frame.

use std::{thread, time::Duration};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_text::{
    alignment::HorizontalAlignment,
    plugin::{tail::Tail, Plugin},
    style::{HeightMode, TextBoxStyleBuilder, VerticalOverdraw},
    TextBox, Token,
};

trait StrExt {
    fn first_n_chars<'a>(&'a self, n: u32) -> &'a str;
}

impl StrExt for str {
    fn first_n_chars<'a>(&'a self, n: u32) -> &'a str {
        if let Some((i, (idx, _))) = self.char_indices().enumerate().take(n as usize + 1).last() {
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

#[derive(Clone)]
struct CharacterLimiter {
    characters: u32,
    measured: u32,
    rendered: u32,
    last_line: bool,
}

impl CharacterLimiter {
    fn new(characters: u32) -> Self {
        Self {
            characters,
            measured: 0,
            rendered: 0,
            last_line: false,
        }
    }
}

// This implementation does not work with Justified text. Justified text needs to be able to measure
// lines of text, but line delimiting does not work nice with lookahead tokens.
// Plugin already returned the next token before we knew it belonged to the next line.
impl<'a, C> Plugin<'a, C> for CharacterLimiter
where
    C: PixelColor,
{
    fn new_line(&mut self) {
        self.last_line = self.measured > self.characters;
    }

    fn next_token(
        &mut self,
        mut next_token: impl FnMut() -> Option<Token<'a, C>>,
    ) -> Option<Token<'a, C>> {
        if self.last_line {
            return None;
        }

        let token = next_token();
        match token {
            Some(Token::Whitespace(_, _)) => {
                // Don't count whitespaces - results in better effect.
                token
            }
            Some(Token::Word(word)) => {
                self.measured += word.chars().count() as u32;

                Some(Token::Word(word))
            }
            Some(Token::Break(_, _)) => {
                self.measured += 1;
                token
            }
            token => token,
        }
    }

    fn render_token(&mut self, token: Token<'a, C>) -> Option<Token<'a, C>> {
        if self.measured <= self.characters {
            self.rendered = self.measured;
            return Some(token);
        }

        let to_render = self.characters.saturating_sub(self.rendered);
        if to_render == 0 {
            return None;
        }
        self.rendered = self.measured;

        match token {
            Token::Whitespace(n, s) => {
                let to_render = n.min(to_render);
                Some(Token::Whitespace(to_render, s.first_n_chars(to_render)))
            }
            Token::Word(s) => Some(Token::Word(s.first_n_chars(to_render))),
            Token::Break(repl, orig) => Some(Token::Break(repl.first_n_chars(to_render), orig)),
            _ => Some(token),
        }
    }
}

fn main() {
    let text = "Hello, World!\n\
    Lorem Ipsum is simply dummy text of the printing and typesetting industry. \
    Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
    an unknown printer took a galley of type and scrambled it to make a type specimen book.";

    // Set up the window.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .scale(2)
        .build();
    let mut window = Window::new("TextBox demonstration", &output_settings);

    let mut chars: u32 = 1;
    loop {
        // Create a simulated display.
        let mut display = SimulatorDisplay::new(Size::new(128, 64));

        let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let textbox_style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Justified)
            .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
            .build();

        chars = chars.saturating_add(1);

        // Specify the bounding box. Note the 0px height. The `FitToText` height mode will
        // measure and adjust the height of the text box in `into_styled()`.
        let bounds = Rectangle::new(Point::zero(), Size::new(128, 64));

        // Create and draw the text boxes.
        TextBox::with_textbox_style(text, bounds, character_style, textbox_style)
            .add_plugin(CharacterLimiter::new(chars))
            .add_plugin(Tail)
            .draw(&mut display)
            .unwrap();

        // Update the window.
        window.update(&display);

        // Handle key and mouse events.
        for event in window.events() {
            if event == SimulatorEvent::Quit {
                return;
            }
        }

        // Wait for a little while.
        thread::sleep(Duration::from_millis(25));
    }
}
