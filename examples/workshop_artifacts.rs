//! Hex dumps for CCSDS 235.1 competition workshop interoperability artifacts (deliverables.pdf §3.12).
use ccsds_sc2::{
    DEFAULT_ASM, DirectivesOrReportsUHF, Frame, FrameKind, PLCW16Bit, PLCW32Bit, Qos, SPDU, SetVR,
    Type1Directive, Version3Frame,
};

fn hex_line(label: &str, bytes: &[u8]) {
    let hex: String = bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{label}: {hex}");
}

fn main() {
    // 1 — Type F1 PLCW
    // Builds a typed SPDU value in memory
    let a1 = SPDU::f1(PLCW16Bit {
        report_value: 127,
        expedited_frame_counter: 3,
        reserved_spare: false,
        pcid: false,
        retransmit_flag: false,
    });

    // Turns the built SPDU into raw on-wire octets
    // The unwrap means "if the SPDU is invalid, panic"
    let b1 = a1.to_bytes().unwrap();
    // Prints a human-readable hex dump of the raw on-wire octets
    hex_line("1 F1 PLCW (SPDU wire)", &b1);

    // 2 — Type F2 PLCW (unspecified flags default false / zero)
    let a2 = SPDU::f2(PLCW32Bit {
        report_value: 500,
        expedited_frame_counter: 6,
        pcid: true,
        retransmit_flag: true,
        reserved_spares: 0,
    });
    let b2 = a2.to_bytes().unwrap();
    hex_line("2 F2 PLCW (SPDU wire)", &b2);

    // 3 — Variable-length Type 1, SET V(R), SEQ_CTRL_FSN=42
    let a3 = SPDU::type1(DirectivesOrReportsUHF {
        directives: vec![Type1Directive::SetVR(SetVR { seq_ctrl_fsn: 42 })],
    });
    let b3 = a3.to_bytes().unwrap();
    hex_line("3 Type1 SET V(R) (SPDU wire)", &b3);

    // 4 — P-frame V3, Expedited, payload = artifact #1 octets; SCID/VCID 0 (not specified in brief)
    let f4 = Frame::V3(Version3Frame {
        kind: FrameKind::PFrame,
        qos: Qos::Expedited,
        scid: 0,
        vcid: 0,
        seq: None,
        payload: b1.clone(),
    });
    let frame4 = f4.to_bytes();
    let pltu4 = f4.to_bytes_with_asm(DEFAULT_ASM);
    hex_line("4 P-frame V3 + CRC-16 (transfer frame, no ASM)", &frame4);
    hex_line("4 P-frame V3 PLTU (ASM || frame || CRC)", &pltu4);

    // 5 — U-frame V3, Sequence Controlled, seq=7, payload 0x00..0x09
    let f5 = Frame::V3(Version3Frame {
        kind: FrameKind::UFrame,
        qos: Qos::SequenceControlled,
        scid: 0,
        vcid: 0,
        seq: Some(7),
        payload: (0u8..=9u8).collect(),
    });
    let frame5 = f5.to_bytes();
    let pltu5 = f5.to_bytes_with_asm(DEFAULT_ASM);
    hex_line("5 U-frame V3 + CRC-16 (transfer frame, no ASM)", &frame5);
    hex_line("5 U-frame V3 PLTU (ASM || frame || CRC)", &pltu5);
}
