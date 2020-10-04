use embedded_graphics::{fonts::Font6x8, pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_text::prelude::*;

fn main() -> Result<(), core::convert::Infallible> {
    let text = format!(
        "{comment}/// Comment\n\
        {base_text}#[{attribute}derive{base_text}(Debug)]\n\
        {keyword}enum {type_name}Foo{base_text}<{lifetime}'a{base_text}> {{\n\
        {comment}\t/// Decide what to do next.\n\
        {highlighted_background}\t{enum_variant}Bar{base_text}({type_name}Token{base_text}<{lifetime}'a{base_text}>),                      \n\
        {line_background}{base_text}}}",
        // colors
        line_background = "\x1b[48;5;16m",
        highlighted_background = "\x1b[48;5;235m",
        enum_variant = "\x1b[38;2;36;144;241m",
        keyword = "\x1b[38;2;84;128;166m",
        comment = "\x1b[38;2;94;153;73m",
        base_text = "\x1b[97m",
        attribute ="\x1b[38;2;220;220;157m",
        type_name = "\x1b[38;2;78;201;176m",
        lifetime = "\x1b[38;2;84;128;166m"
    );

    let textbox_style = TextBoxStyleBuilder::new(Font6x8)
        .text_color(Rgb888::BLACK)
        .build();

    let bounds = Rectangle::new(Point::zero(), Point::new(246, 128));
    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(bounds.size());

    TextBox::new(&text, bounds)
        .into_styled(textbox_style)
        .draw(&mut display)
        .unwrap();

    let output_settings = OutputSettingsBuilder::new().scale(3).build();
    Window::new("Hello TextBox with text background color", &output_settings).show_static(&display);
    Ok(())
}
