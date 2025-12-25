#![no_std]

#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HyperCall {
    Print = 0,
    Exit = 1,
    // Future:
    // DrawFrame = 2,
    // Sleep = 3,
}

impl HyperCall {
    pub fn from_u64(val: u64) -> Option<Self> {
        match val {
            0 => Some(Self::Print),
            1 => Some(Self::Exit),
            _ => None,
        }
    }
}
