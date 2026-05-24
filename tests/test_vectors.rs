use std::fs;
use std::path::{Path, PathBuf};

use ccsds_sc2::{
    DirectivesOrReportsUHF, PLCW16Bit, PLCW32Bit, SPDU, Type1Directive, bytes_to_hex, hex_to_bytes,
};
use serde::Deserialize;

// defines the required JSON keys for the test vectors
// This is a schema wrapper that represents the JSON file for a single test vector.
// It is used to deserialize the JSON file into a Rust struct.

#[derive(Debug, Deserialize)]
struct SpduVector {
    spdu_type: String,
    spdu_id: String,
    spdu_bytes_hex: String,
    #[serde(default)]
    fields: serde_json::Value,
}

// recursively collect all json files in the test-vectors directory
fn collect_json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for ent in entries.flatten() {
        let p = ent.path();
        if p.is_dir() {
            out.extend(collect_json_files(&p));
        } else if p.extension().and_then(|s| s.to_str()) == Some("json") {
            out.push(p);
        }
    }
    out.sort();
    out
}

// extract a required boolean field from a JSON value and panic if it is missing or invalid
fn must_bool(v: &serde_json::Value, key: &str) -> bool {
    v.get(key)
        .and_then(|x| x.as_bool())
        .unwrap_or_else(|| panic!("missing/invalid bool field `{key}`"))
}

// extract a required integer field from a JSON value and panic if it is missing or invalid
fn must_u64(v: &serde_json::Value, key: &str) -> u64 {
    v.get(key)
        .and_then(|x| x.as_u64())
        .unwrap_or_else(|| panic!("missing/invalid integer field `{key}`"))
}

// Read the JSON file, and parse it into the SPDUVector schema wrapper struct.
fn load_vector(path: &Path) -> SpduVector {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read vector `{}`: {e}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("failed to parse json `{}`: {e}", path.display()))
}

// Compare actual byte arrays with expected hex strings.
// If they don't match, panic with a helpful message.
fn assert_hex_bytes_equal(label: &str, actual: &[u8], expected_hex: &str) {
    let expected = hex_to_bytes(expected_hex)
        .unwrap_or_else(|e| panic!("{label}: invalid expected hex `{expected_hex}`: {e}"));
    // checks byte-for-byte equality
    if actual != expected.as_slice() {
        panic!(
            // the panic message is a helpful message that includes the label, the expected hex string, the expected bytes, the actual hex string, and the actual bytes
            "{label}: bytes mismatch\n\
             - expected_hex: `{expected_hex}`\n\
             - expected_bytes: {expected:02x?}\n\
             - actual_hex: `{}`\n\
             - actual_bytes: {actual:02x?}",
            bytes_to_hex(actual),
            expected = expected,
            actual = actual,
        );
    }
}

// Construct a typed SPDU value from the vector's fields section.
// If the vector is not a valid SPDU, return None.

fn vector_to_spdu(v: &SpduVector, source: &Path) -> Option<SPDU> {
    match v.spdu_type.as_str() {
        "F1" => {
            let f = &v.fields;
            Some(SPDU::f1(PLCW16Bit {
                report_value: must_u64(f, "report_value") as u8,
                expedited_frame_counter: must_u64(f, "expedited_counter") as u8,
                reserved_spare: false,
                pcid: must_bool(f, "pcid"),
                retransmit_flag: must_bool(f, "retransmit_flag"),
            }))
        }
        "F2" => {
            let f = &v.fields;
            Some(SPDU::f2(PLCW32Bit {
                report_value: must_u64(f, "report_value") as u16,
                expedited_frame_counter: must_u64(f, "expedited_counter") as u8,
                pcid: must_bool(f, "pcid"),
                retransmit_flag: must_bool(f, "retransmit_flag"),
                reserved_spares: 0,
            }))
        }
        "1" => {
            let f = &v.fields;
            let directives = f
                .get("directives")
                .and_then(|x| x.as_array())
                .unwrap_or_else(|| {
                    panic!(
                        "vector `{}` missing/invalid `fields.directives` array",
                        source.display()
                    )
                });

            // Keep this strict: current vectors are single-directive SET_VR.
            if directives.len() != 1 {
                return None;
            }
            let d0 = &directives[0];
            let name = d0.get("directive").and_then(|x| x.as_str()).unwrap_or("");
            if name != "SET_VR" {
                return None;
            }
            let fsn = d0.get("seq_ctrl_fsn").and_then(|x| x.as_u64()).unwrap_or(0);
            Some(SPDU::type1(DirectivesOrReportsUHF::single(
                Type1Directive::set_vr(fsn as u8),
            )))
        }
        _ => None,
    }
}

#[test]
fn json_test_vectors_roundtrip_and_match_wire_bytes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-vectors");
    let files = collect_json_files(&root);
    assert!(
        !files.is_empty(),
        "no json vectors found under `{}`",
        root.display()
    );

    for path in files {
        let v = load_vector(&path);
        let label = format!("{} ({}/{})", path.display(), v.spdu_type, v.spdu_id);

        // Always enforce: bytes decode and re-encode is stable for the expected wire bytes.
        let expected_bytes = hex_to_bytes(&v.spdu_bytes_hex).unwrap_or_else(|e| {
            panic!("{label}: invalid `spdu_bytes_hex` `{}`: {e}", v.spdu_bytes_hex)
        });
        let decoded = SPDU::from_bytes(&expected_bytes)
            .unwrap_or_else(|e| panic!("{label}: SPDU::from_bytes failed: {e}"));
        let reencoded = decoded
            .to_bytes()
            .unwrap_or_else(|e| panic!("{label}: SPDU::to_bytes failed after decode: {e}"));
        assert_hex_bytes_equal(
            &format!("{label}: decode->encode stability"),
            &reencoded,
            &v.spdu_bytes_hex,
        );

        // When we know how to build a typed SPDU from fields, also check encode and structured round-trip.
        if let Some(constructed) = vector_to_spdu(&v, &path) {
            let encoded = constructed
                .to_bytes()
                .unwrap_or_else(|e| panic!("{label}: constructed SPDU failed to encode: {e}"));
            assert_hex_bytes_equal(
                &format!("{label}: constructed->encode matches vector"),
                &encoded,
                &v.spdu_bytes_hex,
            );

            let parsed = SPDU::from_bytes(&encoded)
                .unwrap_or_else(|e| panic!("{label}: failed to decode encoded bytes: {e}"));
            assert_eq!(
                parsed, constructed,
                "{label}: encode->decode must reproduce the constructed SPDU"
            );
        }
    }
}

