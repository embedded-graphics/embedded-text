0.7.0 (2023-11-03)
==================

 - [#165] Relaxed trait bounds on PixelColor

[#165]: https://github.com/embedded-graphics/embedded-text/pull/165

0.6.6 (2023-10-15)
==================

## Changed:

 - [#164] Internal improvements

[#164]: https://github.com/embedded-graphics/embedded-text/pull/164

0.6.5 (2023-10-13)
==================

## Changed:

 - [#163] Internal improvements

[#163]: https://github.com/embedded-graphics/embedded-text/pull/163

0.6.4 (2023-10-10)
==================

## Changed:

 - [#162] Internal improvements

[#162]: https://github.com/embedded-graphics/embedded-text/pull/162

0.6.3 (2023-10-09)
==================

## Changed:

 - [#161] Internal improvements

[#161]: https://github.com/embedded-graphics/embedded-text/pull/161

0.6.2 (2023-09-04)
==================

## Changed:

 - [#160] Internal improvements

[#160]: https://github.com/embedded-graphics/embedded-text/pull/160

0.6.1 (2023-09-01)
==================

## Changed:

 - [#159] Internal improvements

[#159]: https://github.com/embedded-graphics/embedded-text/pull/159

0.6.0 (2023-05-14)
=======================

## Changed:

 - [#158] The following functions can now be called from `const` code:
   - `Ansi::new()`
 - [#158] The following types are now `Copy`:
   - `Tail`
 - [#158] **breaking** Update embedded-graphics dependency to 0.8.
 - [#158] **breaking** Bump Minimum Supported Rust Version (MSRV) to 1.61.
 - [#158] **breaking** The `ansi` feature is no longer enabled by default

[#158]: https://github.com/embedded-graphics/embedded-text/pull/158

0.5.0 (2021-09-08)
==================

## Fixed:

 * [#157] Fixed carrying whitespace to the next line

## Removed:

 * [#155] Removed `TextBox::fit_height` and `TextBox::fit_height_limited`

[#155]: https://github.com/embedded-graphics/embedded-text/pull/155
[#157]: https://github.com/embedded-graphics/embedded-text/pull/157

0.5.0-beta.4 (2021-08-16)
=========================

## Added:

 * [#148] Added `TextBoxStyleBuilder::leading_spaces()` and `TextBoxStyleBuilder::trailing_spaces()`
 * [#148] Added `TextBoxStyle::leading_spaces` and `TextBoxStyle::trailing_spaces`

## Fixed:

 * [#143] Fixed measuring text height with active plugins.
 * [#144] Fixed a word wrapping issue

## Removed:

 * [#150] Removed unused `as-slice` dependency

[#143]: https://github.com/embedded-graphics/embedded-text/pull/143
[#144]: https://github.com/embedded-graphics/embedded-text/pull/144
[#148]: https://github.com/embedded-graphics/embedded-text/pull/148
[#150]: https://github.com/embedded-graphics/embedded-text/pull/150

0.5.0-beta.3 (2021-08-08)
=========================

## Added:

 * [#133] Added the following `const` functions:
   - `TextBoxStyle::default()`
   - `TextBoxStyleBuilder::default()`
   - `TabSize::default()`
 * [#134] `Tail` plugin
 * [#135] Allow using the built-in plugins without the `plugin` feature.
 * [#137] Allow using multiple plugins.
 * [#136] `Cursor` is now public.
 * [#136] Added `TextBox::take_plugins()`.
 * [#138] `Ansi` plugin to parse ANSI escape sequences.
 * [#138] `Token::MoveCursor` and `Token::ChangeTextStyle`

## Changed:

 * **breaking** [#133] `TextBoxStyle` and `TextBoxStyleBuilder` no longer implement the `Default` trait.
 * [#136] Replaced `TextBoxProperties::box_height` with  in `TextBoxProperties::bounding_box`.
 * [#136] Reworked the editor example to support vertical cursor movement and mouse input.

## Fixed:

 * [#140] Fixed an issue where, under certain circumstances no text was rendered.

## Removed:

 * [#134] `Scrolling` vertical alignment

[#133]: https://github.com/embedded-graphics/embedded-text/pull/133
[#134]: https://github.com/embedded-graphics/embedded-text/pull/134
[#135]: https://github.com/embedded-graphics/embedded-text/pull/135
[#136]: https://github.com/embedded-graphics/embedded-text/pull/136
[#137]: https://github.com/embedded-graphics/embedded-text/pull/137
[#138]: https://github.com/embedded-graphics/embedded-text/pull/138
[#140]: https://github.com/embedded-graphics/embedded-text/pull/140

0.5.0-beta.2 (2021-07-10)
==========================

## Added:

 * [#130] `ChangeTextStyle` token
 * [#114] Added experimental plugin support via the `plugin` Cargo feature. Plugin can be used to modify `TextBox` behaviour.

## Changed:

 * **breaking** [#125] Raised MSRV to `1.46.0`
 * **breaking** [#125] Chaneged how text height is measured.
 
## Removed:

* [#114] `TextBox` no longer implements `PartialEq` and `Eq`

[#114]: https://github.com/embedded-graphics/embedded-text/pull/114
[#125]: https://github.com/embedded-graphics/embedded-text/pull/125
[#130]: https://github.com/embedded-graphics/embedded-text/pull/130

0.5.0-beta.1 (2021-06-04)
==========================

## Changed:

 * **breaking** Replaced `{Horizontal, Vertical}TextAlignment` traits with `{Horizontal, Vertical}Alignment` enums. Vertical centering option has been renamed to `Middle`.
 * **breaking** Moved `VerticalOverdraw` and `HeightMode` to the `style` module.
 * **breaking** Replaced type-state `VerticalOverdraw`, `HeightMode` with an enum.
 * **breaking** Replaced `TextStyle::line_spacing` with `line_height`.
 * **breaking** Added `#[non_exhaustive]` to `TextBoxStyle`.
 * **breaking** Need to pass character style to text box constructors.
 * **breaking** Changed `TextBoxStyleBuilder` API to better align with embedded-graphics' `TextStyleBuilder`.
 * **breaking** Changed measurement of lines that only contain whitespace.
 * **breaking** (developer-facing) Simplified `HorizontalTextAlignment` API.
 * **breaking** (developer-facing) Split off `LineCursor` from `Cursor`.
 * **breaking** The `TabSize` type is now an enum and doesn't depend on a font.
 * **breaking** Raised MSRV to 1.43.
 * **breaking** Updated to embedded-graphics 0.7.
    Changes in embedded-graphics required changing the type signatures of almost every embedded-text type. For example, former `Font` and `PixelColor` type bounds have been replaced by `TextRenderer`, `CharacterStyle` and their `Color` associated type.
 * **breaking** Replaced `style::color::Rgb` with `embedded_graphics::pixelcolor::Rgb888`.
 * ANSI sequence support now requires the `ansi` feature which is enabled by default.

## Removed

 * **breaking** Removed benchmarks.
 * **breaking** Removed `prelude`.
 * **breaking** Removed `Rectangle` extensions.
 * **breaking** Removed deprecated `TextBoxStyleBuilder::{text_style, background_color, text_color, from_text_style, underlined, strikethrough}`. Use `TextBoxStyleBuilder::character_style` instead.
 * **breaking** (developer-facing) The following types and modules have been removed or hidden:
   * `rendering::ansi`, `rendering::cursor`, `rendering::character`, `rendering::decorated_space`, `rendering::line`, `rendering::line_iter`, `rendering::space_config`
 * Removed `TextBox::into_styled()`

## Fixed

 * `interactive_*` examples: fix accidental moving of bounding box.
 * `editor` example: cursor now doesn't stick to the text.
 * The `ansi` feature can now be used in `no_std` environments.

## Added

 * Added `TextBox::vertical_offset` and `TextBox::set_vertical_offset`.
 * Added `TextBoxStyle::paragraph_spacing` and `TextBoxStyleBuilder::paragraph_spacing`.
 * Added `From<&TextBoxStyle>` impl for `TextBoxStyleBuilder`
 * `TextBox::{with_alignment, with_vertical_alignment}`
 * `TextBoxStyle::{with_alignment, with_vertical_alignment}`
 * `TextBox::with_textbox_style()`
 * `TextBoxStyleBuilder` now implements `Default`
 * `StyledTextBox::draw()` now returns unconsumed text.
 * Added `interactive_columns` example to show flowing text into multiple columns.

0.4.1 (2021-04-25)
==================

## Changed:
 
 * Updated `ansi-parser` dependency to `0.8.0`.
 * ANSI sequence support now requires the `ansi` feature which is on by default.
 * Fields of the `style::color::Rgb` struct are now public.

## Fixed:

 * The `ansi` feature can now be used in `no_std` environments.

0.4.0 (2020-11-26)
==================

## Added:

 * Added support for strikethrough and underlined text.
 * `RendererFactory` trait that can be used to create a pixel iterator.
 * Handle tabs `\t` with configurable tab size.
 * Added `TabSize` struct and related style builder method `tab_size`.
 * Added partial support for [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code).
 * `Scrolling` vertical alignment
 * `TextBoxStyleBuilder` now implements `Copy` and `Clone`
 * `TextBox` and `StyledTextBox` now implements `Copy`, `Clone`, `Debug`, `Eq`, `PartialEq` and `Hash`

## Changed:

 * **breaking** Left aligned text now eats a single white space at the end of a wrapped line. This changes some height measurements and rendering output.
 * **breaking** `TextBoxStyle::measure_line` now returns whether the line is underlined.
 * **breaking** Renamed `StyledCharacterIterator` to `CharacterIterator`
 * **breaking** Increase the Minimum Supported Rust Version to `1.41.0`
 * **breaking** `rendering::line_iter::State` is no longer public
 * **breaking** `rendering::line::State` is no longer public
 * **breaking** Removed `StateFactory`
 * **breaking** Removed `FontExt::str_width`, `FontExt::max_str_width` and `FontExt::max_str_width_nocr`
 * **breaking** `TextBoxStyle` and `TextBoxStyleBuilder` no longer derives `Ord` and `PartialOrd`

0.3.0 (2020-10-02)
==================

## Added:

 * `TextBoxStyleBuilder::from_text_style`
 * Added `HeightMode` to select whether and how the `StyledTextBox` height should be aligned to the
   actual text height.
 * Added `Hidden`, `Visible` and `FullRowsOnly` overflow control modes to `Exact` and `ShrinkToText` height modes.
 * Added line spacing support via `TextBoxStyleBuilder::line_spacing`
 * Soft hyphen character support (`\u{AD}`), rendered as a normal `-`.

## Changed:

 * Deprecated `TextBoxStyleBuilder::text_style`. Use `TextBoxStyleBuilder::from_text_style` instead.
 * **breaking** Moved `StyledTextBox` to the root module.
 * Added `StyledTextBox::fit_height` and `StyledTextBox::fit_height_limited` to adjust height to text

## Fixed:

 * Fix `CenterAligned` and `BottomAligned` vertical alignments crashing the program when text is
   taller than the `TextBox`.

0.2.0 (2020-08-15)
==================

## Added:

 * Support for vertical text alignment.
 * Added alignment types to `prelude`.
 * Support for zero-width space character (`\u{200B}`).
 * Support for nonbreaking space character (`\u{A0}`).
 * Added optimized measurement function that do not expect carriage returns.
   * `FontExt::measure_line_nocr`
   * `FontExt::str_width_nocr`
   * `FontExt::max_str_width_nocr`
 * Support carriage return (`\r`) control characters.

## Fixed:

 * Fixed an issue where height measurement unexpectedly carried a space that is consumed during drawing.

0.1.0 (2020-07-31)
==================

## Added:

 * Added `TextBoxStyle::from_text_style`

## Changed:

 * **breaking:** Renamed `measure_text` to `measure_text_height`
 * **breaking:** Moved `measure_text` from `FontExt` to `TextBoxStyle`
 * **breaking:** Removed `FontExt` from `prelude`

## Fixed:

 * Fixed an issue where after a line break, the last line was not rendered if the exact height was available.
 * Fixed several text height measurement issues and inconsistencies.
 * Fixed an issue where text height was measured incorrectly when encountering words wider than line.
 * Fixed an issue where characters could be drawn outside of the bounding box when there is not enough space to render a single character.
 * Fixed a word wrapping issue where the first space may sometimes be rendered 0 width.
 * Crash with `Justified` alignment.

0.0.3 (2020-07-28)
==================

## Added:

 * `prelude` import
 * Render fonts with variable character width
 * `FontExt::measure_text` to measure height using a certain width. Implemented for all `Font` instances.

0.0.1 (2020-07-21)
==================
 * Initial release
