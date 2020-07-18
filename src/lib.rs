//! This crate implements rendering text in a given area for embedded-graphics
#![cfg_attr(not(test), no_std)]

/// Parse text into smaller units
pub mod parser;

/// Helpers to render text
pub mod rendering;
