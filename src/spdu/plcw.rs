//! Proximity Link Control Word (PLCW) fixed-length SPDU payloads.
//!
//! These are the fixed-length SPDUs (Format ID = 1):
//! - **F1**: 16-bit PLCW
//! - **F2**: 32-bit PLCW

#[derive(Debug, Clone, PartialEq)]
pub struct PLCW16Bit {
    pub report_value: u8,            // 8 bits (V(R))
    pub expedited_frame_counter: u8, // 3 bits
    pub reserved_spare: bool,        // 1 bit
    pub pcid: bool,                  // 1 bit
    pub retransmit_flag: bool,       // 1 bit
}

#[derive(Debug, Clone, PartialEq)]
pub struct PLCW32Bit {
    pub report_value: u16,           // 16 bits (V(R))
    pub expedited_frame_counter: u8, // 3 bits
    pub pcid: bool,                  // 1 bit
    pub retransmit_flag: bool,       // 1 bit
    /// Bits **21–29** of the 32-bit Type F2 PLCW word (reserved spares region on the wire).
    ///
    /// This is a single field so callers do not model individual spare names; use **0** when
    /// unspecified. Valid range is **0..=0x1FF** (9 bits).
    pub reserved_spares: u16,
}

impl PLCW16Bit {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.expedited_frame_counter > 7 {
            return Err("F1 expedited_frame_counter must be 0..7");
        }
        Ok(())
    }

    pub fn from_u16(word: u16) -> Self {
        PLCW16Bit {
            report_value: (word & 0x00FF) as u8,
            expedited_frame_counter: ((word >> 8) & 0x07) as u8,
            reserved_spare: (word & (1 << 11)) != 0,
            pcid: (word & (1 << 12)) != 0,
            retransmit_flag: (word & (1 << 13)) != 0,
        }
    }

    pub fn to_u16(&self) -> u16 {
        // Keep encoding “exact” by rejecting values that would be truncated.
        // (Masks below are still applied as a final safety net.)
        let _ = self.validate();
        let mut word = 0u16;
        word |= self.report_value as u16;
        word |= ((self.expedited_frame_counter & 0x07) as u16) << 8;
        if self.reserved_spare {
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
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.expedited_frame_counter > 7 {
            return Err("F2 expedited_frame_counter must be 0..7");
        }
        if self.reserved_spares > 0x1FF {
            return Err("F2 reserved_spares must be 0..=0x1FF (9 bits)");
        }
        Ok(())
    }

    pub fn from_u32(word: u32) -> Self {
        let reserved_spares = ((word >> 21) & 0x1FF) as u16;

        PLCW32Bit {
            report_value: (word & 0xFFFF) as u16,
            expedited_frame_counter: ((word >> 16) & 0x07) as u8,
            pcid: (word & (1 << 19)) != 0,
            retransmit_flag: (word & (1 << 20)) != 0,
            reserved_spares,
        }
    }

    pub fn to_u32(&self) -> u32 {
        let _ = self.validate();
        let mut word = 0u32;
        word |= self.report_value as u32;
        word |= ((self.expedited_frame_counter & 0x07) as u32) << 16;
        if self.pcid {
            word |= 1 << 19;
        }
        if self.retransmit_flag {
            word |= 1 << 20;
        }
        word |= ((self.reserved_spares as u32) & 0x1FF) << 21;
        word |= 1 << 31; // format_id=1
        word |= 1 << 30; // type_id=1
        word
    }
}
