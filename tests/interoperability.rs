//! Interoperability-focused tests for SPDU encoding/decoding and hex exchange.
//!
//! Run:
//! - `cargo test --test interoperability`
//!
//! These tests use fixed expected hex strings so you can compare results across
//! implementations (Rust/C/C++/Python/…).

use ccsds_sc2::{
    DirectivesOrReportsUHF, FixedLengthSPDU, Frame, FrameKind, PLCW16Bit, PLCW32Bit, Qos, SPDU,
    SetVR, SpduError, Type1Directive, VariableLengthSPDU, Version3Frame, bytes_to_hex,
    hex_to_bytes,
};

#[test]
fn interop_fixed_length_f1_known_vector_bytes() {
    // From interoperability.pdf Level 1 examples:
    // Type F1 PLCW: V(R)=42, expedited=3, PCID=1, retransmit=true
    let spdu = SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(PLCW16Bit {
        report_value: 42,
        expedited_frame_counter: 3,
        reserved_spare: false,
        pcid: true,
        retransmit_flag: true,
    }));

    let bytes = spdu.to_bytes().unwrap();
    assert_eq!(bytes_to_hex(&bytes), "b32a");

    let parsed = SPDU::from_bytes(&hex_to_bytes("b32a").unwrap()).unwrap();
    assert_eq!(parsed, spdu);
}

#[test]
fn interop_fixed_length_f2_known_vector_bytes() {
    // From interoperability.pdf Level 1 examples:
    // Type F2 PLCW: V(R)=1234, expedited=5, PCID=0, retransmit=false
    let spdu = SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(PLCW32Bit {
        report_value: 1234,
        expedited_frame_counter: 5,
        pcid: false,
        retransmit_flag: false,
        lockout_flag: false,
        wait_flag: false,
        reserved_spares: 0,
    }));

    let bytes = spdu.to_bytes().unwrap();
    assert_eq!(bytes_to_hex(&bytes), "c00504d2");

    let parsed = SPDU::from_bytes(&hex_to_bytes("c00504d2").unwrap()).unwrap();
    assert_eq!(parsed, spdu);
}

#[test]
fn interop_variable_length_type1_set_vr_workshop_artifact() {
    // From requirements (FR-9.5 artifact #3):
    // Variable-Length SPDU Type 1 Directive, SET V(R) with SEQ_CTRL_FSN=42
    let spdu = SPDU::VariableLengthSPDU(VariableLengthSPDU::Type1(DirectivesOrReportsUHF {
        directives: vec![Type1Directive::SetVR(SetVR { seq_ctrl_fsn: 42 })],
    }));

    // Expected bytes:
    // header: type_id=0 (Type1), len=2 => 0x02
    // body: SetVR directive word = (0b011<<13) | 0x2A = 0x602A => 60 2A
    let bytes = spdu.to_bytes().unwrap();
    assert_eq!(bytes_to_hex(&bytes), "02602a");

    let parsed = SPDU::from_bytes(&hex_to_bytes("02 60 2a").unwrap()).unwrap();
    assert_eq!(parsed, spdu);
}

#[test]
fn interop_fixed_length_workshop_artifacts_1_and_2() {
    // Artifact #1: Type F1 PLCW Report Value=127, Retransmit=false, PCID=0, Expedited Counter=3
    let f1 = SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(PLCW16Bit {
        report_value: 127,
        expedited_frame_counter: 3,
        reserved_spare: false,
        pcid: false,
        retransmit_flag: false,
    }));
    assert_eq!(bytes_to_hex(&f1.to_bytes().unwrap()), "837f");

    // Artifact #2: Type F2 PLCW Report Value=500, Retransmit=true, PCID=1, Expedited Counter=6
    let f2 = SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(PLCW32Bit {
        report_value: 500,
        expedited_frame_counter: 6,
        pcid: true,
        retransmit_flag: true,
        lockout_flag: false,
        wait_flag: false,
        reserved_spares: 0,
    }));
    assert_eq!(bytes_to_hex(&f2.to_bytes().unwrap()), "c01e01f4");
}

#[test]
fn interop_spdu_validation_rejects_bad_fixed_length_size() {
    // Fixed-length SPDUs must be exactly 2 or 4 octets
    let err = SPDU::from_bytes(&[0x80, 0x00, 0x00]).unwrap_err();
    assert!(matches!(err, SpduError::Invalid(_)));
}

#[test]
fn interop_hex_dump_roundtrip_accepts_separators_and_prefix() {
    let bytes = hex_to_bytes("0x02 60_2a").unwrap();
    assert_eq!(bytes_to_hex(&bytes), "02602a");
}

#[test]
fn interop_plcw_generation_matches_spdu_encoding_shape() {
    // PLCW generation: ensure that when a PLCW is constructed (as COP-P would do),
    // its SPDU bytes are byte-identical with the expected workshop artifact.
    let plcw = PLCW16Bit {
        report_value: 127,
        expedited_frame_counter: 3,
        reserved_spare: false,
        pcid: false,
        retransmit_flag: false,
    };
    let spdu = SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(plcw));
    assert_eq!(bytes_to_hex(&spdu.to_bytes().unwrap()), "837f");
}

#[test]
fn interop_frame_v3_pframe_contains_spdu_bytes() {
    // Workshop artifact #4 mentions a Version-3 P-frame containing artifact #1.
    // We validate that the SPDU bytes are carried through the frame payload unchanged.
    let spdu_bytes = hex_to_bytes("837f").unwrap();
    let frame = Frame::V3(Version3Frame {
        kind: FrameKind::PFrame,
        qos: Qos::Expedited,
        scid: 0,
        vcid: 0,
        seq: None,
        payload: spdu_bytes.clone(),
    });

    let encoded = frame.to_bytes();
    let decoded = Frame::from_bytes(&encoded).unwrap();
    assert_eq!(decoded, frame);

    match decoded {
        Frame::V3(v3) => assert_eq!(v3.payload, spdu_bytes),
        _ => panic!("expected v3"),
    }
}
