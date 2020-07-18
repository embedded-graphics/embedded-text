use crate::TextAlignment;

#[derive(Copy, Clone, Debug)]
pub struct LeftAligned;
impl TextAlignment for LeftAligned {}

#[derive(Copy, Clone, Debug)]
pub struct RightAligned;
impl TextAlignment for RightAligned {}

#[derive(Copy, Clone, Debug)]
pub struct CenterAligned;
impl TextAlignment for CenterAligned {}

#[derive(Copy, Clone, Debug)]
pub struct Justified;
impl TextAlignment for Justified {}
