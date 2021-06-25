//! Middleware allow changing TextBox behaviour.

use core::{
    cell::{Cell, RefCell},
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    prelude::{PixelColor, Point},
    primitives::Rectangle,
    text::renderer::TextRenderer,
};

use crate::parser::Token;

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
}

#[derive(Clone, Copy)]
pub struct NoMiddleware<C>(PhantomData<C>);

impl<C> NoMiddleware<C> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, C> Middleware<'a, C> for NoMiddleware<C> where C: PixelColor {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MiddlewareWrapper<M, C> {
    pub middleware: RefCell<M>,
    state: Cell<ProcessingState>,
    _marker: PhantomData<C>,
}

impl<M, C> Hash for MiddlewareWrapper<M, C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.get().hash(state)
    }
}

impl<'a, M, C> MiddlewareWrapper<M, C>
where
    C: PixelColor,
    M: Middleware<'a, C>,
{
    pub fn new(middleware: M) -> Self {
        Self {
            _marker: PhantomData,
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
