//! # Example: styling using middleware.
//!
//! This example demonstrates middleware that affects styling.

use ansi_parser::AnsiSequence;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::{
    alignment::HorizontalAlignment,
    middleware::{Middleware, ProcessingState},
    style::TextBoxStyle,
    TextBox, Token,
};
use heapless::Vec;
use std::convert::Infallible;

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

fn main() -> Result<(), Infallible> {
    let text = "Hello, World!\n\
    Lorem Ipsum is _simply_ dummy text of the printing and typesetting industry. \
    Lorem Ipsum has been the industry's _standard_ dummy text ever since the 1500s, when \
    an unknown printer _took a galley of type and scrambled it_ to make a type specimen book.";

    // Create a simulated display.
    let mut display = SimulatorDisplay::new(Size::new(128, 140));

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let textbox_style = TextBoxStyle::with_alignment(HorizontalAlignment::Justified);

    // Specify the bounding box. Note the 0px height. The `FitToText` height mode will
    // measure and adjust the height of the text box in `into_styled()`.
    let bounds = Rectangle::new(Point::zero(), display.size());

    // Create and draw the text boxes.
    TextBox::with_textbox_style(text, bounds, character_style, textbox_style)
        .add_middleware(Underliner::new())
        .draw(&mut display)?;

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .scale(2)
        .build();
    Window::new("TextBox middleware demonstration", &output_settings).show_static(&display);

    Ok(())
}
