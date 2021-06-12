//! Middleware allow changing TextBox behaviour.

use core::{
    cell::{Cell, RefCell},
    hash::{Hash, Hasher},
};
use embedded_graphics::{
    draw_target::DrawTarget, primitives::Rectangle, text::renderer::TextRenderer,
};

use crate::parser::Token;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ProcessingState {
    Measure,
    Render,
}

pub trait Middleware<'a>: Clone {
    /// Called when a new line is started.
    #[inline]
    fn new_line(&mut self) {}

    #[inline]
    fn next_token_to_measure(
        &mut self,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        next_token.next()
    }

    #[inline]
    fn next_token_to_render(
        &mut self,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        next_token.next()
    }

    #[inline]
    fn post_render_text<T, D>(
        &mut self,
        _draw_target: &mut D,
        _character_style: &T,
        _text: &str,
        _bounds: Rectangle,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer,
        D: DrawTarget<Color = T::Color>,
    {
        Ok(())
    }

    #[inline]
    fn post_render_whitespace<T, D>(
        &mut self,
        _draw_target: &mut D,
        _character_style: &T,
        _width: u32,
        _space_count: u32,
        _bounds: Rectangle,
    ) -> Result<(), D::Error>
    where
        T: TextRenderer,
        D: DrawTarget<Color = T::Color>,
    {
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct NoMiddleware;
impl<'a> Middleware<'a> for NoMiddleware {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MiddlewareWrapper<M> {
    pub middleware: RefCell<M>,
    state: Cell<ProcessingState>,
}

impl<M> Hash for MiddlewareWrapper<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.get().hash(state)
    }
}

impl<'a, M> MiddlewareWrapper<M>
where
    M: Middleware<'a>,
{
    pub fn new(middleware: M) -> Self {
        Self {
            middleware: RefCell::new(middleware),
            state: Cell::new(ProcessingState::Measure),
        }
    }

    pub fn new_line(&self) {
        self.middleware.borrow_mut().new_line();
    }

    pub fn set_state(&self, state: ProcessingState) {
        self.state.set(state);
    }

    pub fn next_token(
        &self,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        match self.state.get() {
            ProcessingState::Measure => self
                .middleware
                .borrow_mut()
                .next_token_to_measure(next_token),
            ProcessingState::Render => self
                .middleware
                .borrow_mut()
                .next_token_to_render(next_token),
        }
    }
}
