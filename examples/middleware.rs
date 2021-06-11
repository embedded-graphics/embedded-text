//! # Example: middleware
//!
//! This example demonstrates a simple middleware that simulates typing input.
//! The middleware itself limits the number of characters printed. The number of printed characters
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
    alignment::{HorizontalAlignment, VerticalAlignment},
    middleware::Middleware,
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
    // We measure everything in the current line to avoid jumping.
    // This flag tells us if we have seen the last line and we can stop measuring.
    last_line_processed: bool,
}

impl CharacterLimiter {
    fn new(characters: u32) -> Self {
        Self {
            characters,
            measured: 0,
            rendered: 0,
            last_line_processed: false,
        }
    }
}

impl<'a> Middleware<'a> for CharacterLimiter {
    fn new_line(&mut self) {
        if self.measured > self.characters {
            self.last_line_processed = true;
        }
    }

    fn next_token_to_measure(
        &mut self,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        let token = next_token.next();

        if self.last_line_processed {
            return None;
        }

        match token {
            Some(Token::Word(word)) => {
                self.measured += word.chars().count() as u32;
            }
            Some(Token::Break(_, _)) => {
                self.measured += 1;
            }
            _ => {}
        };

        token
    }

    fn next_token_to_render(
        &mut self,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        let token = next_token.next();

        let chars_left = self.characters.saturating_sub(self.rendered);
        if chars_left == 0 {
            return None;
        }

        match token {
            Some(Token::Word(word)) => {
                let chars = chars_left.min(word.chars().count() as u32);
                self.rendered += chars;

                Some(Token::Word(word.first_n_chars(chars)))
            }
            Some(Token::Break(_, _)) => {
                self.rendered += 1;
                token
            }
            token => token,
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
        .build();
    let mut window = Window::new("TextBox demonstration", &output_settings);

    let mut chars: u32 = 1;
    loop {
        // Create a simulated display.
        let mut display = SimulatorDisplay::new(Size::new(128, 64));

        let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let textbox_style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Justified)
            .vertical_alignment(VerticalAlignment::Scrolling)
            .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
            .build();

        chars = chars.saturating_add(1);

        // Specify the bounding box. Note the 0px height. The `FitToText` height mode will
        // measure and adjust the height of the text box in `into_styled()`.
        let bounds = Rectangle::new(Point::zero(), Size::new(128, 64));

        // Create and draw the text boxes.
        // TODO: setter methods
        TextBox::with_textbox_style(text, bounds, character_style, textbox_style)
            .add_middleware(CharacterLimiter::new(chars))
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
        thread::sleep(Duration::from_millis(50));
    }
}
