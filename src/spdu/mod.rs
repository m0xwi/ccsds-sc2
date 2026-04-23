//! Supervisory Protocol Data Units (**SPDU**s) per **CCSDS 235.1-W-0.4 §3**.
//!
//! # Fixed-length SPDUs (§3.2)
//!
//! - **Type F1** — 16-bit **PLCW** ([`PLCW16Bit`], [`FixedLengthSPDU::F1`]); see Figure 3-1.
//! - **Type F2** — 32-bit **PLCW** ([`PLCW32Bit`], [`FixedLengthSPDU::F2`]); see Figure 3-2.
//!
//! # Variable-length SPDUs (§3.3)
//!
//! Identified by **SPDU Type** in the header; Types **1–5** map to [`VariableLengthSPDU`].
//! Type 1 directives/reports (e.g. SET V(R)) are in **Annex B**; Types 2–5 have annexes C/E/D.
//!
//! ## Workshop interoperability artifact (INTOP-1.3)
//!
//! **INTOP-1.3** — Variable-length SPDU **Type 1** directive **SET V(R)** with **`SEQ_CTRL_FSN = 42`**.
//!
//! On-wire bytes (3 octets), big-endian inside the Type 1 body:
//! - **Octet 0** (variable-length header): SPDU type `000`, body length `2` → `0x02`.
//! - **Octets 1–2** (one 16-bit directive word): SET V(R) → `0x60 0x2A`.
//!
//! **Canonical hex (no spaces):** `02602a` — same as **FR-9.5 workshop artifact #3** in the
//! reference implementation requirements.
//!
//! # Wire format
//!
//! [`SPDU::to_bytes`] / [`SPDU::from_bytes`] use **big-endian** octet order for multi-byte fields.
//!
//! # See also
//!
//! - COP-P consumes and generates PLCWs — [`crate::cop_p`].

// [MermaidChart: f6bf1054-83bf-45b5-acda-145246876000]
// This is the top-level SPDU API and defines the wire-format rules.
// Wire-format rules are the exact bytes that define the byte layout of the SPDU on the wire
// The wire-format rules include:
/// 1. What kind of SPDU it is
/// 2. The endianness
/// 3. The validation constraints
mod bits;
mod plcw;
mod type1;
mod type2;
mod type3;
mod type4;
mod type5;

pub use plcw::*;
pub use type1::*;
pub use type2::*;
pub use type3::*;
pub use type4::*;
pub use type5::*;

use crate::wire::{WireDecode, WireEncode};

/// Top-level SPDU: fixed (Format ID = 1) or variable-length (Format ID = 0).
///
/// See **§3.1** and Tables 3-1 / 3-2.
#[derive(Debug, Clone, PartialEq)]
pub enum SPDU {
    FixedLengthSPDU(FixedLengthSPDU),       // Format ID = 1
    VariableLengthSPDU(VariableLengthSPDU), // Format ID = 0
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixedLengthSPDU {
    F1(PLCW16Bit), // SPDU Type ID = 0 -> 16-bit PLCW
    F2(PLCW32Bit), // SPDU Type ID = 1 -> 32-bit PLCW
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableLengthSPDU {
    Type1(DirectivesOrReportsUHF), // 000
    Type2(TimeDistributionPDU),    // 001
    Type3(StatusReports),          // 010
    Type4(FirstGenLunar),          // 011
    Type5(SecondGenLunar),         // 100
    Reserved(u8, Vec<u8>),         // type_id, raw data
}

impl SPDU {
    /// Convenience constructor: wrap a Type F1 PLCW as a fixed-length SPDU.
    pub fn f1(plcw: PLCW16Bit) -> Self {
        SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(plcw))
    }

    /// Convenience constructor: wrap a Type F2 PLCW as a fixed-length SPDU.
    pub fn f2(plcw: PLCW32Bit) -> Self {
        SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(plcw))
    }

    /// Convenience constructor: wrap Type 1 directives as a variable-length SPDU.
    pub fn type1(body: DirectivesOrReportsUHF) -> Self {
        SPDU::VariableLengthSPDU(VariableLengthSPDU::Type1(body))
    }

    /// Convenience constructor: wrap Type 2 (Time Distribution) as a variable-length SPDU.
    pub fn type2(body: TimeDistributionPDU) -> Self {
        SPDU::VariableLengthSPDU(VariableLengthSPDU::Type2(body))
    }

    /// Convenience constructor: wrap Type 3 (Status Reports) as a variable-length SPDU.
    pub fn type3(body: StatusReports) -> Self {
        SPDU::VariableLengthSPDU(VariableLengthSPDU::Type3(body))
    }

    /// Convenience constructor: wrap Type 4 (First Gen Lunar) as a variable-length SPDU.
    pub fn type4(body: FirstGenLunar) -> Self {
        SPDU::VariableLengthSPDU(VariableLengthSPDU::Type4(body))
    }

    /// Convenience constructor: wrap Type 5 (Second Gen Lunar) as a variable-length SPDU.
    pub fn type5(body: SecondGenLunar) -> Self {
        SPDU::VariableLengthSPDU(VariableLengthSPDU::Type5(body))
    }

    /// Variable-length SPDU with an **unknown / reserved** type id (3-bit) and raw body (≤15 octets).
    pub fn variable_reserved(type_id: u8, raw: Vec<u8>) -> Self {
        SPDU::VariableLengthSPDU(VariableLengthSPDU::Reserved(type_id & 0x07, raw))
    }
}

impl From<PLCW16Bit> for SPDU {
    fn from(value: PLCW16Bit) -> Self {
        SPDU::f1(value)
    }
}

impl From<PLCW32Bit> for SPDU {
    fn from(value: PLCW32Bit) -> Self {
        SPDU::f2(value)
    }
}

impl From<DirectivesOrReportsUHF> for SPDU {
    fn from(value: DirectivesOrReportsUHF) -> Self {
        SPDU::type1(value)
    }
}

impl From<TimeDistributionPDU> for SPDU {
    fn from(value: TimeDistributionPDU) -> Self {
        SPDU::type2(value)
    }
}

impl From<StatusReports> for SPDU {
    fn from(value: StatusReports) -> Self {
        SPDU::type3(value)
    }
}

impl From<FirstGenLunar> for SPDU {
    fn from(value: FirstGenLunar) -> Self {
        SPDU::type4(value)
    }
}

impl From<SecondGenLunar> for SPDU {
    fn from(value: SecondGenLunar) -> Self {
        SPDU::type5(value)
    }
}

// These are the set of error types that SPDU parsing/encoding can return.
#[derive(Debug, Clone, PartialEq)]
pub enum SpduError {
    Truncated(&'static str), // This means the input byte slice did not have enough bytes to even read the required fields.
    Invalid(&'static str), // This means that the bytes were present, but they violated the wire-format rules.
    // The wire-format rules being (bad version/type bits, invalid fixed-length size, illegal values, etc.)
    LengthMismatch { declared: usize, actual: usize },
    // The length mismatch is used for variable-length SPDUs where the header declares a body length, but the data slice doesn't match.
    // Declared = length from the SPDU header
    // Actual = number of bytes actually provided after the header
    Unsupported(&'static str),
    // This is used for SPDUs that are not supported by the current implementation.
}

impl core::fmt::Display for SpduError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SpduError::Truncated(msg) => write!(f, "truncated: {msg}"),
            SpduError::Invalid(msg) => write!(f, "invalid: {msg}"),
            SpduError::LengthMismatch { declared, actual } => {
                write!(f, "length mismatch: declared {declared}, actual {actual}")
            }
            SpduError::Unsupported(msg) => write!(f, "unsupported: {msg}"),
        }
    }
}

impl std::error::Error for SpduError {}

impl SPDU {
    /// Decode an SPDU from its on-wire big-endian byte representation.
    // The from_bytes function returns a Result which is an enumeration that can be in one of two possible states: Ok or Err.
    // The Ok variant indicates the operation was successful, and it contains the successfully generated value.
    // The Err variant indicates the operation failed, and it contains information about how or why the operation failed.
    // In this case, the successfully generated value is the SPDU itself, and the information about how or why the operation failed is contained in the SpduError enum.
    // The SpduError enum is a user-defined type that contains information about the error that occurred.
    // The SpduError enum is defined in the mod.rs file.
    // The SpduError enum is defined as follows:
    // pub enum SpduError {
    //     Truncated(&'static str),
    //     Invalid(&'static str),
    //     LengthMismatch { declared: usize, actual: usize },
    //     Unsupported(&'static str),
    // }
    // The Truncated variant indicates that the input byte slice did not have enough bytes to even read the required fields.
    // The Invalid variant indicates that the bytes were present, but they violated the wire-format rules.
    // The LengthMismatch variant indicates that the length of the input byte slice did not match the length of the SPDU.
    pub fn from_bytes(data: &[u8]) -> Result<Self, SpduError> {
        let first = *data.first().ok_or(SpduError::Truncated("empty SPDU"))?;
        let format_id = (first & 0x80) != 0; // bit 0 (MSB)

        // All fixed-length SPDUs start with a format_id bit of 1.
        // If the format_id bit is 1, then the SPDU is a fixed-length SPDU.
        // If the format_id bit is 0, then the SPDU is a variable-length SPDU.
        if format_id {
            // Fixed-length SPDUs are 2 or 4 octets long.
            // If the SPDU is 2 octets long, then it is a 16-bit PLCW.
            if data.len() == 2 {
                let word = u16::from_be_bytes([data[0], data[1]]);
                let type_id = ((word >> 14) & 0x01) as u8;
                // Checks the type of PLCW (16-bit or 32-bit)
                if type_id != 0 {
                    return Err(SpduError::Invalid("F1 SPDU must have type identifier 0"));
                }
                Ok(SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(
                    PLCW16Bit::from_u16(word),
                )))
            }
            // If the SPDU is 4 octets long, then it is a 32-bit PLCW.
            else if data.len() == 4 {
                let word = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                let type_id = ((word >> 30) & 0x01) as u8;
                // Checks the type of PLCW (16-bit or 32-bit)
                if type_id != 1 {
                    return Err(SpduError::Invalid("F2 SPDU must have type identifier 1"));
                }
                Ok(SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(
                    PLCW32Bit::from_u32(word),
                )))
            } else {
                // If the SPDU is not 2 or 4 octets long, then it is invalid.
                Err(SpduError::Invalid(
                    "fixed-length SPDU must be 2 or 4 octets",
                ))
            }
        } else {
            // Variable-length SPDUs start with a format_id bit of 0. The type_id is the next 3 bits.
            // The length of the SPDU is the next 4 bits. The length is the number of octets in the body of the SPDU.
            // The body of the SPDU is the remaining bytes. The body is the data of the SPDU.
            let type_id = (first >> 4) & 0x07; // bits 1-3
            let len = (first & 0x0F) as usize; // bits 4-7
            let actual = data.len().saturating_sub(1);
            if len != actual {
                return Err(SpduError::LengthMismatch {
                    declared: len,
                    actual,
                });
            }
            if len > 15 {
                return Err(SpduError::Invalid(
                    "variable-length SPDU data length must be 0..15",
                ));
            }
            let body = &data[1..];
            let vl = match type_id {
                0b000 => VariableLengthSPDU::Type1(
                    DirectivesOrReportsUHF::from_bytes(body)
                        .map_err(|_| SpduError::Invalid("invalid Type 1 body"))?,
                ),
                0b001 => {
                    if body.len() != TimeDistributionPDU::LENGTH_OCTETS {
                        return Err(SpduError::Invalid("Type 2 body must be 15 octets"));
                    }
                    VariableLengthSPDU::Type2(
                        TimeDistributionPDU::from_bytes(body)
                            .map_err(|_| SpduError::Invalid("invalid Type 2 body"))?,
                    )
                }
                0b010 => VariableLengthSPDU::Type3(StatusReports { raw: body.to_vec() }),
                0b011 => VariableLengthSPDU::Type4(
                    FirstGenLunar::from_bytes(body)
                        .map_err(|_| SpduError::Invalid("invalid Type 4 body"))?,
                ),
                0b100 => VariableLengthSPDU::Type5(
                    SecondGenLunar::from_bytes(body)
                        .map_err(|_| SpduError::Invalid("invalid Type 5 body"))?,
                ),
                other => VariableLengthSPDU::Reserved(other, body.to_vec()),
            };
            Ok(SPDU::VariableLengthSPDU(vl))
        }
    }

    /// Encode an SPDU to its on-wire big-endian byte representation.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SpduError> {
        match self {
            SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(plcw)) => {
                Ok(plcw.to_u16().to_be_bytes().to_vec())
            }
            SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(plcw)) => {
                Ok(plcw.to_u32().to_be_bytes().to_vec())
            }
            SPDU::VariableLengthSPDU(vl) => {
                let (type_id, body): (u8, Vec<u8>) = match vl {
                    VariableLengthSPDU::Type1(x) => (
                        0b000,
                        x.to_bytes()
                            .map_err(|_| SpduError::Invalid("invalid Type 1 body"))?,
                    ),
                    VariableLengthSPDU::Type2(x) => (0b001, x.to_bytes().to_vec()),
                    VariableLengthSPDU::Type3(x) => (0b010, x.raw.clone()),
                    VariableLengthSPDU::Type4(x) => (
                        0b011,
                        x.to_bytes()
                            .map_err(|_| SpduError::Invalid("invalid Type 4 body"))?,
                    ),
                    VariableLengthSPDU::Type5(x) => (
                        0b100,
                        x.to_bytes()
                            .map_err(|_| SpduError::Invalid("invalid Type 5 body"))?,
                    ),
                    VariableLengthSPDU::Reserved(t, raw) => (*t & 0x07, raw.clone()),
                };

                if body.len() > 15 {
                    return Err(SpduError::Invalid(
                        "variable-length SPDU body may not exceed 15 octets",
                    ));
                }

                let header = ((type_id & 0x07) << 4) | ((body.len() as u8) & 0x0F);
                let mut out = Vec::with_capacity(1 + body.len());
                out.push(header);
                out.extend_from_slice(&body);
                Ok(out)
            }
        }
    }
}

// Implements the WireDecode trait for the SPDU enum
// The SPDU already implements the from_bytes function, so we can use that to implement the WireDecode trait.
// The trait implementations are thin wrappers: they do not add a second encoding path; they just forward to those methods so other code can depend on WireEncode/WireDecode without naming SPDU specifically.
impl WireDecode for SPDU {
    type Error = SpduError;

    fn from_wire_bytes(data: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(data)
    }
}

// Implements the WireEncode trait for the SPDU enum
// The SPDU already implements the to_bytes function, so we can use that to implement the WireEncode trait.
// The trait implementations are thin wrappers: they do not add a second encoding path; they just forward to those methods so other code can depend on WireEncode/WireDecode without naming SPDU specifically.
impl WireEncode for SPDU {
    type Error = SpduError;

    fn to_wire_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        self.to_bytes()
    }
}

impl TryFrom<&[u8]> for SPDU {
    type Error = SpduError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Import the bytes_to_hex and hex_to_bytes functions from the wire crate
    use crate::{bytes_to_hex, hex_to_bytes};

    /// Compare encoder output to a frozen reference hex vector (same style as interoperability tests).

    // The tests module block checks both directions.
    // Encoder correctness: does SPDU::to_bytes() produce the expected on-wire bytes?
    // Decoder correctness: does SPDU::from_bytes() reconstruct the same SPDU value?
    fn assert_spdu_bytes_match_vector(test_label: &str, actual: &[u8], vector_hex: &str) {
        // Convert the reference vector hex to bytes
        // If the conversion fails, panic with a message that includes the test label and the invalid hex
        let vector_bytes = hex_to_bytes(vector_hex).unwrap_or_else(|e| {
            panic!("{test_label}: invalid reference vector hex `{vector_hex}`: {e}");
        });

        let actual_hex = bytes_to_hex(actual);
        if actual != vector_bytes.as_slice() {
            panic!(
                "{test_label}: encoder output does not match reference test vector.\n\
                 • **Reference vector** (hex): `{vector_hex}`\n\
                 • **Reference bytes**       : {vector_bytes:02x?}\n\
                 • **Encoder output** (hex): `{actual_hex}`\n\
                 • **Encoder bytes**        : {actual:02x?}",
                vector_bytes = vector_bytes,
                actual_hex = actual_hex,
                actual = actual,
            );
        }
    }

    /// Workshop **artifact #1** shape: F1 PLCW, report=127, expedited=3, PCID=1, retransmit=1.
    #[test]
    fn spdu_fixed_f1_roundtrip() {
        // Reference Vector
        const VECTOR_HEX: &str = "b32a";

        // Constructed PLCW16Bit struct
        let plcw = PLCW16Bit {
            report_value: 42,
            expedited_frame_counter: 3,
            reserved_spare: false,
            pcid: true,
            retransmit_flag: true,
        };
        let pdu = SPDU::f1(plcw.clone());
        let bytes = pdu.to_bytes().unwrap();
        // Check the length of the bytes
        assert_eq!(bytes.len(), 2);
        // Check the bytes match the reference vector
        assert_spdu_bytes_match_vector(
            "spdu_fixed_f1_roundtrip (encode vs workshop vector)",
            &bytes,
            VECTOR_HEX,
        );

        // Check the decoded value matches the constructed SPDU
        let parsed = SPDU::from_bytes(&bytes).unwrap();
        // Check the parsed value matches the constructed PLCW16Bit struct
        assert_eq!(
            pdu, parsed,
            "round-trip: decoded value must match constructed SPDU"
        );
    }

    /// F2 round-trip against a frozen reference vector (workshop artifact style).
    #[test]
    fn spdu_fixed_f2_roundtrip() {
        // Reference Vector (workshop artifact #2): V(R)=500, expedited=6, PCID=1, retransmit=1.
        const VECTOR_HEX: &str = "c01e01f4";

        // Constructed PLCW32Bit struct (must encode to the reference vector above).
        let plcw = PLCW32Bit {
            report_value: 500,
            expedited_frame_counter: 6,
            pcid: true,
            retransmit_flag: true,
            reserved_spares: 0,
        };
        let pdu = SPDU::f2(plcw.clone());
        let bytes = pdu.to_bytes().unwrap();
        assert_eq!(bytes.len(), 4);
        assert_spdu_bytes_match_vector(
            "spdu_fixed_f2_roundtrip (encode vs computed vector)",
            &bytes,
            VECTOR_HEX,
        );

        let parsed = SPDU::from_bytes(&bytes).unwrap();
        assert_eq!(
            pdu, parsed,
            "round-trip: decoded value must match constructed PLCW"
        );
    }

    /// **INTOP-1.3** — Variable-length Type 1, SET V(R), `SEQ_CTRL_FSN = 42` (workshop / FR-9.5 artifact #3).
    #[test]
    fn spdu_variable_type1_roundtrip() {
        const VECTOR_HEX: &str = "02602a";

        let pdu = SPDU::type1(DirectivesOrReportsUHF::single(Type1Directive::set_vr(42)));

        let bytes = pdu.to_bytes().unwrap();
        assert_spdu_bytes_match_vector(
            "spdu_variable_type1_roundtrip / INTOP-1.3 (encode vs workshop vector)",
            &bytes,
            VECTOR_HEX,
        );

        let parsed = SPDU::from_bytes(&bytes).unwrap();
        assert_eq!(
            parsed, pdu,
            "round-trip: decoded value must match constructed Type 1 SPDU"
        );
    }
}
