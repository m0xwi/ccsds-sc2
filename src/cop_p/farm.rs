//! FARM-P — **Frame Acceptance and Reporting Mechanism** (receiver side), **§6.3**.

use crate::{PLCW16Bit, PLCW32Bit};

use super::shared::{Seq, SeqWidth, add_mod, dist_mod};

/// Receiver-side COP-P state: `V(R)`, gap detection, expedited counter, PLCW fields.
///
/// Implements **§6.3** (including receiver table **RE0–RE7** behavior) for the competition scope.
#[derive(Debug, Clone)]
pub struct FarmP {
    width: SeqWidth,
    pub r_s: bool, // retransmit needed flag (copied into PLCW)
    pub v_r: Seq,
    pub expedited_frame_counter: u8, // modulo-8
    pub lockout_flag: bool,
    pub wait_flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FarmRx {
    Accepted,
    DiscardedGap,
    DiscardedDuplicate,
    DiscardedInvalid,
}

impl FarmP {
    pub fn new(width: SeqWidth) -> Self {
        Self {
            width,
            r_s: false,
            v_r: Seq(0),
            expedited_frame_counter: 0,
            lockout_flag: false,
            wait_flag: false,
        }
    }

    pub fn on_set_vr(&mut self, seq_ctrl_fsn: Seq) {
        self.r_s = false;
        self.v_r = seq_ctrl_fsn;
    }

    pub fn on_expedited_frame(&mut self) {
        self.expedited_frame_counter = (self.expedited_frame_counter + 1) & 0x07;
    }

    pub fn on_sequence_frame(&mut self, n_s: Seq) -> FarmRx {
        let m = self.width.modulus();
        if n_s.0 == self.v_r.0 {
            self.r_s = false;
            self.v_r = Seq(add_mod(self.v_r.0, 1, m));
            FarmRx::Accepted
        } else if dist_mod(self.v_r.0, n_s.0, m) > 0 && dist_mod(self.v_r.0, n_s.0, m) < (m / 2) {
            self.r_s = true;
            FarmRx::DiscardedGap
        } else {
            FarmRx::DiscardedDuplicate
        }
    }

    pub fn plcw_f1(&self, pcid: bool) -> PLCW16Bit {
        PLCW16Bit {
            report_value: self.v_r.as_u8(),
            expedited_frame_counter: self.expedited_frame_counter,
            reserved_spare: false,
            pcid,
            retransmit_flag: self.r_s,
        }
    }

    pub fn plcw_f2(&self, pcid: bool) -> PLCW32Bit {
        PLCW32Bit {
            report_value: self.v_r.as_u16(),
            expedited_frame_counter: self.expedited_frame_counter,
            pcid,
            retransmit_flag: self.r_s,
            lockout_flag: self.lockout_flag,
            wait_flag: self.wait_flag,
            reserved_spares: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn farm_gap_sets_retransmit_and_plcw_reflects() {
        let mut farm = FarmP::new(SeqWidth::Mod256);
        assert_eq!(farm.v_r, Seq(0));
        let rx = farm.on_sequence_frame(Seq(2));
        assert_eq!(rx, FarmRx::DiscardedGap);
        assert!(farm.r_s);
        let plcw = farm.plcw_f1(false);
        assert!(plcw.retransmit_flag);
        assert_eq!(plcw.report_value, 0);
    }

    #[test]
    fn farm_in_sequence_accepts_and_increments() {
        let mut farm = FarmP::new(SeqWidth::Mod256);
        let rx = farm.on_sequence_frame(Seq(0));
        assert_eq!(rx, FarmRx::Accepted);
        assert_eq!(farm.v_r, Seq(1));
        assert!(!farm.r_s);
    }
}
