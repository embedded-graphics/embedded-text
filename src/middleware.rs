//! Middleware allow changing TextBox behaviour.

use crate::parser::Token;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ProcessingState {
    Measure,

    Render,
}

pub trait Middleware<'a>: Clone {
    /// Called when a new line is started.
    #[inline]
    fn new_line(&mut self) {}

    #[inline]
    fn next_token(
        &mut self,
        _state: ProcessingState,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        next_token.next()
    }
}

#[derive(Clone, Copy)]
pub struct NoMiddleware;
impl<'a> Middleware<'a> for NoMiddleware {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct MiddlewareWrapper<M> {
    pub middleware: M,
    state: ProcessingState,
}

impl<'a, M> MiddlewareWrapper<M>
where
    M: Middleware<'a>,
{
    pub fn new(middleware: M) -> Self {
        Self {
            middleware,
            state: ProcessingState::Measure,
        }
    }

    pub fn new_line(&mut self) {
        self.middleware.new_line();
    }

    pub fn set_state(&mut self, state: ProcessingState) {
        self.state = state;
    }

    pub fn next_token(
        &mut self,
        next_token: &mut impl Iterator<Item = Token<'a>>,
    ) -> Option<Token<'a>> {
        self.middleware.next_token(self.state, next_token)
    }
}
