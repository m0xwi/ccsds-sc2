use super::{FrameError, FrameKind, Qos};

/// Minimal Version-3 (Proximity-1) transfer frame model for Gateway 3.
///
/// NOTE: CCSDS 235.1 references CCSDS 211.0-B-5 for the exact Version-3 header bit layout.
/// This implementation intentionally keeps header parsing/serialization compact and explicit
/// so it can be updated to the exact bit offsets once you want strict 211.0 compliance.
///
/// Current header layout used here (byte-aligned, for testability):
/// - byte 0:
///   - bits 7..6: version = 0b10
///   - bit 5: kind (0=P, 1=U)
///   - bit 4: qos  (0=expedited, 1=seq-controlled)
///   - bits 3..0: reserved (0)
/// - bytes 1..2: SCID (u16 big-endian)
/// - byte 3: VCID (low 6 bits used)
/// - if kind==U and qos==SequenceControlled: bytes 4..5 seq (u16 BE)
/// - remaining: payload
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version3Frame {
    pub kind: FrameKind,
    pub qos: Qos,
    pub scid: u16,
    pub vcid: u8,
    pub seq: Option<u16>,
    pub payload: Vec<u8>,
}

impl Version3Frame {
    pub(crate) fn from_bytes_without_crc(data: &[u8]) -> Result<Self, FrameError> {
        if data.len() < 4 {
            return Err(FrameError::Truncated("v3 header too short"));
        }
        let b0 = data[0];
        let version = b0 >> 6;
        if version != 0b10 {
            return Err(FrameError::Invalid("not a version-3 frame"));
        }
        let kind = if ((b0 >> 5) & 1) == 0 {
            FrameKind::PFrame
        } else {
            FrameKind::UFrame
        };
        let qos = if ((b0 >> 4) & 1) == 0 {
            Qos::Expedited
        } else {
            Qos::SequenceControlled
        };

        let scid = u16::from_be_bytes([data[1], data[2]]);
        let vcid = data[3] & 0x3F;

        let mut idx = 4;
        let seq = match (kind, qos) {
            (FrameKind::UFrame, Qos::SequenceControlled) => {
                if data.len() < idx + 2 {
                    return Err(FrameError::Truncated("missing sequence number"));
                }
                let s = u16::from_be_bytes([data[idx], data[idx + 1]]);
                idx += 2;
                Some(s)
            }
            _ => None,
        };

        Ok(Self {
            kind,
            qos,
            scid,
            vcid,
            seq,
            payload: data[idx..].to_vec(),
        })
    }

    pub(crate) fn to_bytes_without_crc(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut b0 = 0u8;
        b0 |= 0b10 << 6;
        if self.kind == FrameKind::UFrame {
            b0 |= 1 << 5;
        }
        if self.qos == Qos::SequenceControlled {
            b0 |= 1 << 4;
        }
        out.push(b0);
        out.extend_from_slice(&self.scid.to_be_bytes());
        out.push(self.vcid & 0x3F);
        if self.kind == FrameKind::UFrame && self.qos == Qos::SequenceControlled {
            let seq = self.seq.unwrap_or(0);
            out.extend_from_slice(&seq.to_be_bytes());
        }
        out.extend_from_slice(&self.payload);
        out
    }
}
