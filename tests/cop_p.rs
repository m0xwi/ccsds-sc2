//! Integration tests for the COP-P layer (Gateway 2).

use ccsds_sc2::{
    CopP, CopTx, FarmRx, FopState, FopTx, Frame, FrameKind, Qos, Seq, SeqWidth, Version3Frame,
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
fn local_resync_sends_set_vr_and_completes_after_peer_plcw() {
    let mut sender = CopP::new(SeqWidth::Mod256);
    let mut receiver = CopP::new(SeqWidth::Mod256);
    sender.fop.nn_r = Seq(7);
    sender.fop.v_s = Seq(7);
    sender.fop.v_v_s = Seq(7);

    sender.start_resync();
    let set_vr_spdu = match sender.select_transmit() {
        Some(CopTx::Plcw(spdu)) => spdu,
        other => panic!("expected SET V(R) P-frame, got {other:?}"),
    };

    let set_vr_frame = CopP::build_pframe(&set_vr_spdu).unwrap();
    let rx = receiver.receive(&set_vr_frame);
    assert_eq!(rx.farm, FarmRx::Accepted);
    assert_eq!(receiver.farm.v_r, Seq(7));
    assert_eq!(receiver.fop.synch_timer, 0);

    let ack_spdu = match receiver.select_transmit() {
        Some(CopTx::Plcw(spdu)) => spdu,
        other => panic!("expected peer PLCW after SET V(R), got {other:?}"),
    };
    let ack_frame = CopP::build_pframe(&ack_spdu).unwrap();
    sender.receive(&ack_frame);

    assert_eq!(sender.fop.state, FopState::Active);
    assert!(!sender.fop.resync);
}

#[test]
fn mod65536_receive_uses_full_sequence_number() {
    let mut receiver = CopP::new(SeqWidth::Mod65536).with_f2_plcw(true);
    receiver.farm.v_r = Seq(300);
    let frame = Frame::V3(Version3Frame {
        kind: FrameKind::UFrame,
        qos: Qos::SequenceControlled,
        scid: 0,
        vcid: 0,
        seq: Some(300),
        payload: vec![0xAA],
    });

    let rx = receiver.receive(&frame);

    assert_eq!(rx.farm, FarmRx::Accepted);
    assert_eq!(rx.delivered_payload, Some(vec![0xAA]));
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
