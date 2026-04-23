//! Small interoperability traits for “wire-format” types.
//!
//! Goal: allow SPDU / Frame / other layers to interoperate via a minimal shared
//! encode/decode + hex dump surface without pulling in external dependencies.
// [MermaidChart: a2573561-f359-4881-ac2e-f23bbb90e75b]
/// Encode a value into its on-wire octet representation.
pub trait WireEncode {
    type Error;
    fn to_wire_bytes(&self) -> Result<Vec<u8>, Self::Error>;
}

/// Decode a value from its on-wire octet representation.
pub trait WireDecode: Sized {
    type Error;
    fn from_wire_bytes(data: &[u8]) -> Result<Self, Self::Error>;
}

/// Validate a value against wire-format constraints (range checks, etc.).
pub trait Validate {
    type Error;
    fn validate(&self) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexError {
    Empty,
    // Giving an empty string with no actual hex digits is invalid.
    InvalidChar(char),
    // Giving a string with an invalid character is invalid.
    // This is because each hex digit is a character, so an invalid character is not a valid hex string.
    OddLength,
    // Giving a string with an odd number of hex digits is invalid.
    // This is because each hex digit is 4 bits, so an odd number of hex digits is not a valid hex string.
}

// This is a custom-helper that implements the display of the error message of the HexError enum.
impl core::fmt::Display for HexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HexError::Empty => write!(f, "empty hex string"),
            HexError::InvalidChar(c) => write!(f, "invalid hex character: {c:?}"),
            HexError::OddLength => write!(f, "hex string has odd length"),
        }
    }
}

// This is a custom-helper that implements the Error trait for the HexError enum.
impl std::error::Error for HexError {}

/// Convert bytes to lowercase hex (no separators).
// This is a custom-helper that converts bytes to a hex string.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0F) as usize] as char);
    }
    out
}

// This is a custom-helper that converts a character to a hex value.
fn hex_val(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some((c as u8) - b'0'),
        'a'..='f' => Some((c as u8) - b'a' + 10),
        'A'..='F' => Some((c as u8) - b'A' + 10),
        _ => None,
    }
}

/// Parse hex string into bytes.
///
/// Accepts:
/// - optional `0x` prefixes,
/// - whitespace and `_` separators.
pub fn hex_to_bytes(mut s: &str) -> Result<Vec<u8>, HexError> {
    s = s.trim();
    if s.is_empty() {
        return Err(HexError::Empty);
    }

    // First pass: keep only hex digits (and validate).
    let mut digits = String::with_capacity(s.len());
    let mut any = false;
    let mut i = 0usize;
    while i < s.len() {
        let rest = &s[i..];
        if rest.starts_with("0x") || rest.starts_with("0X") {
            i += 2;
            continue;
        }
        let c = rest.chars().next().unwrap();
        let len = c.len_utf8();
        i += len;

        if c.is_whitespace() || c == '_' {
            continue;
        }
        if hex_val(c).is_none() {
            return Err(HexError::InvalidChar(c));
        }
        any = true;
        digits.push(c);
    }

    if !any {
        return Err(HexError::Empty);
    }
    if (digits.len() % 2) != 0 {
        return Err(HexError::OddLength);
    }

    let mut out = Vec::with_capacity(digits.len() / 2);
    let mut iter = digits.chars();
    while let (Some(h), Some(l)) = (iter.next(), iter.next()) {
        let hi = hex_val(h).ok_or(HexError::InvalidChar(h))?;
        let lo = hex_val(l).ok_or(HexError::InvalidChar(l))?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

/// Hex dump interoperability for wire-format values.
pub trait HexDump: WireEncode {
    fn to_hex_dump(&self) -> Result<String, Self::Error> {
        Ok(bytes_to_hex(&self.to_wire_bytes()?))
    }
}

impl<T> HexDump for T where T: WireEncode {}
