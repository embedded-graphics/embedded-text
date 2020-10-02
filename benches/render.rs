use criterion::{black_box, criterion_group, criterion_main, Criterion};
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use embedded_text::{prelude::*, rendering::RendererFactory};

const TEXT: &str = "Benchmark text!";

fn benchmark_render_text(c: &mut Criterion) {
    let style = TextStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    c.bench_function("Text", |b| {
        b.iter(|| {
            let object = Text::new(black_box(TEXT), Point::zero()).into_styled(style);
            object.into_iter().collect::<Vec<Pixel<BinaryColor>>>()
        })
    });
}

fn benchmark_render_textbox(c: &mut Criterion) {
    let style = TextBoxStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    c.bench_function("TextBox", |b| {
        b.iter(|| {
            let obj = TextBox::new(
                black_box(TEXT),
                Rectangle::new(Point::zero(), Point::new(6 * 15 - 1, 7)),
            )
            .into_styled(style);
            let object = obj.create_renderer();
            object.collect::<Vec<Pixel<BinaryColor>>>()
        })
    });
}

fn benchmark_render_textbox_aligned(c: &mut Criterion) {
    let style = TextBoxStyleBuilder::new(Font6x8)
        .alignment(RightAligned)
        .text_color(BinaryColor::On)
        .build();

    c.bench_function("TextBox, RightAligned", |b| {
        b.iter(|| {
            let obj = TextBox::new(
                black_box(TEXT),
                Rectangle::new(Point::zero(), Point::new(6 * 15 - 1, 7)),
            )
            .into_styled(style);
            let object = obj.create_renderer();
            object.collect::<Vec<Pixel<BinaryColor>>>()
        })
    });
}

fn benchmark_render_textbox_vertical_aligned(c: &mut Criterion) {
    let style = TextBoxStyleBuilder::new(Font6x8)
        .vertical_alignment(BottomAligned)
        .text_color(BinaryColor::On)
        .build();

    c.bench_function("TextBox, BottomAligned", |b| {
        b.iter(|| {
            let obj = TextBox::new(
                black_box(TEXT),
                Rectangle::new(Point::zero(), Point::new(6 * 15 - 1, 7)),
            )
            .into_styled(style);
            let object = obj.create_renderer();
            object.collect::<Vec<Pixel<BinaryColor>>>()
        })
    });
}

fn benchmark_render_textbox_both_aligned(c: &mut Criterion) {
    let style = TextBoxStyleBuilder::new(Font6x8)
        .alignment(CenterAligned)
        .vertical_alignment(CenterAligned)
        .text_color(BinaryColor::On)
        .build();

    c.bench_function("TextBox, H/V CenterAligned", |b| {
        b.iter(|| {
            let obj = TextBox::new(
                black_box(TEXT),
                Rectangle::new(Point::zero(), Point::new(6 * 15 - 1, 7)),
            )
            .into_styled(style);
            let object = obj.create_renderer();
            object.collect::<Vec<Pixel<BinaryColor>>>()
        })
    });
}

criterion_group!(
    render,
    benchmark_render_text,
    benchmark_render_textbox,
    benchmark_render_textbox_aligned,
    benchmark_render_textbox_vertical_aligned,
    benchmark_render_textbox_both_aligned,
);
criterion_main!(render);
