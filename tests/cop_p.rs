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
fn set_vr_pframe_does_not_corrupt_fop_state() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.send_sequence_controlled(vec![1]);
    node.send_sequence_controlled(vec![2]);
    assert!(matches!(
        node.fop.select_transmit(),
        Some(FopTx::SeqNew { .. })
    ));
    assert!(matches!(
        node.fop.select_transmit(),
        Some(FopTx::SeqNew { .. })
    ));
    assert_eq!(node.fop.v_v_s, Seq(2));

    let set_vr = SPDU::type1(DirectivesOrReportsUHF::single(Type1Directive::set_vr(42)));
    let pframe = CopP::build_pframe(&set_vr).unwrap();

    let rx = node.receive(&pframe);

    assert_eq!(rx.farm, FarmRx::Accepted);
    assert_eq!(node.farm.v_r, Seq(42));
    assert_eq!(node.fop.state, FopState::Active);
    assert_eq!(node.fop.synch_timer, 0);
    assert_eq!(node.fop.v_v_s, Seq(2));
}

#[test]
fn mod65536_sequence_controlled_receive_uses_full_sequence_number() {
    let mut receiver = CopP::new(SeqWidth::Mod65536);
    receiver.farm.v_r = Seq(300);
    let frame = Frame::V3(Version3Frame {
        kind: FrameKind::UFrame,
        qos: Qos::SequenceControlled,
        scid: 0,
        vcid: 0,
        seq: Some(300),
        payload: b"seq-300".to_vec(),
    });

    let rx = receiver.receive(&frame);

    assert_eq!(rx.farm, FarmRx::Accepted);
    assert_eq!(rx.delivered_payload, Some(b"seq-300".to_vec()));
    assert_eq!(receiver.farm.v_r, Seq(301));
}

#[test]
fn expedited_bypasses_sequence_window() {
    let mut fop_side = CopP::new(SeqWidth::Mod256);
    fop_side.fop.transmission_window = 0;
    fop_side.send_expedited(vec![1, 2, 3]);
    let tx = fop_side.fop.select_transmit();
    assert!(matches!(tx, Some(FopTx::Expedited { .. })));
}
