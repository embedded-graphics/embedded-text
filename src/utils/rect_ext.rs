use embedded_graphics::{prelude::*, primitives::Rectangle};

pub trait RectExt {
    fn size(self) -> Size;
}

impl RectExt for Rectangle {
    fn size(self) -> Size {
        // TODO: remove if fixed in embedded-graphics
        let width = (self.bottom_right.x - self.top_left.x) as u32 + 1;
        let height = (self.bottom_right.y - self.top_left.y) as u32 + 1;

        Size::new(width, height)
    }
}
