# embedded-text [![crates.io](https://img.shields.io/crates/v/embedded_text.svg)](https://crates.io/crates/embedded_text) [![docs.rs](https://docs.rs/embedded-text/badge.svg)](https://docs.rs/embedded-text/) ![Rust](https://github.com/bugadani/embedded-text/workflows/Rust/badge.svg)

TextBox for embedded-graphics

This crate provides a configurable `TextBox` to render multiline text inside a bounding
`Rectangle` using [embedded-graphics].

`TextBox` supports the common text alignments:
 - `LeftAligned`
 - `RightAligned`
 - `CenterAligned`
 - `Justified`

### Example

The examples are based on [the embedded-graphics simulator]. The simulator is built on top of
`SDL2`. See the [simulator README] for more information.

![embedded-text example with center aligned text](https://raw.githubusercontent.com/bugadani/embedded-text/master/assets/center.png)

```rust
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};

use embedded_graphics::{
    fonts::Font6x8, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};

use embedded_text::{alignment::CenterAligned, style::TextBoxStyleBuilder, TextBox};

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(129, 129));

    let textbox_style = TextBoxStyleBuilder::new(Font6x8)
        .alignment(CenterAligned)
        .text_color(BinaryColor::On)
        .build();

    TextBox::new(
        "Hello, World!\nLorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.",
        Rectangle::new(Point::zero(), Point::new(128, 128)),
    )
    .into_styled(textbox_style)
    .draw(&mut display)
    .unwrap();

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello center aligned TextBox", &output_settings).show_static(&display);
    Ok(())
}
```

[embedded-graphics]: https://github.com/jamwaffles/embedded-graphics/
[the embedded-graphics simulator]: https://github.com/jamwaffles/embedded-graphics/tree/master/simulator
[simulator README]: https://github.com/jamwaffles/embedded-graphics/tree/master/simulator#usage-without-sdl2

## Development setup

### Minimum supported Rust version
The minimum supported Rust version for embedded-text is 1.40.0 or greater. Ensure you have the latest stable version of Rust installed, preferably through https://rustup.rs.

### Installation

For setup in general, follow the installation instructions for [embedded-graphics].

To install SDL2 on Windows, see https://github.com/Rust-SDL2/rust-sdl2#windows-msvc

## Attribution

The example text is copied from https://www.lipsum.com
