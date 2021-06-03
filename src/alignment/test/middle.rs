use embedded_graphics::{
    geometry::Point,
    mock_display::MockDisplay,
    mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::Size,
    primitives::Rectangle,
    Drawable,
};

use crate::{alignment::VerticalAlignment, style::TextBoxStyle, utils::test::size_for, TextBox};

fn assert_rendered(text: &str, size: Size, pattern: &[&str]) {
    let mut display = MockDisplay::new();

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .build();

    let style = TextBoxStyle::with_vertical_alignment(VerticalAlignment::Middle);

    TextBox::with_textbox_style(
        text,
        Rectangle::new(Point::zero(), size),
        character_style,
        style,
    )
    .draw(&mut display)
    .unwrap();

    display.assert_pattern(pattern);
}

#[test]
fn test_center_alignment() {
    assert_rendered(
        "word",
        size_for(&FONT_6X9, 4, 2),
        &[
            "                        ",
            "                        ",
            "                        ",
            "                        ",
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
        ],
    );
}
