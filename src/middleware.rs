//! Middleware allow changing TextBox behaviour.

use core::{
    cell::RefCell,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    prelude::{PixelColor, Point},
    primitives::Rectangle,
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

pub trait Middleware<'a, C>: Clone
where
    C: PixelColor,
{
    /// Called when a new line is started.
    #[inline]
    fn new_line(&mut self) {}

    #[inline]
    fn next_token(
        &mut self,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        next_token.next()
    }

    #[inline]
    fn render_token(&mut self, token: Token<'a>) -> Option<Token<'a>> {
        Some(token)
    }

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

    #[inline]
    fn post_line_start<T, D>(
        &mut self,
        _draw_target: &mut D,
        _character_style: &T,
        _pos: Point,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer<Color = C>,
        D: DrawTarget<Color = C>,
    {
        Ok(())
    }

    #[inline]
    fn on_start_render<S: TextRenderer>(
        &mut self,
        _text_box: &TextBox<'a, S, Self>,
        _cursor: &mut Cursor,
    ) {
    }
}

#[derive(Clone, Copy, Default)]
pub struct NoMiddleware<C>(PhantomData<C>);

impl<C> NoMiddleware<C> {
    #[inline]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, C> Middleware<'a, C> for NoMiddleware<C> where C: PixelColor {}

#[derive(Clone, Debug)]
pub(crate) struct MiddlewareInner<'a, M> {
    lookahead: M,
    middleware: M,
    state: ProcessingState,
    peeked_token: (usize, Option<Token<'a>>),
}

#[derive(Clone, Debug)]
pub(crate) struct MiddlewareWrapper<'a, M, C> {
    inner: RefCell<MiddlewareInner<'a, M>>,
    _marker: PhantomData<C>,
}

impl<'a, M, C> Hash for MiddlewareWrapper<'a, M, C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.borrow().state.hash(state)
    }
}

impl<'a, M, C> MiddlewareWrapper<'a, M, C>
where
    C: PixelColor,
    M: Middleware<'a, C>,
{
    pub fn new(middleware: M) -> Self {
        Self {
            _marker: PhantomData,
            inner: RefCell::new(MiddlewareInner {
                lookahead: middleware.clone(),
                middleware,
                state: ProcessingState::Measure,
                peeked_token: (0, None),
            }),
        }
    }

    pub fn new_line(&self) {
        let mut this = self.inner.borrow_mut();
        this.peeked_token.0 = 0;
        this.peeked_token.1 = None;
        this.middleware.new_line();

        this.lookahead = this.middleware.clone();
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
            this.peeked_token.1 = this.lookahead.next_token(&mut cloned);
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

        this.middleware = this.lookahead.clone();
    }

    pub fn replace_peeked_token(&self, len: usize, token: Token<'a>) {
        let mut this = self.inner.borrow_mut();

        this.peeked_token.0 = len;
        this.peeked_token.1.replace(token);

        this.lookahead = this.middleware.clone();
    }

    pub fn start_render<S: TextRenderer>(&self, text_box: &TextBox<'a, S, M>, cursor: &mut Cursor) {
        let mut this = self.inner.borrow_mut();
        this.peeked_token = (0, None);

        this.middleware.on_start_render(text_box, cursor);
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

    pub fn post_line_start<T, D>(
        &self,
        draw_target: &mut D,
        character_style: &T,
        pos: Point,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer<Color = C>,
        D: DrawTarget<Color = C>,
    {
        self.inner
            .borrow_mut()
            .lookahead
            .post_line_start(draw_target, character_style, pos)
    }
}
