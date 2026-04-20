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
//! # Wire format
//!
//! [`SPDU::to_bytes`] / [`SPDU::from_bytes`] use **big-endian** octet order for multi-byte fields.
//!
//! # See also
//!
//! - COP-P consumes and generates PLCWs — [`crate::cop_p`].

// This is the top-level SPDU API and defines the wire-format rules.
// Wire-format rules are the exact bytes that define the byte layout of the SPDU on the wire
// The wire-format rules include:
/// 1. What kind of SPDU it is
/// 2. The endianness
/// 3. The validation constraints



mod bits;
mod type1;
mod type2;
mod type4;
mod type5;

pub use type1::*;
pub use type2::*;
pub use type4::*;
pub use type5::*;

/// Top-level SPDU: fixed (Format ID = 1) or variable-length (Format ID = 0).
///
/// See **§3.1** and Tables 3-1 / 3-2.
#[derive(Debug, Clone, PartialEq)]
pub enum SPDU {
    FixedLengthSPDU(FixedLengthSPDU),      // Format ID = 1
    VariableLengthSPDU(VariableLengthSPDU), // Format ID = 0
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixedLengthSPDU {
    F1(PLCW16Bit), // SPDU Type ID = 0 -> 16-bit PLCW
    F2(PLCW32Bit), // SPDU Type ID = 1 -> 32-bit PLCW
}

#[derive(Debug, Clone, PartialEq)]
pub struct PLCW16Bit {
    pub report_value: u8,              // 8 bits (V(R))
    pub expedited_frame_counter: u8,   // 3 bits
    pub reserved_space: bool,          // 1 bit
    pub pcid: bool,                    // 1 bit
    pub retransmit_flag: bool,         // 1 bit
}

#[derive(Debug, Clone, PartialEq)]
pub struct PLCW32Bit {
    pub report_value: u16,             // 16 bits (V(R))
    pub expedited_frame_counter: u8,   // 3 bits
    pub pcid: bool,                    // 1 bit
    pub retransmit_flag: bool,         // 1 bit
    // Spec figure 3-2 shows 9 spare bits; project requirements additionally call out
    // `lockout_flag` and `wait_flag`. We carry these inside the spare region while
    // keeping a 32-bit wire footprint.
    pub lockout_flag: bool,            // 1 bit (within reserved spares)
    pub wait_flag: bool,               // 1 bit (within reserved spares)
    pub reserved_spares: u8,           // remaining 7 spare bits
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

#[derive(Debug, Clone, PartialEq)]
pub enum SpduError {
    Truncated(&'static str),
    Invalid(&'static str),
    LengthMismatch { declared: usize, actual: usize },
    Unsupported(&'static str),
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
    pub fn from_bytes(data: &[u8]) -> Result<Self, SpduError> {
        let first = *data.first().ok_or(SpduError::Truncated("empty SPDU"))?;
        let format_id = (first & 0x80) != 0; // bit 0 (MSB)

        if format_id {
            if data.len() == 2 {
                let word = u16::from_be_bytes([data[0], data[1]]);
                let type_id = ((word >> 14) & 0x01) as u8;
                if type_id != 0 {
                    return Err(SpduError::Invalid("F1 SPDU must have type identifier 0"));
                }
                Ok(SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(PLCW16Bit::from_u16(word))))
            } else if data.len() == 4 {
                let word = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                let type_id = ((word >> 30) & 0x01) as u8;
                if type_id != 1 {
                    return Err(SpduError::Invalid("F2 SPDU must have type identifier 1"));
                }
                Ok(SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(PLCW32Bit::from_u32(word))))
            } else {
                Err(SpduError::Invalid("fixed-length SPDU must be 2 or 4 octets"))
            }
        } else {
            let type_id = (first >> 4) & 0x07; // bits 1-3
            let len = (first & 0x0F) as usize; // bits 4-7
            let actual = data.len().saturating_sub(1);
            if len != actual {
                return Err(SpduError::LengthMismatch { declared: len, actual });
            }
            if len > 15 {
                return Err(SpduError::Invalid("variable-length SPDU data length must be 0..15"));
            }
            let body = &data[1..];
            let vl = match type_id {
                0b000 => VariableLengthSPDU::Type1(
                    DirectivesOrReportsUHF::from_bytes(body).map_err(|_| SpduError::Invalid("invalid Type 1 body"))?,
                ),
                0b001 => {
                    if body.len() != TimeDistributionPDU::LENGTH_OCTETS {
                        return Err(SpduError::Invalid("Type 2 body must be 15 octets"));
                    }
                    VariableLengthSPDU::Type2(
                        TimeDistributionPDU::from_bytes(body).map_err(|_| SpduError::Invalid("invalid Type 2 body"))?,
                    )
                }
                0b010 => VariableLengthSPDU::Type3(StatusReports { raw: body.to_vec() }),
                0b011 => VariableLengthSPDU::Type4(
                    FirstGenLunar::from_bytes(body).map_err(|_| SpduError::Invalid("invalid Type 4 body"))?,
                ),
                0b100 => VariableLengthSPDU::Type5(
                    SecondGenLunar::from_bytes(body).map_err(|_| SpduError::Invalid("invalid Type 5 body"))?,
                ),
                other => VariableLengthSPDU::Reserved(other, body.to_vec()),
            };
            Ok(SPDU::VariableLengthSPDU(vl))
        }
    }

    /// Encode an SPDU to its on-wire big-endian byte representation.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SpduError> {
        match self {
            SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(plcw)) => Ok(plcw.to_u16().to_be_bytes().to_vec()),
            SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(plcw)) => Ok(plcw.to_u32().to_be_bytes().to_vec()),
            SPDU::VariableLengthSPDU(vl) => {
                let (type_id, body): (u8, Vec<u8>) = match vl {
                    VariableLengthSPDU::Type1(x) => {
                        (0b000, x.to_bytes().map_err(|_| SpduError::Invalid("invalid Type 1 body"))?)
                    }
                    VariableLengthSPDU::Type2(x) => (0b001, x.to_bytes().to_vec()),
                    VariableLengthSPDU::Type3(x) => (0b010, x.raw.clone()),
                    VariableLengthSPDU::Type4(x) => {
                        (0b011, x.to_bytes().map_err(|_| SpduError::Invalid("invalid Type 4 body"))?)
                    }
                    VariableLengthSPDU::Type5(x) => {
                        (0b100, x.to_bytes().map_err(|_| SpduError::Invalid("invalid Type 5 body"))?)
                    }
                    VariableLengthSPDU::Reserved(t, raw) => (*t & 0x07, raw.clone()),
                };

                if body.len() > 15 {
                    return Err(SpduError::Invalid("variable-length SPDU body may not exceed 15 octets"));
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

impl PLCW16Bit {
    pub fn from_u16(word: u16) -> Self {
        PLCW16Bit {
            report_value: (word & 0x00FF) as u8,
            expedited_frame_counter: ((word >> 8) & 0x07) as u8,
            reserved_space: (word & (1 << 11)) != 0,
            pcid: (word & (1 << 12)) != 0,
            retransmit_flag: (word & (1 << 13)) != 0,
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = 0u16;
        word |= self.report_value as u16;
        word |= ((self.expedited_frame_counter & 0x07) as u16) << 8;
        if self.reserved_space {
            word |= 1 << 11;
        }
        if self.pcid {
            word |= 1 << 12;
        }
        if self.retransmit_flag {
            word |= 1 << 13;
        }
        word |= 1 << 15; // format_id=1
        word
    }
}

impl PLCW32Bit {
    pub fn from_u32(word: u32) -> Self {
        let lockout_flag = (word & (1 << 29)) != 0;
        let wait_flag = (word & (1 << 28)) != 0;
        let reserved_spares = ((word >> 21) & 0x7F) as u8;

        PLCW32Bit {
            report_value: (word & 0xFFFF) as u16,
            expedited_frame_counter: ((word >> 16) & 0x07) as u8,
            pcid: (word & (1 << 19)) != 0,
            retransmit_flag: (word & (1 << 20)) != 0,
            lockout_flag,
            wait_flag,
            reserved_spares,
        }
    }

    pub fn to_u32(&self) -> u32 {
        let mut word = 0u32;
        word |= self.report_value as u32;
        word |= ((self.expedited_frame_counter & 0x07) as u32) << 16;
        if self.pcid {
            word |= 1 << 19;
        }
        if self.retransmit_flag {
            word |= 1 << 20;
        }
        if self.lockout_flag {
            word |= 1 << 29;
        }
        if self.wait_flag {
            word |= 1 << 28;
        }
        word |= ((self.reserved_spares as u32) & 0x7F) << 21;
        word |= 1 << 31; // format_id=1
        word |= 1 << 30; // type_id=1
        word
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spdu_fixed_f1_roundtrip() {
        let plcw = PLCW16Bit {
            report_value: 127,
            expedited_frame_counter: 3,
            reserved_space: false,
            pcid: false,
            retransmit_flag: false,
        };
        let pdu = SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(plcw.clone()));
        let bytes = pdu.to_bytes().unwrap();
        assert_eq!(bytes.len(), 2);
        let parsed = SPDU::from_bytes(&bytes).unwrap();
        assert_eq!(pdu, parsed);
        if let SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(p)) = parsed {
            assert_eq!(p, plcw);
        } else {
            panic!("wrong SPDU variant");
        }
    }

    #[test]
    fn spdu_fixed_f2_roundtrip() {
        let plcw = PLCW32Bit {
            report_value: 500,
            expedited_frame_counter: 6,
            pcid: true,
            retransmit_flag: true,
            lockout_flag: false,
            wait_flag: true,
            reserved_spares: 0,
        };
        let pdu = SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(plcw.clone()));
        let bytes = pdu.to_bytes().unwrap();
        assert_eq!(bytes.len(), 4);
        let parsed = SPDU::from_bytes(&bytes).unwrap();
        assert_eq!(pdu, parsed);
    }
}

