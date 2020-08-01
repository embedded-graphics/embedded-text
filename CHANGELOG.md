Unreleased
==========

## Added:

 * Support carriage return (`\r`) control characters.

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
