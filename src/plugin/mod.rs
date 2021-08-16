//! Plugins allow changing TextBox behaviour.
//!
//! Note: Custom plugins are experimental. Ff you wish to implement custom plugins,
//! you need to activate the `plugin` feature.

use core::{
    cell::RefCell,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    prelude::PixelColor,
    primitives::Rectangle,
    text::renderer::{CharacterStyle, TextRenderer},
};

use crate::{
    parser::{Parser, Token},
    rendering::cursor::Cursor,
    TextBoxProperties,
};

#[cfg(feature = "plugin")]
pub mod private;
#[cfg(feature = "plugin")]
pub use private::Plugin;

#[cfg(not(feature = "plugin"))]
mod private;
#[cfg(not(feature = "plugin"))]
use private::Plugin;

#[cfg(feature = "ansi")]
pub mod ansi;
pub mod tail;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ProcessingState {
    Measure,
    Render,
}

/// Placeholder type when no plugin is used.
#[derive(Clone, Copy, Default)]
pub struct NoPlugin<C>(PhantomData<C>)
where
    C: PixelColor;

impl<C> NoPlugin<C>
where
    C: PixelColor,
{
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

/// Plugin marker trait.
///
/// This trait is an implementation detail. Most likely you don't need to implement this.
#[cfg_attr(
    feature = "plugin",
    doc = "If you wish to implement a plugin, see [Plugin]."
)]
// TODO: remove this trait once Plugin is stabilized, then move Plugin here
pub trait PluginMarker<'a, C: PixelColor>: Plugin<'a, C> {}

impl<'a, C, T> PluginMarker<'a, C> for T
where
    T: Plugin<'a, C>,
    C: PixelColor,
{
}

#[derive(Clone, Debug)]
pub(crate) struct PluginInner<'a, M, C>
where
    C: PixelColor,
{
    pub(crate) lookahead: M,
    pub(crate) plugin: M,
    state: ProcessingState,
    peeked_token: (usize, Option<Token<'a, C>>),
}

#[derive(Clone, Debug)]
pub(crate) struct PluginWrapper<'a, M, C>
where
    C: PixelColor,
{
    pub inner: RefCell<PluginInner<'a, M, C>>,
}

impl<'a, M, C> Hash for PluginWrapper<'a, M, C>
where
    C: PixelColor,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.borrow().state.hash(state)
    }
}

impl<'a, M, C> PluginWrapper<'a, M, C>
where
    C: PixelColor,
    M: private::Plugin<'a, C>,
{
    pub fn new(plugin: M) -> Self {
        Self {
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

        this.lookahead.new_line();
    }

    pub fn set_state(&self, state: ProcessingState) {
        self.inner.borrow_mut().state = state;
    }

    #[inline]
    pub fn render_token(&self, token: Token<'a, C>) -> Option<Token<'a, C>> {
        let mut this = self.inner.borrow_mut();
        match this.state {
            ProcessingState::Measure => Some(token),
            ProcessingState::Render => this.lookahead.render_token(token),
        }
    }

    pub fn peek_token(&self, source: &mut Parser<'a, C>) -> Option<Token<'a, C>> {
        let mut this = self.inner.borrow_mut();

        if this.peeked_token.1.is_none() {
            let mut cloned = source.clone();

            this.peeked_token.1 = this.lookahead.next_token(|| cloned.next());

            // It's possible that plugins modify the returned token so this isn't reliable
            this.peeked_token.0 = source.as_str().len() - cloned.as_str().len();
        }
        this.peeked_token.1.clone()
    }

    pub fn consume_peeked_token(&self, source: &mut Parser<'a, C>) {
        let mut this = self.inner.borrow_mut();

        if this.peeked_token.1.is_some() {
            unsafe {
                source.consume(this.peeked_token.0);
            }
            this.peeked_token.0 = 0;
            this.peeked_token.1 = None;

            this.plugin = this.lookahead.clone();
        }
    }

    pub fn consume_partial(&self, len: usize, source: &mut Parser<'a, C>) {
        let mut this = self.inner.borrow_mut();

        // Only string-like tokens can be partially consumed.
        debug_assert!(matches!(
            this.peeked_token.1,
            Some(Token::Whitespace(_, _)) | Some(Token::Word(_))
        ));

        let skip_chars = |str: &'a str, n| {
            let mut chars = str.chars();
            for _ in 0..n {
                chars.next();
            }
            chars.as_str()
        };

        let token = match this.peeked_token.1.take().unwrap() {
            Token::Whitespace(count, seq) => {
                Token::Whitespace(count - len as u32, skip_chars(seq, len))
            }
            Token::Word(w) => Token::Word(skip_chars(w, len)),
            _ => unreachable!(),
        };

        // In case plugin only returned a partial token, we are consuming parts but
        // `this.peeked_token.0` contains the length of the whole token.
        let consumed = len.min(this.peeked_token.0);

        unsafe {
            source.consume(consumed);
        }

        this.peeked_token.0 -= consumed;
        this.peeked_token.1.replace(token);
    }

    pub fn on_start_render<S: CharacterStyle + TextRenderer>(
        &self,
        cursor: &mut Cursor,
        props: TextBoxProperties<'_, S>,
    ) {
        let mut this = self.inner.borrow_mut();
        this.peeked_token = (0, None);

        this.lookahead.on_start_render(cursor, &props);
    }

    pub fn on_rendering_finished(&self) {
        let mut this = self.inner.borrow_mut();

        this.lookahead.on_rendering_finished();
    }

    pub fn post_render<T, D>(
        &self,
        draw_target: &mut D,
        character_style: &T,
        text: Option<&str>,
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
