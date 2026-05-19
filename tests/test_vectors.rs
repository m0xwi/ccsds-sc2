use std::fs;
use std::path::PathBuf;

use ccsds_sc2::{
    DirectivesOrReportsUHF, FirstGenLunar, PLCW16Bit, PLCW32Bit, SPDU, SecondGenLunar,
    TimeDistributionPDU, Type1Directive, Type4Directive, Type4ReportRequest, Type4SetVR,
    Type5Directive, Type5PnRanging, Type5ReportRequest, bytes_to_hex, hex_to_bytes,
};

struct VectorSpec {
    directory: &'static str,
    spdu_type: &'static str,
    timestamp_iso: &'static str,
    timestamp_unix: u64,
    spdu_hex: &'static str,
    spdu: SPDU,
}

fn vector_path(directory: &str, file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-vectors")
        .join("spdus")
        .join(directory)
        .join(file_name)
}

fn assert_json_string_field(json: &str, field: &str, expected: &str) {
    let needle = format!("\"{field}\": \"{expected}\"");
    assert!(
        json.contains(&needle),
        "JSON metadata missing expected field {needle}; document was:\n{json}"
    );
}

fn assert_binary_export(spec: &VectorSpec, payload: &[u8]) {
    let bytes = fs::read(vector_path(spec.directory, "test_001.bin")).unwrap();
    assert!(
        bytes.len() >= 64,
        "{} binary export must include a 64-byte header",
        spec.directory
    );

    assert_eq!(&bytes[0..8], b"CCSDS\0\0\0");
    assert_eq!(u32::from_be_bytes(bytes[8..12].try_into().unwrap()), 1);

    let mut expected_type = [0u8; 4];
    let type_bytes = spec.spdu_type.as_bytes();
    expected_type[..type_bytes.len()].copy_from_slice(type_bytes);
    assert_eq!(&bytes[12..16], &expected_type);

    assert_eq!(
        u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as usize,
        payload.len()
    );
    assert_eq!(
        u64::from_be_bytes(bytes[20..28].try_into().unwrap()),
        spec.timestamp_unix
    );
    assert!(bytes[28..64].iter().all(|b| *b == 0));
    assert_eq!(&bytes[64..], payload);
}

fn vector_specs() -> Vec<VectorSpec> {
    vec![
        VectorSpec {
            directory: "type_f1",
            spdu_type: "F1",
            timestamp_iso: "2026-03-07T12:00:00Z",
            timestamp_unix: 1_772_884_800,
            spdu_hex: "b32a",
            spdu: SPDU::f1(PLCW16Bit {
                report_value: 42,
                expedited_frame_counter: 3,
                reserved_spare: false,
                pcid: true,
                retransmit_flag: true,
            }),
        },
        VectorSpec {
            directory: "type_f2",
            spdu_type: "F2",
            timestamp_iso: "2026-03-07T12:00:00Z",
            timestamp_unix: 1_772_884_800,
            spdu_hex: "c00504d2",
            spdu: SPDU::f2(PLCW32Bit {
                report_value: 1234,
                expedited_frame_counter: 5,
                pcid: false,
                retransmit_flag: false,
                reserved_spares: 0,
            }),
        },
        VectorSpec {
            directory: "type_1",
            spdu_type: "1",
            timestamp_iso: "2026-04-24T12:00:00Z",
            timestamp_unix: 1_777_032_000,
            spdu_hex: "02602a",
            spdu: SPDU::type1(DirectivesOrReportsUHF::single(Type1Directive::set_vr(42))),
        },
        VectorSpec {
            directory: "type_2",
            spdu_type: "2",
            timestamp_iso: "2026-04-24T12:00:00Z",
            timestamp_unix: 1_777_032_000,
            spdu_hex: "1f010102030405060708090a0b0c0d0e",
            spdu: SPDU::type2(TimeDistributionPDU {
                directive_type: 1,
                transceiver_clock: [1, 2, 3, 4, 5, 6, 7, 8],
                send_side_delay: [9, 10, 11],
                one_way_light_time: [12, 13, 14],
            }),
        },
        VectorSpec {
            directory: "type_4",
            spdu_type: "4",
            timestamp_iso: "2026-04-24T12:00:00Z",
            timestamp_unix: 1_777_032_000,
            spdu_hex: "343243405a",
            spdu: SPDU::type4(FirstGenLunar {
                directives: vec![
                    Type4Directive::ReportRequest(Type4ReportRequest {
                        pcid0_plcw_request: true,
                        pcid1_plcw_request: false,
                        time_tag_sample_request: 0x12,
                        status_report_request: 0x03,
                    }),
                    Type4Directive::SetVR(Type4SetVR { seq_ctrl_fsn: 0x5A }),
                ],
            }),
        },
        VectorSpec {
            directory: "type_5",
            spdu_type: "5",
            timestamp_iso: "2026-04-24T12:00:00Z",
            timestamp_unix: 1_777_032_000,
            spdu_hex: "4e9192348d160081018202830428e0",
            spdu: SPDU::type5(SecondGenLunar {
                directives: vec![
                    Type5Directive::PnRanging(Type5PnRanging {
                        mode_type: 2,
                        ranging_code: 0,
                        chip_rate_k: 6,
                        chip_rate_l: 0x1234 & 0x3FFF,
                        chip_rate_m: 0x2345 & 0x3FFF,
                        ranging_mod_index: 4,
                        pn_epoch_time_tag: [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
                        status_report_request: 1,
                    }),
                    Type5Directive::ReportRequest(Type5ReportRequest {
                        pcid0_plcw_request: false,
                        pcid1_plcw_request: true,
                        time_tag_sample_request: 7,
                        status_report_request: 0,
                    }),
                ],
            }),
        },
    ]
}

#[test]
fn spdu_test_vectors_match_json_binary_and_encoder() {
    for spec in vector_specs() {
        let json = fs::read_to_string(vector_path(spec.directory, "test_001.json")).unwrap();
        assert_json_string_field(&json, "timestamp", spec.timestamp_iso);
        assert_json_string_field(&json, "spdu_type", spec.spdu_type);
        assert_json_string_field(&json, "spdu_bytes_hex", spec.spdu_hex);

        let payload = hex_to_bytes(spec.spdu_hex).unwrap();
        assert_eq!(bytes_to_hex(&payload), spec.spdu_hex);
        assert_eq!(spec.spdu.to_bytes().unwrap(), payload);
        assert_eq!(SPDU::from_bytes(&payload).unwrap(), spec.spdu);

        assert_binary_export(&spec, &payload);
    }
}
