//! Interoperability-focused tests for SPDU encoding/decoding and hex exchange.
//!
//! Run:
//! - `cargo test --test interoperability`
//!
//! These tests use fixed expected hex strings so you can compare results across
//! implementations (Rust/C/C++/Python/…).
//!
//! ## How to read failures
//!
//! Each test compares **reference vector** (frozen hex from the interoperability brief /
//! workshop artifacts) against **encoder output** from this crate. On mismatch, the panic
//! message prints both **hex** and **raw byte arrays** side by side.

use ccsds_sc2::{
    DirectivesOrReportsUHF, FixedLengthSPDU, Frame, FrameKind, PLCW16Bit, PLCW32Bit, Qos, SPDU,
    SpduError, Type1Directive, VariableLengthSPDU, Version3Frame, bytes_to_hex, hex_to_bytes,
};
use std::fs;

/// `vector_hex` is canonical lowercase hex (no separators), same style as [`bytes_to_hex`].
fn assert_spdu_bytes_match_vector(test_label: &str, actual: &[u8], vector_hex: &str) {
    let vector_bytes = hex_to_bytes(vector_hex).unwrap_or_else(|e| {
        panic!("{test_label}: invalid reference vector hex `{vector_hex}`: {e}");
    });
    let actual_hex = bytes_to_hex(actual);
    if actual != vector_bytes.as_slice() {
        panic!(
            "{test_label}: encoder output does not match reference test vector.\n\
             • **Reference vector** (hex): `{vector_hex}`\n\
             • **Reference bytes**       : {vector_bytes:02x?}\n\
             • **Encoder output** (hex): `{actual_hex}`\n\
             • **Encoder bytes**        : {actual:02x?}",
            vector_bytes = vector_bytes,
            actual_hex = actual_hex,
            actual = actual,
        );
    }
}

fn json_string_value(json: &str, key: &str) -> String {
    let needle = format!("\"{key}\": \"");
    let start = json
        .find(&needle)
        .unwrap_or_else(|| panic!("missing JSON string field `{key}`"))
        + needle.len();
    let rest = &json[start..];
    let end = rest
        .find('"')
        .unwrap_or_else(|| panic!("unterminated JSON string field `{key}`"));
    rest[..end].to_string()
}

fn assert_spdu_variant_matches_type(spdu: SPDU, expected_type: &str) {
    match (expected_type, spdu) {
        ("F1", SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(_))) => {}
        ("F2", SPDU::FixedLengthSPDU(FixedLengthSPDU::F2(_))) => {}
        ("1", SPDU::VariableLengthSPDU(VariableLengthSPDU::Type1(_))) => {}
        ("2", SPDU::VariableLengthSPDU(VariableLengthSPDU::Type2(_))) => {}
        ("4", SPDU::VariableLengthSPDU(VariableLengthSPDU::Type4(_))) => {}
        ("5", SPDU::VariableLengthSPDU(VariableLengthSPDU::Type5(_))) => {}
        (expected, actual) => panic!("expected SPDU type `{expected}`, decoded {actual:?}"),
    }
}

#[test]
fn interop_fixed_length_f1_known_vector_bytes() {
    // From interoperability.pdf Level 1 examples:
    // Type F1 PLCW: V(R)=42, expedited=3, PCID=1, retransmit=true
    const VECTOR_HEX: &str = "b32a";

    let spdu = SPDU::f1(PLCW16Bit {
        report_value: 42,
        expedited_frame_counter: 3,
        reserved_spare: false,
        pcid: true,
        retransmit_flag: true,
    });

    let bytes = spdu.to_bytes().unwrap();
    assert_spdu_bytes_match_vector(
        "interop_fixed_length_f1_known_vector_bytes (encode vs vector)",
        &bytes,
        VECTOR_HEX,
    );

    let parsed = SPDU::from_bytes(&hex_to_bytes(VECTOR_HEX).unwrap()).unwrap();
    assert_eq!(
        parsed, spdu,
        "decode(reference vector) must reproduce the constructed SPDU"
    );
}

#[test]
fn interop_fixed_length_f2_known_vector_bytes() {
    // From interoperability.pdf Level 1 examples:
    // Type F2 PLCW: V(R)=1234, expedited=5, PCID=0, retransmit=false
    const VECTOR_HEX: &str = "c00504d2";

    let spdu = SPDU::f2(PLCW32Bit {
        report_value: 1234,
        expedited_frame_counter: 5,
        pcid: false,
        retransmit_flag: false,
        reserved_spares: 0,
    });

    let bytes = spdu.to_bytes().unwrap();
    assert_spdu_bytes_match_vector(
        "interop_fixed_length_f2_known_vector_bytes (encode vs vector)",
        &bytes,
        VECTOR_HEX,
    );

    let parsed = SPDU::from_bytes(&hex_to_bytes(VECTOR_HEX).unwrap()).unwrap();
    assert_eq!(
        parsed, spdu,
        "decode(reference vector) must reproduce the constructed SPDU"
    );
}

#[test]
fn interop_variable_length_type1_set_vr_workshop_artifact() {
    // From requirements (FR-9.5 artifact #3) / INTOP-1.3:
    // Variable-Length SPDU Type 1 Directive, SET V(R) with SEQ_CTRL_FSN=42
    const VECTOR_HEX: &str = "02602a";

    let spdu = SPDU::type1(DirectivesOrReportsUHF::single(Type1Directive::set_vr(42)));

    // Expected bytes:
    // header: type_id=0 (Type1), len=2 => 0x02
    // body: SetVR directive word = (0b011<<13) | 0x2A = 0x602A => 60 2A
    let bytes = spdu.to_bytes().unwrap();
    assert_spdu_bytes_match_vector(
        "interop_variable_length_type1_set_vr_workshop_artifact (encode vs vector)",
        &bytes,
        VECTOR_HEX,
    );

    // Parser accepts spaced hex; canonical form must still match VECTOR_HEX after round-trip.
    let parsed = SPDU::from_bytes(&hex_to_bytes("02 60 2a").unwrap()).unwrap();
    assert_eq!(
        parsed, spdu,
        "decode(spaced hex of same vector) must reproduce the constructed SPDU"
    );
    assert_spdu_bytes_match_vector(
        "interop_variable_length_type1_set_vr_workshop_artifact (re-encode vs vector)",
        &parsed.to_bytes().unwrap(),
        VECTOR_HEX,
    );
}

#[test]
fn interop_fixed_length_workshop_artifacts_1_and_2() {
    // Artifact #1: Type F1 PLCW Report Value=127, Retransmit=false, PCID=0, Expedited Counter=3
    const ARTIFACT_1_HEX: &str = "837f";
    let f1 = SPDU::f1(PLCW16Bit {
        report_value: 127,
        expedited_frame_counter: 3,
        reserved_spare: false,
        pcid: false,
        retransmit_flag: false,
    });
    assert_spdu_bytes_match_vector(
        "workshop artifact #1 (F1 PLCW encode vs vector)",
        &f1.to_bytes().unwrap(),
        ARTIFACT_1_HEX,
    );

    // Artifact #2: Type F2 PLCW Report Value=500, Retransmit=true, PCID=1, Expedited Counter=6
    const ARTIFACT_2_HEX: &str = "c01e01f4";
    let f2 = SPDU::f2(PLCW32Bit {
        report_value: 500,
        expedited_frame_counter: 6,
        pcid: true,
        retransmit_flag: true,
        reserved_spares: 0,
    });
    assert_spdu_bytes_match_vector(
        "workshop artifact #2 (F2 PLCW encode vs vector)",
        &f2.to_bytes().unwrap(),
        ARTIFACT_2_HEX,
    );
}

#[test]
fn interop_spdu_validation_rejects_bad_fixed_length_size() {
    // Fixed-length SPDUs must be exactly 2 or 4 octets
    let err = SPDU::from_bytes(&[0x80, 0x00, 0x00]).unwrap_err();
    assert!(matches!(err, SpduError::Invalid(_)));
}

#[test]
fn interop_hex_dump_roundtrip_accepts_separators_and_prefix() {
    const CANONICAL_VECTOR_HEX: &str = "02602a";
    let bytes = hex_to_bytes("0x02 60_2a").unwrap();
    let canonical = bytes_to_hex(&bytes);
    assert_eq!(
        canonical, CANONICAL_VECTOR_HEX,
        "hex parser output (canonical) must match interoperability vector `{CANONICAL_VECTOR_HEX}`; got `{canonical}`"
    );
}

#[test]
fn interop_plcw_generation_matches_spdu_encoding_shape() {
    // PLCW generation: ensure that when a PLCW is constructed (as COP-P would do),
    // its SPDU bytes are byte-identical with the expected workshop artifact.
    const ARTIFACT_1_HEX: &str = "837f";
    let plcw = PLCW16Bit {
        report_value: 127,
        expedited_frame_counter: 3,
        reserved_spare: false,
        pcid: false,
        retransmit_flag: false,
    };
    let spdu = SPDU::f1(plcw);
    assert_spdu_bytes_match_vector(
        "interop_plcw_generation_matches_spdu_encoding_shape (F1 as SPDU vs artifact #1)",
        &spdu.to_bytes().unwrap(),
        ARTIFACT_1_HEX,
    );
}

#[test]
fn interop_frame_v3_pframe_contains_spdu_bytes() {
    // Workshop artifact #4 mentions a Version-3 P-frame containing artifact #1.
    // We validate that the SPDU bytes are carried through the frame payload unchanged.
    const ARTIFACT_1_HEX: &str = "837f";
    let spdu_bytes = hex_to_bytes(ARTIFACT_1_HEX).unwrap();
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
        Frame::V3(v3) => {
            assert_spdu_bytes_match_vector(
                "interop_frame_v3_pframe_contains_spdu_bytes (payload after frame encode/decode vs artifact #1)",
                &v3.payload,
                ARTIFACT_1_HEX,
            );
            assert_eq!(v3.payload, spdu_bytes);
        }
        _ => panic!("expected v3"),
    }
}

#[test]
fn interop_spdu_test_vector_exports_match_metadata() {
    let cases = [
        ("type_f1", "F1", "b32a"),
        ("type_f2", "F2", "c00504d2"),
        ("type_1", "1", "02602a"),
        ("type_2", "2", "1f0102030405060708090a0b0c0d0e0f"),
        ("type_4", "4", "32402a"),
        ("type_5", "5", "42402a"),
    ];

    for (dir, expected_type, expected_hex) in cases {
        let json_path = format!("test-vectors/spdus/{dir}/test_001.json");
        let bin_path = format!("test-vectors/spdus/{dir}/test_001.bin");

        let json = fs::read_to_string(&json_path).expect("vector JSON must be readable");
        assert_eq!(json_string_value(&json, "spdu_type"), expected_type);
        assert_eq!(json_string_value(&json, "spdu_bytes_hex"), expected_hex);

        let payload = hex_to_bytes(expected_hex).expect("expected vector hex must be valid");
        let decoded = SPDU::from_bytes(&payload).expect("vector payload must decode as an SPDU");
        assert_spdu_variant_matches_type(decoded, expected_type);

        let export = fs::read(&bin_path).expect("binary vector export must be readable");
        assert!(
            export.len() >= 64,
            "{bin_path}: binary export must include a 64-byte header"
        );
        assert_eq!(&export[..8], b"CCSDS\0\0\0", "{bin_path}: bad magic");
        assert_eq!(
            u32::from_be_bytes(export[8..12].try_into().unwrap()),
            1,
            "{bin_path}: bad export version"
        );

        let mut padded_type = [0u8; 4];
        let expected_type_bytes = expected_type.as_bytes();
        padded_type[..expected_type_bytes.len()].copy_from_slice(expected_type_bytes);
        assert_eq!(
            &export[12..16],
            padded_type.as_slice(),
            "{bin_path}: header SPDU type must match JSON metadata"
        );
        assert_eq!(
            u32::from_be_bytes(export[16..20].try_into().unwrap()) as usize,
            payload.len(),
            "{bin_path}: header payload length must match SPDU bytes"
        );
        assert_eq!(
            &export[64..],
            payload.as_slice(),
            "{bin_path}: binary payload must match JSON spdu_bytes_hex"
        );
    }
}
