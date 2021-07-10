//! Plugin allow changing TextBox behaviour.

use core::{
    cell::RefCell,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use embedded_graphics::{
    draw_target::DrawTarget, prelude::PixelColor, primitives::Rectangle,
    text::renderer::TextRenderer,
};

use crate::{
    parser::{Parser, Token},
    rendering::cursor::Cursor,
    TextBox,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ProcessingState {
    Measure,
    Render,
}

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
        mut next_token: impl FnMut() -> Option<Token<'a>>,
    ) -> Option<Token<'a>> {
        next_token()
    }

    /// Modify the current token immediately before it is rendered.
    ///
    /// This function must return the same token type as the input, otherwise the returned token
    /// is ignored.
    #[inline]
    fn render_token(&mut self, token: Token<'a>) -> Option<Token<'a>> {
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
    fn on_start_render<S: TextRenderer>(
        &mut self,
        _text_box: &TextBox<'a, S, Self>,
        _cursor: &mut Cursor,
    ) {
    }
}

/// Placeholder type when no plugin is used.
#[derive(Clone, Copy, Default)]
pub struct NoPlugin<C>(PhantomData<C>);

impl<C> NoPlugin<C> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, C> Plugin<'a, C> for NoPlugin<C> where C: PixelColor {}

#[derive(Clone, Debug)]
pub(crate) struct PluginInner<'a, M> {
    lookahead: M,
    plugin: M,
    state: ProcessingState,
    peeked_token: (usize, Option<Token<'a>>),
}

#[derive(Clone, Debug)]
pub(crate) struct PluginWrapper<'a, M, C> {
    inner: RefCell<PluginInner<'a, M>>,
    _marker: PhantomData<C>,
}

impl<'a, M, C> Hash for PluginWrapper<'a, M, C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.borrow().state.hash(state)
    }
}

impl<'a, M, C> PluginWrapper<'a, M, C>
where
    C: PixelColor,
    M: Plugin<'a, C>,
{
    pub fn new(plugin: M) -> Self {
        Self {
            _marker: PhantomData,
            inner: RefCell::new(PluginInner {
                lookahead: plugin.clone(),
                plugin,
                state: ProcessingState::Measure,
                peeked_token: (0, None),
            }),
        }
    }

    pub fn new_line(&self) {
        let mut this = self.inner.borrow_mut();
        this.peeked_token.0 = 0;
        this.peeked_token.1 = None;
        this.plugin.new_line();

        this.lookahead = this.plugin.clone();
    }

    pub fn set_state(&self, state: ProcessingState) {
        self.inner.borrow_mut().state = state;
    }

    #[inline]
    pub fn render_token(&self, token: Token<'a>) -> Option<Token<'a>> {
        let mut this = self.inner.borrow_mut();
        match this.state {
            ProcessingState::Measure => Some(token),
            ProcessingState::Render => this.lookahead.render_token(token),
        }
    }

    pub fn peek_token(&self, source: &mut Parser<'a>) -> Option<Token<'a>> {
        let mut this = self.inner.borrow_mut();

        if this.peeked_token.1.is_none() {
            let mut cloned = source.clone();
            this.peeked_token.1 = this.lookahead.next_token(|| cloned.next());
            this.peeked_token.0 = source.as_str().len() - cloned.as_str().len();
        }
        this.peeked_token.1.clone()
    }

    pub fn consume_peeked_token(&self, source: &mut Parser<'a>) {
        let mut this = self.inner.borrow_mut();

        unsafe {
            source.consume(this.peeked_token.0);
        }
        this.peeked_token.0 = 0;
        this.peeked_token.1 = None;

        this.plugin = this.lookahead.clone();
    }

    pub fn replace_peeked_token(&self, len: usize, token: Token<'a>) {
        let mut this = self.inner.borrow_mut();

        this.peeked_token.0 = len;
        this.peeked_token.1.replace(token);

        this.lookahead = this.plugin.clone();
    }

    pub fn on_start_render<S: TextRenderer>(
        &self,
        text_box: &TextBox<'a, S, M>,
        cursor: &mut Cursor,
    ) {
        let mut this = self.inner.borrow_mut();
        this.peeked_token = (0, None);

        this.plugin.on_start_render(text_box, cursor);
    }

    pub fn post_render<T, D>(
        &self,
        draw_target: &mut D,
        character_style: &T,
        text: &str,
        bounds: Rectangle,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer<Color = C>,
        D: DrawTarget<Color = C>,
    {
        self.inner
            .borrow_mut()
            .lookahead
            .post_render(draw_target, character_style, text, bounds)
    }
}
