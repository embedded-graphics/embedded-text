//! This example demonstrates middleware that affects styling.

use std::{thread, time::Duration};

use ansi_parser::AnsiSequence;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_text::{
    alignment::HorizontalAlignment,
    middleware::{Middleware, ProcessingState},
    style::TextBoxStyleBuilder,
    TextBox, Token,
};
use heapless::Vec;

#[derive(Clone)]
struct Underliner<'a> {
    underlined: bool,
    current_token: Option<Token<'a>>,
}

impl<'a> Underliner<'a> {
    fn new() -> Self {
        Self {
            underlined: false,
            current_token: None,
        }
    }
}

impl<'a> Middleware<'a> for Underliner<'a> {
    fn next_token(
        &mut self,
        state: ProcessingState,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        let token = if let Some(token) = self.current_token.take() {
            Some(token)
        } else {
            next_token.next()
        };

        match token {
            Some(Token::Word(w)) => {
                if let Some(pos) = w.find('_') {
                    if pos == 0 {
                        self.current_token = Some(Token::Word(&w[1..]));

                        if state == ProcessingState::Render {
                            if self.underlined {
                                self.underlined = false;
                                Some(Token::EscapeSequence(AnsiSequence::SetGraphicsMode(
                                    Vec::from_slice(&[24]).unwrap(),
                                )))
                            } else {
                                self.underlined = true;
                                Some(Token::EscapeSequence(AnsiSequence::SetGraphicsMode(
                                    Vec::from_slice(&[4]).unwrap(),
                                )))
                            }
                        } else {
                            Some(Token::Word(""))
                        }
                    } else {
                        let prefix = &w[0..pos];
                        self.current_token = Some(Token::Word(&w[pos..]));
                        Some(Token::Word(prefix))
                    }
                } else {
                    Some(Token::Word(w))
                }
            }
            token => token,
        }
    }
}

fn main() {
    let text = "Hello, World!\n\
    Lorem Ipsum is _simply_ dummy text of the printing and typesetting industry. \
    Lorem Ipsum has been the industry's _standard_ dummy text ever since the 1500s, when \
    an unknown printer _took a galley of type and scrambled it_ to make a type specimen book.";

    // Set up the window.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let mut window = Window::new("TextBox demonstration", &output_settings);

    let mut chars: u32 = 1;
    loop {
        // Create a simulated display.
        let mut display = SimulatorDisplay::new(Size::new(128, 140));

        let character_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        let textbox_style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Justified)
            .build();

        chars = chars.saturating_add(1);

        // Specify the bounding box. Note the 0px height. The `FitToText` height mode will
        // measure and adjust the height of the text box in `into_styled()`.
        let bounds = Rectangle::new(Point::zero(), display.size());

        // Create and draw the text boxes.
        // TODO: setter methods
        let mut tb = TextBox::with_middleware(text, bounds, character_style, Underliner::new());
        tb.style = textbox_style;

        tb.draw(&mut display).unwrap();

        // Update the window.
        window.update(&display);

        // Handle key and mouse events.
        for event in window.events() {
            if event == SimulatorEvent::Quit {
                return;
            }
        }

        // Wait for a little while.
        thread::sleep(Duration::from_millis(100));
    }
}
