//! This example demonstrates text styling using in-line ANSI escape sequences.

use embedded_graphics::{fonts::Font6x8, pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_text::prelude::*;

fn main() {
    let text = format!(
        "{comment}/// Comment\n\
        {base_text}#[{attribute}derive{base_text}(Debug)]\n\
        {keyword}enum {type_name}{underlined}Foo{underlined_off}{base_text}<{lifetime}'a{base_text}> {{\n\
        {comment}\t/// Decide what {strikethrough}not{strikethrough_off} to do next.\n\
        {highlighted_background}\t{enum_variant}Bar{base_text}({type_name}{underlined}Token{underlined_off}{base_text}<{lifetime}'a{base_text}>),{end_of_line}\n\
        {line_background}{base_text}}}",
        // Name the ANSI escape sequences we use for styling
        line_background = "\x1b[48;5;16m",
        highlighted_background = "\x1b[48;5;235m",
        enum_variant = "\x1b[38;2;36;144;241m",
        keyword = "\x1b[38;2;84;128;166m",
        comment = "\x1b[38;2;94;153;73m",
        base_text = "\x1b[97m",
        attribute ="\x1b[38;2;220;220;157m",
        type_name = "\x1b[38;2;78;201;176m",
        lifetime = "\x1b[38;2;84;128;166m",
        end_of_line = "\x1b[40C",
        underlined = "\x1b[4m",
        underlined_off = "\x1b[24m",
        strikethrough = "\x1b[9m",
        strikethrough_off = "\x1b[29m",
    );

    // Specify the styling options:
    // * Use the 6x8 MonoFont from embedded-graphics.
    // * Draw the text horizontally left aligned (default option, not specified here).
    // * Draw the text with black, which will be overridden by in-line styling.
    // * Use 2px line spacing because we'll draw underlines.
    let textbox_style = TextBoxStyleBuilder::new(Font6x8)
        .text_color(Rgb888::BLACK)
        .line_spacing(2)
        .build();

    // Specify the bounding box.
    let bounds = Rectangle::new(Point::zero(), Size::new(241, 97));

    // Create the text box and apply styling options.
    let text_box = TextBox::new(&text, bounds).into_styled(textbox_style);

    // Create a simulated display with the dimensions of the text box.
    let mut display = SimulatorDisplay::new(text_box.bounding_box().size);

    // Draw the text box.
    text_box.draw(&mut display).unwrap();

    // Set up the window and show the display's contents.
    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    Window::new("In-line styling example", &output_settings).show_static(&display);
}
