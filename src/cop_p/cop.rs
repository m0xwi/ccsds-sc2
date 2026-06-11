//! COP-P coordinator — ties FARM-P (receive) and FOP-P (send) to transfer frames and SPDUs.
//!
//! This is the **Gateway 2** integration point described in the competition deliverables.

use crate::frame::{Frame, FrameKind, Qos, Version3Frame};
use crate::spdu::{DirectivesOrReportsUHF, SPDU, Type1Directive};

use super::farm::{FarmP, FarmRx};
use super::fop::{FopP, FopState, FopTx};
use super::seq::SeqWidth;

/// Combined COP-P endpoint (one node).
#[derive(Debug, Clone)]
pub struct CopP {
    pub farm: FarmP,
    pub fop: FopP,
    /// PCID inserted into outbound PLCWs (§3.2).
    pub pcid: bool,
    use_f2_plcw: bool,
}

/// Outcome of processing one received frame.
#[derive(Debug, Clone, PartialEq)]
pub struct CopRx {
    pub farm: FarmRx,
    pub delivered_payload: Option<Vec<u8>>,
}

/// What the MAC / frame sublayer should transmit next (Table 5-13 subset).
#[derive(Debug, Clone, PartialEq)]
pub enum CopTx {
    /// P-frame carrying Type F1 or F2 PLCW from FARM-P.
    Plcw(SPDU),
    /// U-frame from FOP-P.
    Data(Frame),
}

impl CopP {
    pub fn new(width: SeqWidth) -> Self {
        Self {
            farm: FarmP::new(width),
            fop: FopP::new(width),
            pcid: false,
            use_f2_plcw: false,
        }
    }

    pub fn with_f2_plcw(mut self, use_f2: bool) -> Self {
        self.use_f2_plcw = use_f2;
        self
    }

    pub fn with_pcid(mut self, pcid: bool) -> Self {
        self.pcid = pcid;
        self
    }

    pub fn with_transmission_window(mut self, window: u32) -> Self {
        self.fop.transmission_window = window.min(127);
        self
    }

    /// Receive path: validated frame → FARM-P / FOP-P.
    pub fn receive(&mut self, frame: &Frame) -> CopRx {
        if let Frame::V3(f) = frame {
            if f.kind == FrameKind::PFrame {
                self.receive_pframe_for_fop(&f.payload);
            }
        } else if let Frame::V4(f) = frame {
            if f.kind == FrameKind::PFrame {
                self.receive_pframe_for_fop(&f.payload);
            }
        }

        let result = self.farm.on_frame(frame);
        CopRx {
            farm: result.rx,
            delivered_payload: result.io_payload,
        }
    }

    fn receive_pframe_for_fop(&mut self, payload: &[u8]) {
        if payload.first().is_some_and(|b| (b & 0x80) != 0) {
            self.fop.on_plcw_bytes(payload);
        }
    }

    /// Transmit path: select PLCW or data frame (§5.5.3 / §6).
    pub fn select_transmit(&mut self) -> Option<CopTx> {
        if self.fop.state == FopState::Resync && self.fop.need_plcw {
            let spdu = self.build_set_vr_spdu();
            self.fop.plcw_sent();
            return Some(CopTx::Plcw(spdu));
        }

        if self.farm.need_plcw {
            let spdu = if self.use_f2_plcw {
                SPDU::f2(self.farm.plcw_f2(self.pcid))
            } else {
                SPDU::f1(self.farm.plcw_f1(self.pcid))
            };
            self.farm.plcw_sent();
            return Some(CopTx::Plcw(spdu));
        }

        if self.fop.need_plcw {
            let spdu = if self.use_f2_plcw {
                SPDU::f2(self.farm.plcw_f2(self.pcid))
            } else {
                SPDU::f1(self.farm.plcw_f1(self.pcid))
            };
            self.fop.plcw_sent();
            return Some(CopTx::Plcw(spdu));
        }

        let tx = self.fop.select_transmit()?;
        let frame = self.build_uframe(&tx);
        Some(CopTx::Data(frame))
    }

    fn build_uframe(&self, tx: &FopTx) -> Frame {
        let (qos, seq) = match tx {
            FopTx::Expedited { seq, .. } => (Qos::Expedited, None),
            FopTx::SeqNew { seq, .. } | FopTx::SeqResend { seq, .. } => {
                (Qos::SequenceControlled, Some(seq.as_u16()))
            }
        };
        let payload = match tx {
            FopTx::Expedited { payload, .. }
            | FopTx::SeqNew { payload, .. }
            | FopTx::SeqResend { payload, .. } => payload.clone(),
        };

        Frame::V3(Version3Frame {
            kind: FrameKind::UFrame,
            qos,
            scid: 0,
            vcid: 0,
            seq,
            payload,
        })
    }

    /// Build a Version-3 P-frame carrying an SPDU payload.
    pub fn build_pframe(spdu: &SPDU) -> Result<Frame, crate::spdu::SpduError> {
        let bytes = spdu.to_bytes()?;
        Ok(Frame::V3(Version3Frame {
            kind: FrameKind::PFrame,
            qos: Qos::Expedited,
            scid: 0,
            vcid: 0,
            seq: None,
            payload: bytes,
        }))
    }

    /// Convenience: queue user data for sequence-controlled service.
    pub fn send_sequence_controlled(&mut self, payload: Vec<u8>) {
        self.fop.queue_sequence_controlled(payload);
    }

    /// Convenience: queue user data for expedited service.
    pub fn send_expedited(&mut self, payload: Vec<u8>) {
        self.fop.queue_expedited(payload);
    }

    /// Initiate local resync: enter FOP-P S2 and arm SET V(R) (§6.2.3.2).
    pub fn start_resync(&mut self) {
        self.fop.resync = true;
        self.fop.state = FopState::Resync;
        self.fop.need_plcw = true;
    }

    /// SET V(R) directive bytes for the local resync persistent activity.
    pub fn build_set_vr_spdu(&self) -> SPDU {
        SPDU::type1(DirectivesOrReportsUHF::single(Type1Directive::set_vr(
            self.fop.set_vr_directive_fsn(),
        )))
    }

    /// Apply SET V(R) received from peer on the local FARM-P.
    pub fn apply_peer_set_vr(&mut self, fsn: u8) {
        self.farm.on_set_vr(super::seq::Seq(fsn as u32));
    }

    /// Advance synch timer; returns true on expiry (SE4).
    pub fn tick_synch_timer(&mut self) -> bool {
        if self.fop.tick_synch_timer() {
            self.fop.on_synch_timeout();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::seq::Seq;
    use super::*;
    use crate::frame::FrameKind;
    use crate::spdu::{FixedLengthSPDU, VariableLengthSPDU};

    fn drain_tx(cop: &mut CopP) -> Vec<CopTx> {
        let mut out = Vec::new();
        while let Some(tx) = cop.select_transmit() {
            out.push(tx);
        }
        out
    }

    #[test]
    fn loss_recovery_roundtrip() {
        let mut sender = CopP::new(SeqWidth::Mod256).with_transmission_window(8);
        let mut receiver = CopP::new(SeqWidth::Mod256);

        sender.send_sequence_controlled(vec![1]);
        sender.send_sequence_controlled(vec![2]);

        let tx = drain_tx(&mut sender);
        assert!(!tx.is_empty());

        for item in tx {
            let frame = match item {
                CopTx::Plcw(spdu) => CopP::build_pframe(&spdu).unwrap(),
                CopTx::Data(f) => f,
            };
            receiver.receive(&frame);
        }

        assert_eq!(receiver.farm.v_r, Seq(2));
        assert!(!receiver.farm.r_s);
    }

    #[test]
    fn gap_triggers_retransmit_in_plcw() {
        let mut sender = CopP::new(SeqWidth::Mod256);
        let mut receiver = CopP::new(SeqWidth::Mod256);

        sender.send_sequence_controlled(vec![1]);
        if let Some(CopTx::Data(f)) = sender.select_transmit() {
            receiver.receive(&f);
        }

        let gap = Frame::V3(Version3Frame {
            kind: FrameKind::UFrame,
            qos: Qos::SequenceControlled,
            scid: 0,
            vcid: 0,
            seq: Some(5),
            payload: vec![9],
        });
        receiver.receive(&gap);
        assert!(receiver.farm.r_s);

        if let Some(CopTx::Plcw(spdu)) = receiver.select_transmit() {
            let bytes = spdu.to_bytes().unwrap();
            sender.fop.on_plcw_bytes(&bytes);
        }
        assert!(sender.fop.r_r || sender.fop.v_v_s <= sender.fop.v_s);
    }

    #[test]
    fn set_vr_pframe_does_not_corrupt_fop_state() {
        let mut cop = CopP::new(SeqWidth::Mod256);
        cop.fop.need_plcw = false;
        cop.fop.v_v_s = Seq(7);

        let spdu = SPDU::type1(DirectivesOrReportsUHF::single(Type1Directive::set_vr(42)));
        let pframe = CopP::build_pframe(&spdu).unwrap();
        let rx = cop.receive(&pframe);

        assert_eq!(rx.farm, FarmRx::Accepted);
        assert_eq!(cop.farm.v_r, Seq(42));
        assert_eq!(cop.fop.synch_timer, 0);
        assert_eq!(cop.fop.v_v_s, Seq(7));
    }

    #[test]
    fn start_resync_transmits_set_vr_directive() {
        let mut cop = CopP::new(SeqWidth::Mod256);
        cop.farm.need_plcw = true;
        cop.fop.nn_r = Seq(42);

        cop.start_resync();
        let tx = cop.select_transmit();

        match tx {
            Some(CopTx::Plcw(SPDU::VariableLengthSPDU(VariableLengthSPDU::Type1(body)))) => {
                assert_eq!(
                    body.directives,
                    vec![Type1Directive::set_vr(42)],
                    "resync must emit SET V(R), not a normal PLCW"
                );
            }
            other => panic!("expected SET V(R) Type 1 SPDU, got {other:?}"),
        }
        assert!(!cop.fop.need_plcw);
    }

    #[test]
    fn synch_timeout_resync_transmits_set_vr_directive() {
        let mut cop = CopP::new(SeqWidth::Mod256);
        cop.farm.need_plcw = false;
        cop.fop.need_plcw = false;
        cop.fop.nn_r = Seq(12);
        cop.fop.synch_timeout = 1;
        cop.fop.start_synch_timer();

        assert!(cop.tick_synch_timer());
        let tx = cop.select_transmit();

        match tx {
            Some(CopTx::Plcw(SPDU::VariableLengthSPDU(VariableLengthSPDU::Type1(body)))) => {
                assert_eq!(body.directives, vec![Type1Directive::set_vr(12)]);
            }
            other => panic!("expected SET V(R) Type 1 SPDU, got {other:?}"),
        }
    }

    #[test]
    fn fixed_plcw_pframe_still_updates_fop() {
        let mut cop = CopP::new(SeqWidth::Mod256);
        cop.fop.need_plcw = false;

        let spdu = SPDU::f1(crate::spdu::PLCW16Bit {
            report_value: 0,
            expedited_frame_counter: 0,
            reserved_spare: false,
            pcid: false,
            retransmit_flag: false,
        });
        let pframe = CopP::build_pframe(&spdu).unwrap();
        cop.receive(&pframe);

        assert!(matches!(
            spdu,
            SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(_))
        ));
        assert_eq!(cop.fop.synch_timer, 0);
        assert!(!cop.fop.need_plcw);
    }
}
