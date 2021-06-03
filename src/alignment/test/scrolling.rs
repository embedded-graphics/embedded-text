use embedded_graphics::{
    mock_display::MockDisplay,
    mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
};

use crate::{
    alignment::VerticalAlignment,
    style::{HeightMode, TextBoxStyle, TextBoxStyleBuilder, VerticalOverdraw},
    utils::test::size_for,
    TextBox,
};

fn assert_rendered(text: &str, size: Size, pattern: &[&str]) {
    let mut display = MockDisplay::new();

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .build();

    let style = TextBoxStyle::with_vertical_alignment(VerticalAlignment::Scrolling);

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
fn scrolling_behaves_as_top_if_lines_dont_overflow() {
    assert_rendered(
        "word",
        size_for(&FONT_6X9, 4, 2),
        &[
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
            "                        ",
        ],
    );
}

#[test]
fn scrolling_behaves_as_bottom_if_lines_overflow() {
    assert_rendered(
        "word word2 word3 word4",
        size_for(&FONT_6X9, 5, 2),
        &[
            "..............................",
            "......................#..####.",
            "......................#....#..",
            "#...#...##...#.#....###...##..",
            "#.#.#..#..#..##.#..#..#.....#.",
            "#.#.#..#..#..#.....#..#.....#.",
            ".#.#....##...#......###..###..",
            "..............................",
            "..............................",
            "..............................",
            "......................#....#..",
            "......................#...##..",
            "#...#...##...#.#....###..#.#..",
            "#.#.#..#..#..##.#..#..#.#..#..",
            "#.#.#..#..#..#.....#..#.#####.",
            ".#.#....##...#......###....#..",
            "..............................",
            "..............................",
        ],
    );
}

#[test]
fn scrolling_applies_full_rows_vertical_overflow() {
    assert_rendered(
        "word word2 word3 word4",
        size_for(&FONT_6X9, 5, 2) - Size::new(0, 5),
        &[
            "                              ",
            "                              ",
            "                              ",
            "                              ",
            "..............................",
            "......................#....#..",
            "......................#...##..",
            "#...#...##...#.#....###..#.#..",
            "#.#.#..#..#..##.#..#..#.#..#..",
            "#.#.#..#..#..#.....#..#.#####.",
            ".#.#....##...#......###....#..",
            "..............................",
            "..............................",
        ],
    );
}

#[test]
fn scrolling_applies_hidden_vertical_overflow() {
    let mut display = MockDisplay::new();

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .build();

    let style = TextBoxStyleBuilder::new()
        .vertical_alignment(VerticalAlignment::Scrolling)
        .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
        .build();

    TextBox::with_textbox_style(
        "word word2 word3 word4",
        Rectangle::new(Point::zero(), size_for(&FONT_6X9, 5, 2) - Size::new(0, 5)),
        character_style,
        style,
    )
    .draw(&mut display)
    .unwrap();

    display.assert_pattern(&[
        "#.#.#..#..#..#.....#..#.....#.",
        ".#.#....##...#......###..###..",
        "..............................",
        "..............................",
        "..............................",
        "......................#....#..",
        "......................#...##..",
        "#...#...##...#.#....###..#.#..",
        "#.#.#..#..#..##.#..#..#.#..#..",
        "#.#.#..#..#..#.....#..#.#####.",
        ".#.#....##...#......###....#..",
        "..............................",
        "..............................",
    ]);
}
