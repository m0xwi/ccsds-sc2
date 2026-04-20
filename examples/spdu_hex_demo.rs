//! Small SPDU + hex-dump demonstration.
//!
//! Run:
//! - `cargo run --example spdu_hex_demo`

use ccsds_sc2::{FixedLengthSPDU, PLCW16Bit, SPDU, bytes_to_hex, hex_to_bytes};

fn main() {
    // Type F1 (16-bit PLCW) example:
    // V(R)=42, expedited=3, PCID=1, retransmit=true
    let spdu = SPDU::FixedLengthSPDU(FixedLengthSPDU::F1(PLCW16Bit {
        report_value: 42,
        expedited_frame_counter: 3,
        reserved_spare: false,
        pcid: true,
        retransmit_flag: true,
    }));

    let bytes = spdu.to_bytes().expect("encode SPDU");
    println!("SPDU bytes (hex): {}", bytes_to_hex(&bytes));

    // Demonstrate forgiving hex parsing (prefix + separators).
    let parsed_bytes = hex_to_bytes("0xb3 2a").expect("parse hex");
    let parsed_spdu = SPDU::from_bytes(&parsed_bytes).expect("decode SPDU");
    println!("Parsed equals original: {}", parsed_spdu == spdu);
}
