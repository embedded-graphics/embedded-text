//! Plugin trait.

use embedded_graphics::{
    draw_target::DrawTarget,
    prelude::PixelColor,
    primitives::Rectangle,
    text::renderer::{CharacterStyle, TextRenderer},
};

use crate::{parser::Token, rendering::cursor::Cursor, TextBoxProperties};

/// Plugin trait.
///
/// Plugins allow modifying and extending TextBox's internals.
///
/// *Important*:
/// This is an experimental, unstable feature. It can be, and probably will be modified without
/// any prior notice.
/// Using plugins require enabling the `plugin` crate feature.
pub trait Plugin<'a, C>: Clone
where
    C: PixelColor,
{
    /// Called when a new line is started.
    #[inline]
    fn new_line(&mut self) {}

    /// Generate the next text token.
    #[inline]
    fn next_token(
        &mut self,
        mut next_token: impl FnMut() -> Option<Token<'a, C>>,
    ) -> Option<Token<'a, C>> {
        next_token()
    }

    /// Modify the current token immediately before it is rendered.
    ///
    /// This function must return the same token type as the input, otherwise the returned token
    /// is ignored.
    #[inline]
    fn render_token(&mut self, token: Token<'a, C>) -> Option<Token<'a, C>> {
        Some(token)
    }

    /// Called after a piece of text is rendered.
    #[inline]
    fn post_render<T, D>(
        &mut self,
        _draw_target: &mut D,
        _character_style: &T,
        _text: &str,
        _bounds: Rectangle,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer<Color = C>,
        D: DrawTarget<Color = C>,
    {
        Ok(())
    }

    /// Called before TextBox rendering is started.
    #[inline]
    fn on_start_render<S: CharacterStyle>(
        &mut self,
        _cursor: &mut Cursor,
        _props: TextBoxProperties<'_, S>,
    ) {
    }
}

impl<'a, C> Plugin<'a, C> for super::NoPlugin<C> where C: PixelColor {}
