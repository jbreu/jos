#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub rows: usize,
    pub cols: usize,
}

pub fn get() -> Option<Size> {
    None
}
