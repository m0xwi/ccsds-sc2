//! Transfer **frames** and PLTU-style framing for Proximity-1 data services.
//!
//! **CCSDS 235.1-W-0.4 §5.4–5.6** describe how the Data Services layer interacts with the
//! **Coding & Synchronization** sublayer (ASM, coding) and the **Physical** layer. **Version-3**
//! and **Version-4** transfer frames are defined in **CCSDS 211.0** (Proximity-1 / USLP).
//!
//! This module provides:
//!
//! - [`Frame`] — Version-3 ([`Version3Frame`]) or Version-4 ([`Version4Frame`]) **transfer frame**
//!   with **P-frame** vs **U-frame** ([`FrameKind`]) and **QoS** ([`Qos`]).
//! - **CRC-16** trailer and optional **ASM** prefix ([`DEFAULT_ASM`], [`Frame::to_bytes_with_asm`])
//!   for workshop-style octet streams.
//!
//! # Compliance note
//!
//! Header bit layouts are a **simplified** byte-oriented model for tests; strict **211.0** header
//! packing should be verified against the published figures when interoperability requires it.

mod crc16;
mod v3;
mod v4;

pub use crc16::*;
pub use v3::*;
pub use v4::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameVersion {
    V2, // AOS (not fully supported here; expedited only per spec note)
    V3, // Prox-1
    V4, // USLP
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Qos {
    Expedited,
    SequenceControlled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    PFrame,
    UFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    Truncated(&'static str),
    Invalid(&'static str),
    BadCrc { expected: u16, computed: u16 },
    BadAsm,
}

impl core::fmt::Display for FrameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FrameError::Truncated(msg) => write!(f, "truncated: {msg}"),
            FrameError::Invalid(msg) => write!(f, "invalid: {msg}"),
            FrameError::BadCrc { expected, computed } => {
                write!(f, "bad CRC: expected 0x{expected:04x}, computed 0x{computed:04x}")
            }
            FrameError::BadAsm => write!(f, "bad ASM"),
        }
    }
}

impl std::error::Error for FrameError {}

/// Default Attached Sync Marker (ASM) used by many CCSDS links.
/// This is kept configurable at the API boundary; the spec references an ASM prepended at C&S.
pub const DEFAULT_ASM: [u8; 4] = [0x1A, 0xCF, 0xFC, 0x1D];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    V3(Version3Frame),
    V4(Version4Frame),
}

impl Frame {
    pub fn version(&self) -> FrameVersion {
        match self {
            Frame::V3(_) => FrameVersion::V3,
            Frame::V4(_) => FrameVersion::V4,
        }
    }

    pub fn kind(&self) -> FrameKind {
        match self {
            Frame::V3(f) => f.kind,
            Frame::V4(f) => f.kind,
        }
    }

    pub fn qos(&self) -> Qos {
        match self {
            Frame::V3(f) => f.qos,
            Frame::V4(f) => f.qos,
        }
    }

    /// Decode a transfer frame with an attached ASM and trailing CRC-16.
    pub fn from_bytes_with_asm(data: &[u8], asm: [u8; 4]) -> Result<Self, FrameError> {
        if data.len() < asm.len() + 1 + 2 {
            return Err(FrameError::Truncated("frame too short"));
        }
        if data[..4] != asm {
            return Err(FrameError::BadAsm);
        }
        Self::from_bytes(&data[asm.len()..])
    }

    /// Encode a transfer frame with an attached ASM and trailing CRC-16.
    pub fn to_bytes_with_asm(&self, asm: [u8; 4]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&asm);
        out.extend_from_slice(&self.to_bytes());
        out
    }

    /// Decode a transfer frame (without ASM) that ends with CRC-16.
    pub fn from_bytes(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 1 + 2 {
            return Err(FrameError::Truncated("missing header or CRC"));
        }
        let (without_crc, crc_bytes) = data.split_at(data.len() - 2);
        let expected = u16::from_be_bytes([crc_bytes[0], crc_bytes[1]]);
        let computed = crc16_ccitt_false(without_crc);
        if expected != computed {
            return Err(FrameError::BadCrc { expected, computed });
        }

        let first = without_crc[0];
        let version_bits = first >> 6; // first two bits of header
        match version_bits {
            0b10 => Ok(Frame::V3(Version3Frame::from_bytes_without_crc(without_crc)?)),
            0b11 => Ok(Frame::V4(Version4Frame::from_bytes_without_crc(without_crc)?)),
            0b01 => Err(FrameError::Invalid("version-2 frames not supported by this decoder")),
            _ => Err(FrameError::Invalid("version bits 00 are invalid")),
        }
    }

    /// Encode a transfer frame (without ASM) and append CRC-16.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut body = match self {
            Frame::V3(f) => f.to_bytes_without_crc(),
            Frame::V4(f) => f.to_bytes_without_crc(),
        };
        let crc = crc16_ccitt_false(&body);
        body.extend_from_slice(&crc.to_be_bytes());
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spdu::{FixedLengthSPDU, PLCW16Bit, SPDU};

    #[test]
    fn frame_v3_pframe_roundtrip_with_crc_and_asm() {
        let pdu = SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(PLCW16Bit {
            report_value: 7,
            expedited_frame_counter: 1,
            reserved_space: false,
            pcid: false,
            retransmit_flag: false,
        }));

        let f = Frame::V3(Version3Frame {
            kind: FrameKind::PFrame,
            qos: Qos::Expedited,
            scid: 0x123,
            vcid: 2,
            seq: None,
            payload: pdu.to_bytes().unwrap(),
        });

        let bytes = f.to_bytes_with_asm(DEFAULT_ASM);
        let parsed = Frame::from_bytes_with_asm(&bytes, DEFAULT_ASM).unwrap();
        assert_eq!(f, parsed);
    }

    #[test]
    fn frame_bad_crc_rejected() {
        let f = Frame::V3(Version3Frame {
            kind: FrameKind::UFrame,
            qos: Qos::SequenceControlled,
            scid: 1,
            vcid: 0,
            seq: Some(7),
            payload: vec![0, 1, 2, 3],
        });

        let mut bytes = f.to_bytes_with_asm(DEFAULT_ASM);
        // flip one payload bit after ASM
        bytes[DEFAULT_ASM.len() + 5] ^= 0x01;
        let err = Frame::from_bytes_with_asm(&bytes, DEFAULT_ASM).unwrap_err();
        assert!(matches!(err, FrameError::BadCrc { .. }));
    }
}

