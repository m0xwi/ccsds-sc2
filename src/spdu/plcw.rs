//! Proximity Link Control Word (PLCW) fixed-length SPDU payloads.
//!
//! These are the fixed-length SPDUs (Format ID = 1):
//! - **F1**: 16-bit PLCW
//! - **F2**: 32-bit PLCW

#[derive(Debug, Clone, PartialEq)]
pub struct PLCW16Bit {
    pub report_value: u8,            // 8 bits (V(R))
    pub expedited_frame_counter: u8, // 3 bits
    pub reserved_space: bool,        // 1 bit
    pub pcid: bool,                  // 1 bit
    pub retransmit_flag: bool,       // 1 bit
}

#[derive(Debug, Clone, PartialEq)]
pub struct PLCW32Bit {
    pub report_value: u16,           // 16 bits (V(R))
    pub expedited_frame_counter: u8, // 3 bits
    pub pcid: bool,                  // 1 bit
    pub retransmit_flag: bool,       // 1 bit
    /// Within the reserved spares region; required by competition requirements.
    pub lockout_flag: bool,
    /// Within the reserved spares region; required by competition requirements.
    pub wait_flag: bool,
    /// Remaining spare bits (excluding lockout/wait).
    pub reserved_spares: u8,
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

