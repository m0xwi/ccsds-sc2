//! FOP-P — **Frame Operation Procedure** (sender side), **§6.2**.

use super::shared::{CopError, CopFrame, ExpFrame, Seq, SeqFrame, SeqWidth, add_mod, dist_mod};

/// Sender-side COP-P state: sequencing, window, retransmission, PLCW handling.
///
/// Implements **§6.2** (including **§6.2.3.3** events SE0–SE4) at the level required for the
/// reference competition gateways.
#[derive(Debug, Clone)]
pub struct FopP {
    width: SeqWidth,
    transmission_window: u8, // max 127

    // Internal variables (6.2.2 + state table usage)
    pub ve_s: u8,   // modulo-8
    pub v_s: Seq,   // next new seq number to allocate
    pub vv_s: Seq,  // retransmission pointer
    pub n_r: Seq,   // last received report value
    pub nn_r: Seq,  // next unacknowledged (left edge)
    pub r_r: bool,  // last retransmit flag from PLCW
    pub rr_r: bool, // previous retransmit flag from PLCW (for validity checks)

    sent_queue: Vec<SeqFrame>,
    expedited_queue: Vec<Vec<u8>>,
    seq_queue: Vec<Vec<u8>>,
}

impl FopP {
    pub fn new(width: SeqWidth, transmission_window: u8) -> Self {
        let tw = transmission_window.min(127).max(1);
        Self {
            width,
            transmission_window: tw,
            ve_s: 0,
            v_s: Seq(0),
            vv_s: Seq(0),
            n_r: Seq(0),
            nn_r: Seq(0),
            r_r: false,
            rr_r: false,
            sent_queue: Vec::new(),
            expedited_queue: Vec::new(),
            seq_queue: Vec::new(),
        }
    }

    pub fn set_transmission_window(&mut self, tw: u8) {
        self.transmission_window = tw.min(127).max(1);
    }

    pub fn enqueue_expedited(&mut self, payload: Vec<u8>) {
        self.expedited_queue.push(payload);
    }

    pub fn enqueue_sequence_controlled(&mut self, payload: Vec<u8>) {
        self.seq_queue.push(payload);
    }

    /// Window occupancy: number of outstanding unacknowledged seq frames (V(S) - NN(R)).
    pub fn outstanding(&self) -> u32 {
        let m = self.width.modulus();
        dist_mod(self.nn_r.0, self.v_s.0, m)
    }

    pub fn is_window_full(&self) -> bool {
        self.outstanding() >= (self.transmission_window as u32)
    }

    /// Spec 6.2.3.3, event SE1: choose the next frame to transmit.
    pub fn next_frame_to_transmit(&mut self) -> Result<Option<CopFrame>, CopError> {
        let m = self.width.modulus();

        if let Some(payload) = self.expedited_queue.first().cloned() {
            self.expedited_queue.remove(0);
            let out = ExpFrame {
                ve_s: self.ve_s,
                payload,
            };
            self.ve_s = (self.ve_s + 1) & 0x07;
            return Ok(Some(CopFrame::Expedited(out)));
        }

        if self.vv_s.0 != self.v_s.0 {
            // Continue in-progress retransmission
            let ns = self.vv_s;
            if let Some(frame) = self.find_sent_cloned(ns) {
                self.vv_s = Seq(add_mod(self.vv_s.0, 1, m));
                return Ok(Some(CopFrame::SequenceControlled(frame)));
            }
        }

        if !self.seq_queue.is_empty() && !self.is_window_full() {
            // Send new sequence-controlled frame
            let payload = self.seq_queue.remove(0);
            let ns = self.v_s;
            self.v_s = Seq(add_mod(self.v_s.0, 1, m));
            // Keep VV(S) aligned with V(S) unless retransmission is underway.
            self.vv_s = self.v_s;
            let frame = SeqFrame { ns, payload };
            self.sent_queue.push(frame.clone());
            return Ok(Some(CopFrame::SequenceControlled(frame)));
        }

        // Initiate progressive retransmission if outstanding unacked exist.
        if self.nn_r.0 != self.v_s.0 {
            self.vv_s = self.nn_r;
            let ns = self.vv_s;
            if let Some(frame) = self.find_sent_cloned(ns) {
                self.vv_s = Seq(add_mod(self.vv_s.0, 1, m));
                return Ok(Some(CopFrame::SequenceControlled(frame)));
            }
        }

        Ok(None)
    }

    fn find_sent_cloned(&self, ns: Seq) -> Option<SeqFrame> {
        self.sent_queue.iter().find(|f| f.ns == ns).cloned()
    }

    /// Spec 6.2.3.3, event SE2: process a received PLCW (validity + window updates).
    pub fn on_plcw(&mut self, report_value: Seq, retransmit_flag: bool) -> Result<(), CopError> {
        let m = self.width.modulus();

        // Validity rules per note 5 in the spec.
        if dist_mod(self.nn_r.0, report_value.0, m) > dist_mod(self.nn_r.0, self.v_s.0, m) {
            return Err(CopError::InvalidPlcw("N(R) < NN(R) (too small)"));
        }

        if dist_mod(self.v_s.0, report_value.0, m) != 0
            && dist_mod(self.nn_r.0, report_value.0, m) > self.outstanding()
        {
            return Err(CopError::InvalidPlcw("N(R) > V(S) (too large)"));
        }

        if retransmit_flag && report_value.0 == self.v_s.0 {
            return Err(CopError::InvalidPlcw(
                "retransmit set but all frames are acknowledged",
            ));
        }

        if !retransmit_flag && self.rr_r && report_value.0 == self.nn_r.0 {
            return Err(CopError::InvalidPlcw(
                "retransmit cleared but no new frames acknowledged",
            ));
        }

        if report_value.0 != self.nn_r.0 {
            self.remove_acked(report_value);
            self.nn_r = report_value;
        }

        // Retransmission pointer update.
        if retransmit_flag || dist_mod(self.vv_s.0, report_value.0, m) != 0 {
            let vv_to_nr = dist_mod(self.vv_s.0, report_value.0, m);
            let vv_to_vs = dist_mod(self.vv_s.0, self.v_s.0, m);
            if retransmit_flag || vv_to_nr <= vv_to_vs {
                self.vv_s = report_value;
            }
        }

        self.rr_r = self.r_r;
        self.r_r = retransmit_flag;
        self.n_r = report_value;
        Ok(())
    }

    fn remove_acked(&mut self, new_nn_r: Seq) {
        let m = self.width.modulus();
        let nn = self.nn_r.0;
        let target = new_nn_r.0;

        self.sent_queue.retain(|f| {
            let dist_to_ns = dist_mod(target, f.ns.0, m);
            let dist_to_vs = dist_mod(target, self.v_s.0, m);
            dist_to_ns < dist_to_vs
        });

        if self.sent_queue.is_empty() {
            self.nn_r = Seq(target);
            self.vv_s = Seq(target);
            self.n_r = Seq(target);
        } else if nn != target {
            // nothing else needed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fop_window_blocks_new_frames() {
        let mut fop = FopP::new(SeqWidth::Mod256, 2);
        fop.enqueue_sequence_controlled(vec![1]);
        fop.enqueue_sequence_controlled(vec![2]);
        fop.enqueue_sequence_controlled(vec![3]);

        let _ = fop.next_frame_to_transmit().unwrap(); // ns=0
        let _ = fop.next_frame_to_transmit().unwrap(); // ns=1
        assert!(fop.is_window_full());
        let next = fop.next_frame_to_transmit().unwrap();
        assert!(matches!(next, Some(CopFrame::SequenceControlled(_))));
    }

    #[test]
    fn fop_ack_slides_window_and_allows_new() {
        let mut fop = FopP::new(SeqWidth::Mod256, 2);
        fop.enqueue_sequence_controlled(vec![1]);
        fop.enqueue_sequence_controlled(vec![2]);
        fop.enqueue_sequence_controlled(vec![3]);

        let _ = fop.next_frame_to_transmit().unwrap(); // ns=0
        let _ = fop.next_frame_to_transmit().unwrap(); // ns=1
        assert!(fop.is_window_full());

        fop.on_plcw(Seq(1), false).unwrap(); // ack ns=0
        assert!(!fop.is_window_full());

        let next = fop.next_frame_to_transmit().unwrap();
        match next {
            Some(CopFrame::SequenceControlled(f)) => assert_eq!(f.ns, Seq(2)),
            _ => panic!("expected new seq frame"),
        }
    }

    #[test]
    fn fop_retransmit_flag_moves_vv_s() {
        let mut fop = FopP::new(SeqWidth::Mod256, 4);
        fop.enqueue_sequence_controlled(vec![1]);
        fop.enqueue_sequence_controlled(vec![2]);
        let _ = fop.next_frame_to_transmit().unwrap(); // ns=0
        let _ = fop.next_frame_to_transmit().unwrap(); // ns=1

        fop.on_plcw(Seq(0), true).unwrap();
        assert_eq!(fop.vv_s, Seq(0));
        let next = fop.next_frame_to_transmit().unwrap();
        match next {
            Some(CopFrame::SequenceControlled(f)) => assert_eq!(f.ns, Seq(0)),
            _ => panic!("expected retransmit"),
        }
    }
}
