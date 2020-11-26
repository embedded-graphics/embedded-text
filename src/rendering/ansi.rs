//! ANSI escape sequence related types and functions.
use crate::style::color::Rgb;

/// List of supported SGR (Select Graphics Rendition) sequences
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sgr {
    /// Reset all styling options
    Reset,

    /// Draw a line under the text
    Underline,

    /// Cross out the text
    CrossedOut,

    /// Disable drawing underline
    UnderlineOff,

    /// Disable crossing out
    NotCrossedOut,

    /// Change the text color
    ChangeTextColor(Rgb),

    /// Reset the text color to transparent
    DefaultTextColor,

    /// Change the background color
    ChangeBackgroundColor(Rgb),

    /// Reset the background color to transparent
    DefaultBackgroundColor,
}

fn try_parse_8b_color(v: &[u8]) -> Option<Rgb> {
    let color = *v.get(0)?;
    match color {
        //   0-  7:  standard colors (as in ESC [ 30–37 m)
        //   8- 15:  high intensity colors (as in ESC [ 90–97 m)
        0..=15 => Some(standard_to_rgb(color)),

        //  16-231:  6 × 6 × 6 cube (216 colors): 16 + 36 × r + 6 × g + b (0 ≤ r, g, b ≤ 5)
        16..=231 => {
            fn extract_ch(source: u8) -> (u8, u8) {
                let ch = (source % 6) * 51; // 5 * 51 = 255
                let remainder = source / 6;

                (ch, remainder)
            }

            let source_rgb = color - 16;
            let (b, source_rg) = extract_ch(source_rgb);
            let (g, source_r) = extract_ch(source_rg);
            let (r, _) = extract_ch(source_r);

            Some(Rgb::new(r, g, b))
        }

        // 232-255:  grayscale from black to white in 24 steps
        232..=255 => {
            let level = color - 232;
            let g = if level == 23 { 255 } else { level * 11 };
            Some(Rgb::new(g, g, g))
        }
    }
}

fn try_parse_rgb(v: &[u8]) -> Option<Rgb> {
    let r = *v.get(0)?;
    let g = *v.get(1)?;
    let b = *v.get(2)?;

    Some(Rgb::new(r, g, b))
}

fn standard_to_rgb(idx: u8) -> Rgb {
    // These colors are used in PowerShell 6 in Windows 10
    match idx {
        0 => Rgb::new(12, 12, 12),
        1 => Rgb::new(197, 15, 31),
        2 => Rgb::new(19, 161, 14),
        3 => Rgb::new(193, 156, 0),
        4 => Rgb::new(0, 55, 218),
        5 => Rgb::new(136, 23, 152),
        6 => Rgb::new(58, 150, 221),
        7 => Rgb::new(204, 204, 204),

        8 => Rgb::new(118, 118, 118),
        9 => Rgb::new(231, 72, 86),
        10 => Rgb::new(22, 198, 12),
        11 => Rgb::new(249, 241, 165),
        12 => Rgb::new(59, 120, 255),
        13 => Rgb::new(180, 0, 158),
        14 => Rgb::new(97, 214, 214),
        _ => Rgb::new(242, 242, 242),
    }
}

fn try_parse_color(v: &[u8]) -> Option<Rgb> {
    let color_type = *v.get(0)?;

    match color_type {
        2 => try_parse_rgb(&v[1..]),
        5 => try_parse_8b_color(&v[1..]),

        _ => None,
    }
}

/// Parse a set of SGR parameter numbers into a more convenient type
#[inline]
pub fn try_parse_sgr(v: &[u8]) -> Option<Sgr> {
    let code = *v.get(0)?;
    match code {
        0 => Some(Sgr::Reset),
        4 => Some(Sgr::Underline),
        9 => Some(Sgr::CrossedOut),
        24 => Some(Sgr::UnderlineOff),
        29 => Some(Sgr::NotCrossedOut),
        39 => Some(Sgr::DefaultTextColor),
        49 => Some(Sgr::DefaultBackgroundColor),
        30..=37 => Some(Sgr::ChangeTextColor(standard_to_rgb(code - 30))),
        38 => {
            let color = try_parse_color(&v[1..])?;
            Some(Sgr::ChangeTextColor(color))
        }
        90..=97 => Some(Sgr::ChangeTextColor(standard_to_rgb(code - 82))),
        40..=47 => Some(Sgr::ChangeBackgroundColor(standard_to_rgb(code - 40))),
        48 => {
            let color = try_parse_color(&v[1..])?;
            Some(Sgr::ChangeBackgroundColor(color))
        }
        100..=107 => Some(Sgr::ChangeBackgroundColor(standard_to_rgb(code - 92))),
        _ => None,
    }
}
