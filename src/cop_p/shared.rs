//! Shared COP-P types: sequence width, frame wrappers, errors.
//!
//! Sequence numbers follow **§6.1** modulo arithmetic for 8-bit or 16-bit widths.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqWidth {
    Mod256,
    Mod65536,
}

impl SeqWidth {
    pub(crate) fn modulus(self) -> u32 {
        match self {
            SeqWidth::Mod256 => 256,
            SeqWidth::Mod65536 => 65536,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seq(pub u32);

impl Seq {
    pub fn as_u8(self) -> u8 {
        (self.0 & 0xFF) as u8
    }
    pub fn as_u16(self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }
}

pub(crate) fn add_mod(x: u32, inc: u32, m: u32) -> u32 {
    (x + inc) % m
}

/// Return distance `b - a` modulo m in the range 0..m-1.
pub(crate) fn dist_mod(a: u32, b: u32, m: u32) -> u32 {
    (b + m - a) % m
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopError {
    WindowFull,
    InvalidPlcw(&'static str),
}

impl core::fmt::Display for CopError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CopError::WindowFull => write!(f, "transmit window full"),
            CopError::InvalidPlcw(msg) => write!(f, "invalid PLCW: {msg}"),
        }
    }
}

impl std::error::Error for CopError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeqFrame {
    pub ns: Seq,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpFrame {
    pub ve_s: u8, // modulo-8 counter
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopFrame {
    Expedited(ExpFrame),
    SequenceControlled(SeqFrame),
}
