//! Sequence-number types and modulo arithmetic for COP-P.
//!
//! COP-P defines sequence variables (e.g. `V(S)`, `V(R)`, `NN(R)`) as counters that wrap around
//! in a fixed-width number space.
//!
//! In the original COP-P description (CCSDS 235.1 §6.1), the sending/receiving procedures use
//! **single-octet modulo-256** counters and define special subtraction/comparison rules.
//!
//! This module captures that behavior in reusable helpers so later layers (FOP-P / FARM-P) can
//! do window arithmetic and gap/duplicate detection correctly even across wrap-around.

/// Which modulo number space the COP-P sequence variables live in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqWidth {
    /// 8-bit (single-octet) sequence numbers: modulo 256.
    Mod256,
    /// 16-bit sequence numbers: modulo 65536.
    ///
    /// Not used in the COP-P PDF excerpt you provided (which focuses on single-octet vars),
    /// but the competition requirements mention supporting both widths.
    Mod65536,
}

impl SeqWidth {
    pub fn modulus(self) -> u32 {
        match self {
            SeqWidth::Mod256 => 256,
            SeqWidth::Mod65536 => 65536,
        }
    }

    /// Half the modulus (used by the spec's wrap-safe comparison rule).
    pub fn half(self) -> u32 {
        self.modulus() / 2
    }
}

/// A COP-P sequence value.
///
/// We store it as a `u32` so we can do arithmetic without overflowing, and interpret it modulo
/// the selected [`SeqWidth`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Seq(pub u32);

impl Seq {
    /// Interpret this sequence as an 8-bit value (low octet).
    pub fn as_u8(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    /// Interpret this sequence as a 16-bit value (low word).
    pub fn as_u16(self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }
}

/// Add `inc` to `x` modulo `m`.
pub fn add_mod(x: u32, inc: u32, m: u32) -> u32 {
    (x + inc) % m
}

/// COP-P subtraction rule: return \(A - B\) in the spec's sense.
///
/// In §6.1:
/// - “The difference, A–B, is the number of times B needs to be incremented to reach A.”
///
/// This is exactly the **forward distance** from `b` to `a` around a ring of size `m`.
pub fn diff(a: Seq, b: Seq, width: SeqWidth) -> u32 {
    let m = width.modulus();
    (a.0 + m - (b.0 % m)) % m
}

/// COP-P comparison rule from §6.1 for single-octet vars:
///
/// “B < A is true if (A–B) is between 1 and 127.”
pub fn less_than(b: Seq, a: Seq, width: SeqWidth) -> bool {
    let d = diff(a, b, width);
    d >= 1 && d <= (width.half() - 1)
}

/// COP-P comparison rule from §6.1 for single-octet vars:
///
/// “B > A is true if (A–B) is between 128 and 255.”
pub fn greater_than(b: Seq, a: Seq, width: SeqWidth) -> bool {
    let d = diff(a, b, width);
    d >= width.half() && d <= (width.modulus() - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_matches_spec_examples_mod256() {
        let w = SeqWidth::Mod256;
        // B needs 11 increments to reach A: 250 -> 5 wraps
        assert_eq!(diff(Seq(5), Seq(250), w), 11);
        // identical => 0
        assert_eq!(diff(Seq(10), Seq(10), w), 0);
        // simple forward
        assert_eq!(diff(Seq(42), Seq(40), w), 2);
    }

    #[test]
    fn comparison_rule_mod256() {
        let w = SeqWidth::Mod256;

        // If A-B in 1..=127 then B < A
        let a = Seq(10);
        let b = Seq(9);
        assert!(less_than(b, a, w));
        assert!(!greater_than(b, a, w));

        // If A-B in 128..=255 then B > A (wrap-safe "ahead/behind" rule)
        // Here: A=0, B=200 => A-B = 56? Wait: diff(A,B)= (0+256-200)=56 -> B < A (because B is "behind" A)
        // So choose B=1, A=200 => A-B = 199 -> B > A.
        let a = Seq(200);
        let b = Seq(1);
        assert!(greater_than(b, a, w));
        assert!(!less_than(b, a, w));

        // equality
        let a = Seq(77);
        let b = Seq(77);
        assert!(!less_than(b, a, w));
        assert!(!greater_than(b, a, w));
    }
}

