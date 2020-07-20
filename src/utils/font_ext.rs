use core::str::Chars;
use embedded_graphics::{fonts::Font, geometry::Point};

pub trait FontExt {
    fn max_fitting(iter: Chars<'_>, max_width: u32) -> (u32, bool);

    fn character_point(c: char, p: Point) -> bool;
}

impl<F> FontExt for F
where
    F: Font,
{
    fn max_fitting(iter: Chars<'_>, max_width: u32) -> (u32, bool) {
        let mut total_width = 0;
        let mut fits = true;
        for c in iter {
            let width = F::char_width(c);
            let new_width = total_width + width;
            if new_width < max_width {
                total_width = new_width;
            } else {
                fits = false;
                break;
            }
        }

        (total_width, fits)
    }

    fn character_point(c: char, p: Point) -> bool {
        Self::character_pixel(c, p.x as u32, p.y as u32)
    }
}
