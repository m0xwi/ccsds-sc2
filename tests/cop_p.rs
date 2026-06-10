//! Integration tests for the COP-P layer (Gateway 2).

use ccsds_sc2::{
    CopP, CopTx, FarmRx, FopState, FopTx, Frame, FrameKind, Qos, SPDU, Seq, SeqWidth,
    Type1Directive, VariableLengthSPDU, Version3Frame,
};

#[test]
fn two_node_sequence_controlled_delivery() {
    let mut alice = CopP::new(SeqWidth::Mod256).with_transmission_window(4);
    let mut bob = CopP::new(SeqWidth::Mod256).with_transmission_window(4);

    alice.send_sequence_controlled(b"frame-0".to_vec());
    alice.send_sequence_controlled(b"frame-1".to_vec());

    for _ in 0..6 {
        if let Some(tx) = alice.select_transmit() {
            let frame = match tx {
                CopTx::Plcw(spdu) => CopP::build_pframe(&spdu).unwrap(),
                CopTx::Data(f) => f,
            };
            bob.receive(&frame);
        }
    }

    assert_eq!(bob.farm.v_r.0, 2);
    assert_eq!(bob.farm.r_s, false);
}

#[test]
fn plcw_retransmit_after_gap() {
    let mut sender = CopP::new(SeqWidth::Mod256);
    let mut receiver = CopP::new(SeqWidth::Mod256);

    sender.send_sequence_controlled(vec![42]);
    if let Some(CopTx::Data(f)) = sender.select_transmit() {
        receiver.receive(&f);
    }

    let skip = Frame::V3(Version3Frame {
        kind: FrameKind::UFrame,
        qos: Qos::SequenceControlled,
        scid: 0,
        vcid: 0,
        seq: Some(9),
        payload: vec![0],
    });
    let r = receiver.receive(&skip);
    assert_eq!(r.farm, FarmRx::DiscardedGap);
    assert!(receiver.farm.r_s);

    if let Some(CopTx::Plcw(spdu)) = receiver.select_transmit() {
        let bytes = spdu.to_bytes().unwrap();
        sender.fop.on_plcw_bytes(&bytes);
        assert!(sender.fop.r_r);
    }
}

#[test]
fn set_vr_resynchronizes_receiver() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.apply_peer_set_vr(100);
    assert_eq!(node.farm.v_r.0, 100);
    assert!(!node.farm.r_s);
}

#[test]
fn set_vr_pframe_does_not_corrupt_fop_state() {
    let mut peer = CopP::new(SeqWidth::Mod256);
    peer.fop.nn_r = Seq(100);
    let set_vr = CopP::build_pframe(&peer.build_set_vr_spdu()).unwrap();

    let mut node = CopP::new(SeqWidth::Mod256);
    node.fop.nn_r = Seq(7);
    node.fop.v_v_s = Seq(9);

    let rx = node.receive(&set_vr);

    assert_eq!(rx.farm, FarmRx::Accepted);
    assert_eq!(node.farm.v_r, Seq(100));
    assert_eq!(node.fop.synch_timer, 0);
    assert_eq!(node.fop.v_v_s, Seq(9));
}

#[test]
fn resync_transmit_emits_set_vr_directive() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.farm.need_plcw = false;
    node.fop.nn_r = Seq(42);

    node.start_resync();

    assert_set_vr_tx(node.select_transmit(), 42);
    assert_eq!(node.fop.state, FopState::Resync);
    assert!(!node.fop.need_plcw);
    assert!(node.select_transmit().is_none());
}

#[test]
fn synch_timeout_resync_emits_set_vr_directive() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.farm.need_plcw = false;
    node.fop.need_plcw = false;
    node.fop.nn_r = Seq(17);
    node.fop.synch_timer = 1;

    assert!(node.tick_synch_timer());

    assert_set_vr_tx(node.select_transmit(), 17);
}

#[test]
fn mod65536_receiver_preserves_high_sequence_bits() {
    let mut receiver = CopP::new(SeqWidth::Mod65536);
    receiver.farm.v_r = Seq(256);

    let frame = Frame::V3(Version3Frame {
        kind: FrameKind::UFrame,
        qos: Qos::SequenceControlled,
        scid: 0,
        vcid: 0,
        seq: Some(256),
        payload: b"seq-256".to_vec(),
    });

    let rx = receiver.receive(&frame);

    assert_eq!(rx.farm, FarmRx::Accepted);
    assert_eq!(rx.delivered_payload, Some(b"seq-256".to_vec()));
    assert_eq!(receiver.farm.v_r, Seq(257));
}

#[test]
fn expedited_bypasses_sequence_window() {
    let mut fop_side = CopP::new(SeqWidth::Mod256);
    fop_side.fop.transmission_window = 0;
    fop_side.send_expedited(vec![1, 2, 3]);
    let tx = fop_side.fop.select_transmit();
    assert!(matches!(tx, Some(FopTx::Expedited { .. })));
}

fn assert_set_vr_tx(tx: Option<CopTx>, expected_fsn: u8) {
    let Some(CopTx::Plcw(SPDU::VariableLengthSPDU(VariableLengthSPDU::Type1(body)))) = tx else {
        panic!("expected Type 1 SET V(R) P-frame payload");
    };

    assert_eq!(body.directives.len(), 1);
    match &body.directives[0] {
        Type1Directive::SetVR(set_vr) => {
            assert_eq!(set_vr.seq_ctrl_fsn, expected_fsn);
        }
        other => panic!("expected SET V(R), got {other:?}"),
    }
}
