use core::str::Chars;
use embedded_graphics::fonts::Font;

pub trait FontExt {
    fn max_fitting<'a>(iter: Chars<'a>, max_width: u32) -> (u32, bool);
}

impl<F> FontExt for F
where
    F: Font,
{
    fn max_fitting<'a>(iter: Chars<'a>, max_width: u32) -> (u32, bool) {
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
}
