//! FOP-P (Frame Operation Procedure — Proximity) — sender-side COP.
//!
//! Implements **CCSDS 235.1-W-0.4 §6.2** variables, helper procedures (§6.2.3.1), and state-table
//! events **SE0–SE8** (§6.2.3.3).

use std::collections::VecDeque;

use crate::spdu::{FixedLengthSPDU, PLCW16Bit, PLCW32Bit, SPDU};

use super::seq::{Seq, SeqWidth, diff, greater_than, less_than};

/// FOP-P state (§6.2.3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FopState {
    /// S1 — Active.
    Active,
    /// S2 — Resync (SET V(R) persistent activity).
    Resync,
}

/// A sequence-controlled frame held in the sent queue (§6.2.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentFrame {
    pub seq: Seq,
    pub payload: Vec<u8>,
}

/// What SE1 selected for transmission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FopTx {
    /// Expedited U-frame (§6.2.3.1.6).
    Expedited { seq: Seq, payload: Vec<u8> },
    /// New sequence-controlled U-frame (§6.2.3.1.8).
    SeqNew { seq: Seq, payload: Vec<u8> },
    /// Retransmitted sequence-controlled U-frame (§6.2.3.1.7).
    SeqResend { seq: Seq, payload: Vec<u8> },
}

/// Sender-side COP-P state (FOP-P) per §6.2.2.
#[derive(Debug, Clone)]
pub struct FopP {
    width: SeqWidth,
    pub state: FopState,

    pub v_e_s: Seq,
    pub v_s: Seq,
    pub v_v_s: Seq,
    pub n_r: Seq,
    pub nn_r: Seq,
    pub r_r: bool,
    pub rr_r: bool,
    pub need_plcw: bool,
    pub need_status_report: bool,
    pub synch_timer: u32,
    pub synch_timeout: u32,
    pub resync: bool,
    pub transmission_window: u32,
    pub resync_local: bool,

    exp_queue: VecDeque<Vec<u8>>,
    seq_queue: VecDeque<Vec<u8>>,
    sent_queue: VecDeque<SentFrame>,
}

impl FopP {
    pub fn new(width: SeqWidth) -> Self {
        let mut fop = Self {
            width,
            state: FopState::Active,
            v_e_s: Seq(0),
            v_s: Seq(0),
            v_v_s: Seq(0),
            n_r: Seq(0),
            nn_r: Seq(0),
            r_r: false,
            rr_r: false,
            need_plcw: true,
            need_status_report: true,
            synch_timer: 0,
            synch_timeout: 100,
            resync: false,
            transmission_window: 127,
            resync_local: true,
            exp_queue: VecDeque::new(),
            seq_queue: VecDeque::new(),
            sent_queue: VecDeque::new(),
        };
        fop.initialize();
        fop
    }

    /// **SE0** / Initialize (§6.2.3.1.1).
    pub fn initialize(&mut self) {
        self.state = FopState::Active;
        self.v_e_s = Seq(0);
        self.v_s = Seq(0);
        self.v_v_s = Seq(0);
        self.n_r = Seq(0);
        self.nn_r = Seq(0);
        self.r_r = false;
        self.rr_r = false;
        self.resync = false;
        self.need_plcw = true;
        self.need_status_report = true;
        self.synch_timer = 0;
        self.clear_sent_queue();
        self.clear_seq_queue();
        self.clear_exp_queue();
    }

    pub fn clear_sent_queue(&mut self) {
        self.sent_queue.clear();
    }

    pub fn clear_seq_queue(&mut self) {
        self.seq_queue.clear();
    }

    pub fn clear_exp_queue(&mut self) {
        self.exp_queue.clear();
    }

    pub fn queue_expedited(&mut self, payload: Vec<u8>) {
        self.exp_queue.push_back(payload);
    }

    pub fn queue_sequence_controlled(&mut self, payload: Vec<u8>) {
        self.seq_queue.push_back(payload);
    }

    pub fn expedited_available(&self) -> bool {
        !self.exp_queue.is_empty()
    }

    pub fn sequence_controlled_available(&self) -> bool {
        !self.seq_queue.is_empty()
    }

    pub fn unacked_count(&self) -> u32 {
        diff(self.v_s, self.nn_r, self.width)
    }

    fn window_allows_new_seq(&self) -> bool {
        self.unacked_count() < self.transmission_window
    }

    /// **SE1** — Frame sublayer needs a frame to transmit (§6.2.3.3).
    pub fn select_transmit(&mut self) -> Option<FopTx> {
        if self.state == FopState::Resync {
            return None;
        }

        if self.expedited_available() {
            return Some(self.send_exp_frame());
        }

        if less_than(self.v_v_s, self.v_s, self.width) {
            return Some(self.resend_seq_frame());
        }

        if self.sequence_controlled_available() && self.window_allows_new_seq() {
            return Some(self.send_new_seq_frame());
        }

        None
    }

    fn send_exp_frame(&mut self) -> FopTx {
        let payload = self.exp_queue.pop_front().unwrap_or_default();
        let seq = self.v_e_s;
        self.v_e_s = Seq((self.v_e_s.0 + 1) % self.width.modulus());
        FopTx::Expedited { seq, payload }
    }

    fn send_new_seq_frame(&mut self) -> FopTx {
        let payload = self.seq_queue.pop_front().unwrap_or_default();
        let seq = self.v_s;
        self.sent_queue.push_back(SentFrame {
            seq,
            payload: payload.clone(),
        });
        let m = self.width.modulus();
        self.v_s = Seq((self.v_s.0 + 1) % m);
        self.v_v_s = Seq((self.v_v_s.0 + 1) % m);
        FopTx::SeqNew { seq, payload }
    }

    fn resend_seq_frame(&mut self) -> FopTx {
        let seq = self.v_v_s;
        let payload = self
            .sent_queue
            .iter()
            .find(|f| f.seq.0 % self.width.modulus() == seq.0 % self.width.modulus())
            .map(|f| f.payload.clone())
            .unwrap_or_default();
        let m = self.width.modulus();
        self.v_v_s = Seq((self.v_v_s.0 + 1) % m);
        FopTx::SeqResend { seq, payload }
    }

    /// Remove acknowledged frames from the sent queue (§6.2.3.1.2).
    pub fn remove_acknowledged_from_sent_queue(&mut self) {
        let n = diff(self.n_r, self.nn_r, self.width) as usize;
        for _ in 0..n {
            self.sent_queue.pop_front();
        }
    }

    /// Store this PLCW (§6.2.3.1.5).
    pub fn store_plcw(&mut self) {
        self.nn_r = self.n_r;
        self.rr_r = self.r_r;
    }

    pub fn clear_synch_timer(&mut self) {
        self.synch_timer = 0;
    }

    pub fn start_synch_timer(&mut self) {
        if self.synch_timer == 0 && self.synch_timeout > 0 {
            self.synch_timer = self.synch_timeout;
        }
    }

    /// Advance the synch timer by one tick; returns true if it expired this tick.
    pub fn tick_synch_timer(&mut self) -> bool {
        if self.synch_timer == 0 {
            return false;
        }
        self.synch_timer -= 1;
        self.synch_timer == 0
    }

    /// Validate an incoming PLCW (§6.2.3.3 note 5).
    pub fn is_valid_plcw(&self, n_r: Seq, r_r: bool) -> bool {
        if less_than(n_r, self.nn_r, self.width) {
            return false;
        }
        if greater_than(n_r, self.v_s, self.width) {
            return false;
        }
        if r_r && n_r.0 % self.width.modulus() == self.v_s.0 % self.width.modulus() {
            return false;
        }
        if !r_r && self.rr_r && n_r.0 % self.width.modulus() == self.nn_r.0 % self.width.modulus() {
            return false;
        }
        true
    }

    /// Apply report values from a Type F1 PLCW.
    pub fn apply_plcw_f1(&mut self, plcw: &PLCW16Bit) {
        self.apply_plcw(Seq(plcw.report_value as u32), plcw.retransmit_flag);
    }

    /// Apply report values from a Type F2 PLCW.
    pub fn apply_plcw_f2(&mut self, plcw: &PLCW32Bit) {
        self.apply_plcw(Seq(plcw.report_value as u32), plcw.retransmit_flag);
    }

    fn apply_plcw(&mut self, n_r: Seq, r_r: bool) {
        self.n_r = n_r;
        self.r_r = r_r;
    }

    /// **SE2** — Valid PLCW received (§6.2.3.3).
    pub fn on_valid_plcw(&mut self) {
        let m = self.width.modulus();
        let resync_complete = !self.r_r && self.n_r.0 % m == self.nn_r.0 % m;

        if greater_than(self.n_r, self.nn_r, self.width) {
            self.remove_acknowledged_from_sent_queue();
        }
        if self.r_r || greater_than(self.n_r, self.v_v_s, self.width) {
            self.v_v_s = self.n_r;
        }
        self.store_plcw();
        self.clear_synch_timer();
        self.need_plcw = false;

        if resync_complete {
            self.resync = false;
            self.state = FopState::Active;
        }
    }

    /// **SE3** — Invalid PLCW received (§6.2.3.3).
    pub fn on_invalid_plcw(&mut self) {
        self.start_synch_timer();
        self.v_v_s = self.nn_r;
    }

    /// **SE4** — Synch-timer expired (§6.2.3.3).
    pub fn on_synch_timeout(&mut self) {
        if self.resync_local {
            self.rr_r = false;
            self.resync = true;
            self.state = FopState::Resync;
            self.need_plcw = true;
        }
    }

    /// **SE7** — Reset request (§6.2.3.3).
    pub fn on_reset(&mut self) {
        self.initialize();
    }

    /// Process PLCW bytes from a received P-frame.
    pub fn on_plcw_bytes(&mut self, bytes: &[u8]) {
        match SPDU::from_bytes(bytes) {
            Ok(SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(p))) => {
                self.apply_plcw_f1(&p);
                if self.is_valid_plcw(self.n_r, self.r_r) {
                    self.on_valid_plcw();
                } else {
                    self.on_invalid_plcw();
                }
            }
            Ok(SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(p))) => {
                self.apply_plcw_f2(&p);
                if self.is_valid_plcw(self.n_r, self.r_r) {
                    self.on_valid_plcw();
                } else {
                    self.on_invalid_plcw();
                }
            }
            _ => self.on_invalid_plcw(),
        }
    }

    /// Build SET V(R) directive body bytes for resync (§6.2.3.2): SEQ_CTRL_FSN = NN(R).
    pub fn set_vr_directive_fsn(&self) -> u8 {
        self.nn_r.as_u8()
    }

    pub fn plcw_sent(&mut self) {
        self.need_plcw = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn se0_initialize() {
        let fop = FopP::new(SeqWidth::Mod256);
        assert_eq!(fop.state, FopState::Active);
        assert_eq!(fop.v_s, Seq(0));
        assert!(fop.need_plcw);
    }

    #[test]
    fn se1_send_new_seq_respects_window() {
        let mut fop = FopP::new(SeqWidth::Mod256);
        fop.transmission_window = 2;
        fop.queue_sequence_controlled(vec![1]);
        fop.queue_sequence_controlled(vec![2]);
        fop.queue_sequence_controlled(vec![3]);

        assert!(matches!(fop.select_transmit(), Some(FopTx::SeqNew { .. })));
        assert!(matches!(fop.select_transmit(), Some(FopTx::SeqNew { .. })));
        assert!(fop.select_transmit().is_none());
        assert_eq!(fop.unacked_count(), 2);
    }

    #[test]
    fn se2_valid_plcw_removes_acked_frames() {
        let mut fop = FopP::new(SeqWidth::Mod256);
        fop.queue_sequence_controlled(vec![10]);
        let _ = fop.select_transmit();
        fop.queue_sequence_controlled(vec![11]);
        let _ = fop.select_transmit();
        assert_eq!(fop.sent_queue.len(), 2);

        fop.n_r = Seq(1);
        fop.r_r = false;
        assert!(fop.is_valid_plcw(fop.n_r, fop.r_r));
        fop.on_valid_plcw();
        assert_eq!(fop.sent_queue.len(), 1);
    }

    #[test]
    fn se2_retransmit_flag_sets_vv_s() {
        let mut fop = FopP::new(SeqWidth::Mod256);
        fop.queue_sequence_controlled(vec![1]);
        let _ = fop.select_transmit();
        fop.nn_r = Seq(0);
        fop.n_r = Seq(0);
        fop.r_r = true;
        fop.on_valid_plcw();
        assert_eq!(fop.v_v_s, Seq(0));
    }

    #[test]
    fn se3_invalid_plcw_starts_synch() {
        let mut fop = FopP::new(SeqWidth::Mod256);
        fop.v_s = Seq(5);
        fop.nn_r = Seq(0);
        fop.n_r = Seq(10);
        fop.on_invalid_plcw();
        assert!(fop.synch_timer > 0);
        assert_eq!(fop.v_v_s, Seq(0));
    }

    #[test]
    fn progressive_retransmit_when_nn_r_behind() {
        let mut fop = FopP::new(SeqWidth::Mod256);
        fop.queue_sequence_controlled(vec![1]);
        let _ = fop.select_transmit();
        fop.queue_sequence_controlled(vec![2]);
        let _ = fop.select_transmit();
        fop.nn_r = Seq(0);
        fop.n_r = Seq(0);
        fop.r_r = true;
        fop.on_valid_plcw();
        let tx = fop.select_transmit();
        assert!(matches!(tx, Some(FopTx::SeqResend { .. })));
    }
}
