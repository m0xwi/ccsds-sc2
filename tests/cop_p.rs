//! Integration tests for the COP-P layer (Gateway 2).

use ccsds_sc2::{
    CopP, CopTx, DirectivesOrReportsUHF, FarmRx, FopState, FopTx, Frame, FrameKind, Qos, SPDU, Seq,
    SeqWidth, Type1Directive, Version3Frame,
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
fn set_vr_pframe_does_not_invalidate_fop_plcw_state() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.fop.synch_timeout = 3;

    let set_vr = SPDU::type1(DirectivesOrReportsUHF::single(Type1Directive::set_vr(42)));
    let frame = CopP::build_pframe(&set_vr).unwrap();
    let rx = node.receive(&frame);

    assert_eq!(rx.farm, FarmRx::Accepted);
    assert_eq!(node.farm.v_r.0, 42);
    assert_eq!(node.fop.synch_timer, 0);
    assert_eq!(node.fop.state, FopState::Active);
}

#[test]
fn start_resync_selects_set_vr_before_pending_plcw() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.fop.nn_r = Seq(42);

    node.start_resync();

    let Some(CopTx::Plcw(spdu)) = node.select_transmit() else {
        panic!("resync must emit a SET V(R) P-frame");
    };
    assert_eq!(spdu.to_bytes().unwrap(), vec![0x02, 0x60, 0x2A]);
}

#[test]
fn synch_timeout_resync_can_emit_set_vr() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.fop.synch_timeout = 1;
    node.fop.nn_r = Seq(42);
    node.fop.on_invalid_plcw();

    assert!(node.tick_synch_timer());
    assert_eq!(node.fop.state, FopState::Resync);

    let Some(CopTx::Plcw(spdu)) = node.select_transmit() else {
        panic!("synch-timeout resync must keep SET V(R) active");
    };
    assert_eq!(spdu.to_bytes().unwrap(), vec![0x02, 0x60, 0x2A]);
}

#[test]
fn mod65536_receive_uses_full_frame_sequence_number() {
    let mut node = CopP::new(SeqWidth::Mod65536);
    node.farm.v_r = Seq(300);
    let frame = Frame::V3(Version3Frame {
        kind: FrameKind::UFrame,
        qos: Qos::SequenceControlled,
        scid: 0,
        vcid: 0,
        seq: Some(300),
        payload: vec![1, 2, 3],
    });

    let rx = node.receive(&frame);

    assert_eq!(rx.farm, FarmRx::Accepted);
    assert_eq!(rx.delivered_payload, Some(vec![1, 2, 3]));
    assert_eq!(node.farm.v_r.0, 301);
}

#[test]
fn expedited_bypasses_sequence_window() {
    let mut fop_side = CopP::new(SeqWidth::Mod256);
    fop_side.fop.transmission_window = 0;
    fop_side.send_expedited(vec![1, 2, 3]);
    let tx = fop_side.fop.select_transmit();
    assert!(matches!(tx, Some(FopTx::Expedited { .. })));
}
