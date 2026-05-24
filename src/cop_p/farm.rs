//! FARM-P (Frame Acceptance and Reporting Mechanism — Proximity) — receiver-side COP.
//!
//! Implements state variables from **CCSDS 235.1-W-0.4 §6.3.2** and events **RE0–RE7** from §6.3.1.

use crate::spdu::{
    FixedLengthSPDU, PLCW16Bit, PLCW32Bit, SPDU, Type1Directive, VariableLengthSPDU,
};
use crate::frame::{Frame, FrameKind, Qos};

use super::seq::{Seq, SeqWidth, add_mod, greater_than, less_than};

/// Receiver-side COP-P state (FARM-P) per §6.3.2.
#[derive(Debug, Clone)]
pub struct FarmP {
    width: SeqWidth,

    /// R(S): retransmit-needed flag copied into PLCW.
    pub r_s: bool,

    /// V(R): expected next sequence number (modulo m).
    pub v_r: Seq,

    /// EXPEDITED_FRAME_COUNTER (modulo 8).
    pub expedited_frame_counter: u8,

    /// NEED_PLCW — report must be sent when true (§5.1.2.6, RE0/RE2/RE4/RE5).
    pub need_plcw: bool,

    /// NEED_STATUS_REPORT — paired with NEED_PLCW at initialization (§6.3.1 RE0).
    pub need_status_report: bool,
}

/// What FARM-P decided to do with a received frame (§6.3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FarmRx {
    /// RE3 / RE4 — frame passed to I/O sublayer.
    Accepted,
    /// RE5 — gap detected.
    DiscardedGap,
    /// RE6 — duplicate.
    DiscardedDuplicate,
    /// RE1 — invalid frame.
    DiscardedInvalid,
}

impl FarmP {
    /// **RE0** — initialization at session startup.
    pub fn new(width: SeqWidth) -> Self {
        Self {
            width,
            r_s: false,
            v_r: Seq(0),
            expedited_frame_counter: 0,
            need_plcw: true,
            need_status_report: true,
        }
    }

    /// **RE1** — invalid frame arrives.
    pub fn on_invalid_frame(&mut self) -> FarmRx {
        FarmRx::DiscardedInvalid
    }

    /// **RE2** — valid SET V(R) directive arrives.
    pub fn on_set_vr(&mut self, new_vr: Seq) {
        self.r_s = false;
        self.v_r = new_vr;
        self.need_plcw = true;
    }

    /// **RE3** — valid expedited frame arrives.
    pub fn on_expedited_frame(&mut self) -> FarmRx {
        self.expedited_frame_counter = (self.expedited_frame_counter + 1) & 0x07;
        FarmRx::Accepted
    }

    /// **RE4 / RE5 / RE6** — valid sequence-controlled frame arrives.
    pub fn on_sequence_frame(&mut self, n_s: Seq) -> FarmRx {
        let m = self.width.modulus();
        let ns = n_s.0 % m;
        let vr = self.v_r.0 % m;

        if ns == vr {
            self.r_s = false;
            self.v_r = Seq(add_mod(vr, 1, m));
            self.need_plcw = true;
            return FarmRx::Accepted;
        }

        if greater_than(n_s, self.v_r, self.width) {
            self.r_s = true;
            self.need_plcw = true;
            return FarmRx::DiscardedGap;
        }

        if less_than(n_s, self.v_r, self.width) {
            return FarmRx::DiscardedDuplicate;
        }

        FarmRx::DiscardedInvalid
    }

    /// **RE7** — build Type F1 PLCW from current state.
    pub fn plcw_f1(&self, pcid: bool) -> PLCW16Bit {
        PLCW16Bit {
            report_value: self.v_r.as_u8(),
            expedited_frame_counter: self.expedited_frame_counter,
            reserved_spare: false,
            pcid,
            retransmit_flag: self.r_s,
        }
    }

    /// **RE7** — build Type F2 PLCW from current state.
    pub fn plcw_f2(&self, pcid: bool) -> PLCW32Bit {
        PLCW32Bit {
            report_value: self.v_r.as_u16(),
            expedited_frame_counter: self.expedited_frame_counter,
            pcid,
            retransmit_flag: self.r_s,
            reserved_spares: 0,
        }
    }

    /// Mark PLCW as sent (§5.5.5 — NEED_PLCW cleared when PLCW is chosen for output).
    pub fn plcw_sent(&mut self) {
        self.need_plcw = false;
    }

    /// Process a validated transfer frame on the receive path.
    pub fn on_frame(&mut self, frame: &Frame) -> FarmFrameResult {
        match frame {
            Frame::V3(f) => self.on_frame_parts(f.kind, f.qos, f.seq, &f.payload),
            Frame::V4(f) => self.on_frame_parts(f.kind, f.qos, f.seq, &f.payload),
        }
    }

    fn on_frame_parts(
        &mut self,
        kind: FrameKind,
        qos: Qos,
        seq: Option<u16>,
        payload: &[u8],
    ) -> FarmFrameResult {
        let n_s = seq.map(|s| Seq((s & 0xFF) as u32));

        match kind {
            FrameKind::PFrame => self.on_pframe_payload(payload),
            FrameKind::UFrame => match qos {
                Qos::Expedited => {
                    let rx = self.on_expedited_frame();
                    FarmFrameResult {
                        rx,
                        io_payload: Some(payload.to_vec()),
                    }
                }
                Qos::SequenceControlled => {
                    let n_s = n_s.unwrap_or(Seq(0));
                    let rx = self.on_sequence_frame(n_s);
                    FarmFrameResult {
                        rx,
                        io_payload: if rx == FarmRx::Accepted {
                            Some(payload.to_vec())
                        } else {
                            None
                        },
                    }
                }
            },
        }
    }

    fn on_pframe_payload(&mut self, payload: &[u8]) -> FarmFrameResult {
        let spdu = match SPDU::from_bytes(payload) {
            Ok(s) => s,
            Err(_) => {
                return FarmFrameResult {
                    rx: self.on_invalid_frame(),
                    io_payload: None,
                };
            }
        };

        match spdu {
            SPDU::FixedLengthSPDU(_) => FarmFrameResult {
                rx: FarmRx::Accepted,
                io_payload: None,
            },
            SPDU::VariableLengthSPDU(VariableLengthSPDU::Type1(body)) => {
                for d in &body.directives {
                    if let Type1Directive::SetVR(sv) = d {
                        self.on_set_vr(Seq(sv.seq_ctrl_fsn as u32));
                    }
                }
                FarmFrameResult {
                    rx: FarmRx::Accepted,
                    io_payload: None,
                }
            }
            SPDU::VariableLengthSPDU(_) => FarmFrameResult {
                rx: FarmRx::Accepted,
                io_payload: None,
            },
        }
    }

    /// Parse an on-wire PLCW SPDU and return the typed word (F1 or F2).
    pub fn parse_plcw_spdu(bytes: &[u8]) -> Result<FarmPlcw, &'static str> {
        match SPDU::from_bytes(bytes) {
            Ok(SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(p))) => Ok(FarmPlcw::F1(p)),
            Ok(SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(p))) => Ok(FarmPlcw::F2(p)),
            Ok(_) => Err("not a PLCW SPDU"),
            Err(_) => Err("invalid PLCW bytes"),
        }
    }
}

/// Parsed PLCW from a received P-frame.
#[derive(Debug, Clone, PartialEq)]
pub enum FarmPlcw {
    F1(PLCW16Bit),
    F2(PLCW32Bit),
}

/// Result of processing one received frame through FARM-P.
#[derive(Debug, Clone, PartialEq)]
pub struct FarmFrameResult {
    pub rx: FarmRx,
    /// User data delivered to the I/O sublayer when `rx == Accepted` (§6.3.3).
    pub io_payload: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Version3Frame, Frame};

    #[test]
    fn re0_initializes_need_flags() {
        let farm = FarmP::new(SeqWidth::Mod256);
        assert!(farm.need_plcw);
        assert!(farm.need_status_report);
    }

    #[test]
    fn re4_in_sequence_accepts_and_increments_vr() {
        let mut farm = FarmP::new(SeqWidth::Mod256);
        assert_eq!(farm.v_r, Seq(0));
        assert_eq!(farm.on_sequence_frame(Seq(0)), FarmRx::Accepted);
        assert_eq!(farm.v_r, Seq(1));
        assert!(!farm.r_s);
        assert!(farm.need_plcw);
    }

    #[test]
    fn re5_gap_sets_retransmit_and_discards() {
        let mut farm = FarmP::new(SeqWidth::Mod256);
        assert_eq!(farm.on_sequence_frame(Seq(2)), FarmRx::DiscardedGap);
        assert!(farm.r_s);
        assert_eq!(farm.v_r, Seq(0));
        assert!(farm.need_plcw);
    }

    #[test]
    fn re6_duplicate_discards_without_moving_vr() {
        let mut farm = FarmP::new(SeqWidth::Mod256);
        let _ = farm.on_sequence_frame(Seq(0));
        assert_eq!(farm.v_r, Seq(1));
        assert_eq!(farm.on_sequence_frame(Seq(0)), FarmRx::DiscardedDuplicate);
        assert_eq!(farm.v_r, Seq(1));
    }

    #[test]
    fn re3_expedited_counter_wraps_mod8() {
        let mut farm = FarmP::new(SeqWidth::Mod256);
        for _ in 0..9 {
            farm.on_expedited_frame();
        }
        assert_eq!(farm.expedited_frame_counter, 1);
    }

    #[test]
    fn re2_set_vr_overrides_state() {
        let mut farm = FarmP::new(SeqWidth::Mod256);
        farm.r_s = true;
        farm.v_r = Seq(10);
        farm.on_set_vr(Seq(42));
        assert_eq!(farm.v_r, Seq(42));
        assert!(!farm.r_s);
        assert!(farm.need_plcw);
    }

    #[test]
    fn plcw_reflects_state() {
        let mut farm = FarmP::new(SeqWidth::Mod256);
        farm.v_r = Seq(7);
        farm.r_s = true;
        farm.expedited_frame_counter = 3;
        let plcw = farm.plcw_f1(false);
        assert_eq!(plcw.report_value, 7);
        assert!(plcw.retransmit_flag);
        assert_eq!(plcw.expedited_frame_counter, 3);
    }

    #[test]
    fn re7_clears_need_plcw_when_sent() {
        let mut farm = FarmP::new(SeqWidth::Mod256);
        assert!(farm.need_plcw);
        farm.plcw_sent();
        assert!(!farm.need_plcw);
    }

    #[test]
    fn on_uframe_sequence_controlled() {
        let mut farm = FarmP::new(SeqWidth::Mod256);
        let frame = Frame::V3(Version3Frame {
            kind: FrameKind::UFrame,
            qos: Qos::SequenceControlled,
            scid: 0,
            vcid: 0,
            seq: Some(0),
            payload: vec![1, 2, 3],
        });
        let r = farm.on_frame(&frame);
        assert_eq!(r.rx, FarmRx::Accepted);
        assert_eq!(r.io_payload, Some(vec![1, 2, 3]));
        assert_eq!(farm.v_r, Seq(1));
    }
}
