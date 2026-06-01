//! Integration tests for the COP-P layer (Gateway 2).

use ccsds_sc2::{
    CopP, CopTx, DirectivesOrReportsUHF, FarmRx, FirstGenLunar, FopTx, Frame, FrameKind, Qos, SPDU,
    SecondGenLunar, Seq, SeqWidth, Type1Directive, Type4Directive, Type4SetVR, Type5Directive,
    Type5SetVR, VariableLengthSPDU, Version3Frame,
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
fn directive_pframe_does_not_poison_fop_plcw_state() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.fop.v_v_s = Seq(7);
    node.fop.nn_r = Seq(3);

    let directive = SPDU::type1(DirectivesOrReportsUHF::single(Type1Directive::set_vr(42)));
    let pframe = CopP::build_pframe(&directive).unwrap();

    let rx = node.receive(&pframe);

    assert_eq!(rx.farm, FarmRx::Accepted);
    assert_eq!(node.farm.v_r, Seq(42));
    assert_eq!(node.fop.synch_timer, 0);
    assert_eq!(node.fop.v_v_s, Seq(7));
}

#[test]
fn type4_and_type5_set_vr_pframes_resynchronize_receiver() {
    let mut node = CopP::new(SeqWidth::Mod256);

    let type4 = SPDU::type4(FirstGenLunar {
        directives: vec![Type4Directive::SetVR(Type4SetVR { seq_ctrl_fsn: 17 })],
    });
    let type4_frame = CopP::build_pframe(&type4).unwrap();
    assert_eq!(node.receive(&type4_frame).farm, FarmRx::Accepted);
    assert_eq!(node.farm.v_r, Seq(17));

    let type5 = SPDU::type5(SecondGenLunar {
        directives: vec![Type5Directive::SetVR(Type5SetVR { seq_ctrl_fsn: 99 })],
    });
    let type5_frame = CopP::build_pframe(&type5).unwrap();
    assert_eq!(node.receive(&type5_frame).farm, FarmRx::Accepted);
    assert_eq!(node.farm.v_r, Seq(99));
}

#[test]
fn mod65536_receive_uses_full_sequence_number() {
    let mut node = CopP::new(SeqWidth::Mod65536);
    node.farm.v_r = Seq(0x0100);

    let frame = Frame::V3(Version3Frame {
        kind: FrameKind::UFrame,
        qos: Qos::SequenceControlled,
        scid: 0,
        vcid: 0,
        seq: Some(0x0100),
        payload: vec![1, 2, 3],
    });

    let rx = node.receive(&frame);

    assert_eq!(rx.farm, FarmRx::Accepted);
    assert_eq!(rx.delivered_payload, Some(vec![1, 2, 3]));
    assert_eq!(node.farm.v_r, Seq(0x0101));
}

#[test]
fn resync_transmit_selects_set_vr_directive() {
    let mut node = CopP::new(SeqWidth::Mod256);
    node.fop.need_plcw = false;
    node.fop.nn_r = Seq(55);
    node.start_resync();

    let tx = node.select_transmit().expect("resync should emit SET V(R)");
    let CopTx::Plcw(spdu) = tx else {
        panic!("resync should emit a P-frame SPDU");
    };

    match spdu {
        SPDU::VariableLengthSPDU(VariableLengthSPDU::Type1(body)) => {
            assert_eq!(body.directives, vec![Type1Directive::set_vr(55)],);
        }
        other => panic!("expected Type 1 SET V(R), got {other:?}"),
    }
}

#[test]
fn expedited_bypasses_sequence_window() {
    let mut fop_side = CopP::new(SeqWidth::Mod256);
    fop_side.fop.transmission_window = 0;
    fop_side.send_expedited(vec![1, 2, 3]);
    let tx = fop_side.fop.select_transmit();
    assert!(matches!(tx, Some(FopTx::Expedited { .. })));
}
