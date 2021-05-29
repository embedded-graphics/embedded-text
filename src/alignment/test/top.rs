use embedded_graphics::{
    mock_display::MockDisplay,
    mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
};

use crate::{alignment::VerticalAlignment, style::TextBoxStyleBuilder, TextBox};

#[test]
fn test_top_alignment() {
    let mut display = MockDisplay::new();

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .build();

    let style = TextBoxStyleBuilder::new()
        .vertical_alignment(VerticalAlignment::Top)
        .build();

    TextBox::with_textbox_style(
        "word",
        Rectangle::new(Point::zero(), Size::new(55, 16)),
        character_style,
        style,
    )
    .draw(&mut display)
    .unwrap();

    display.assert_pattern(&[
        "........................",
        "......................#.",
        "......................#.",
        "#...#...##...#.#....###.",
        "#.#.#..#..#..##.#..#..#.",
        "#.#.#..#..#..#.....#..#.",
        ".#.#....##...#......###.",
        "........................",
        "........................",
        "                        ",
        "                        ",
        "                        ",
        "                        ",
        "                        ",
        "                        ",
        "                        ",
        "                        ",
    ]);
}
