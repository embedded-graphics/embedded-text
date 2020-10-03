# embedded-text [![crates.io](https://img.shields.io/crates/v/embedded_text.svg)](https://crates.io/crates/embedded_text) [![docs.rs](https://docs.rs/embedded-text/badge.svg)](https://docs.rs/embedded-text/) ![Rust](https://github.com/bugadani/embedded-text/workflows/Rust/badge.svg)

TextBox for embedded-graphics.

This crate provides a configurable `TextBox` to render multiline text inside a bounding
`Rectangle` using [embedded-graphics].

`TextBox` supports the common text alignments:
 - Horizontal:
     - `LeftAligned`
     - `RightAligned`
     - `CenterAligned`
     - `Justified`
 - Vertical:
     - `TopAligned`
     - `CenterAligned`
     - `BottomAligned`

`TextBox` also supports some special characters not handled by embedded-graphics' `Text`:
 - non-breaking space (`\u{200b}`)
 - zero-width space (`\u{a0}`)
 - soft hyphen (`\u{ad}`)
 - carriage return (`\r`)
 - tab (`\t`) with configurable tab size

`TextBox` also supports text coloring using [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code).

### Example

The examples are based on [the embedded-graphics simulator]. The simulator is built on top of
`SDL2`. See the [simulator README] for more information.

![embedded-text example with center aligned text](https://raw.githubusercontent.com/bugadani/embedded-text/master/assets/center.png)

![embedded-text example with colored text](https://raw.githubusercontent.com/bugadani/embedded-text/master/assets/colored_text.png)

```rust
use embedded_graphics::{fonts::Font6x8, pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::prelude::*;

fn main() -> Result<(), core::convert::Infallible> {
    let text = "Hello, World!\nLorem Ipsum is simply dummy text of the printing and typesetting \
    industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
    an unknown printer took a galley of type and scrambled it to make a type specimen book.";

    let textbox_style = TextBoxStyleBuilder::new(Font6x8)
        .alignment(CenterAligned)
        .height_mode(FitToText)
        .text_color(BinaryColor::On)
        .build();

    let text_box = TextBox::new(text, Rectangle::new(Point::zero(), Point::new(128, 0)))
        .into_styled(textbox_style);

    // Create a window just tall enough to fit the text.
    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(text_box.size());
    text_box.draw(&mut display).unwrap();

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
