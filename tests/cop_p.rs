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
fn set_vr_pframe_does_not_corrupt_sender_state() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.fop.nn_r = Seq(3);
    node.fop.v_v_s = Seq(7);

    let set_vr = SPDU::type1(DirectivesOrReportsUHF::single(Type1Directive::set_vr(100)));
    let frame = CopP::build_pframe(&set_vr).unwrap();
    let result = node.receive(&frame);

    assert_eq!(result.farm, FarmRx::Accepted);
    assert_eq!(node.farm.v_r, Seq(100));
    assert_eq!(node.fop.v_v_s, Seq(7));
    assert_eq!(node.fop.synch_timer, 0);
}

#[test]
fn start_resync_transmits_set_vr_before_plcw() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.fop.nn_r = Seq(42);
    node.start_resync();

    let Some(CopTx::Plcw(spdu)) = node.select_transmit() else {
        panic!("resync should emit SET V(R) as a P-frame SPDU");
    };

    assert_eq!(node.fop.state, FopState::Resync);
    assert_eq!(spdu, node.build_set_vr_spdu());
    assert!(!node.fop.need_plcw);
}

#[test]
fn partial_ack_during_retransmit_restarts_from_new_ack_point() {
    let mut fop_side = CopP::new(SeqWidth::Mod256);
    for payload in 0..5 {
        fop_side.send_sequence_controlled(vec![payload]);
    }
    for expected in 0..5 {
        match fop_side.fop.select_transmit() {
            Some(FopTx::SeqNew { seq, .. }) => assert_eq!(seq, Seq(expected)),
            other => panic!("expected new sequence frame {expected}, got {other:?}"),
        }
    }

    fop_side.fop.n_r = Seq(0);
    fop_side.fop.r_r = true;
    fop_side.fop.on_valid_plcw();
    for expected in 0..4 {
        match fop_side.fop.select_transmit() {
            Some(FopTx::SeqResend { seq, .. }) => assert_eq!(seq, Seq(expected)),
            other => panic!("expected retransmit {expected}, got {other:?}"),
        }
    }

    fop_side.fop.n_r = Seq(2);
    fop_side.fop.r_r = false;
    assert!(fop_side.fop.is_valid_plcw(Seq(2), false));
    fop_side.fop.on_valid_plcw();

    match fop_side.fop.select_transmit() {
        Some(FopTx::SeqResend { seq, payload }) => {
            assert_eq!(seq, Seq(2));
            assert_eq!(payload, vec![2]);
        }
        other => panic!("expected retransmission to restart at NN(R), got {other:?}"),
    }
}

#[test]
fn mod65536_sequence_frames_keep_high_byte() {
    let mut receiver = CopP::new(SeqWidth::Mod65536);
    receiver.farm.on_set_vr(Seq(300));

    let frame = Frame::V3(Version3Frame {
        kind: FrameKind::UFrame,
        qos: Qos::SequenceControlled,
        scid: 0,
        vcid: 0,
        seq: Some(300),
        payload: vec![1, 2, 3],
    });

    let result = receiver.receive(&frame);
    assert_eq!(result.farm, FarmRx::Accepted);
    assert_eq!(result.delivered_payload, Some(vec![1, 2, 3]));
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
