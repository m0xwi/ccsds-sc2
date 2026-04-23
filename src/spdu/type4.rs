// [MermaidChart: 56e34539-ff3d-4d19-8654-87fe5bd1f28f]

use super::bits::{BitReader, BitWriter};

#[derive(Debug, Clone, PartialEq)]
pub struct FirstGenLunar {
    pub directives: Vec<Type4Directive>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type4Directive {
    LinkEstablishmentAndControl(Type4LinkEstablishmentAndControl), // 56 bits
    ReportRequest(Type4ReportRequest),                             // 16 bits
    SetVR(Type4SetVR),                                             // 16 bits
    ReportSourceScid(Type4ReportSourceScid),                       // 32 bits
    Reserved {
        directive_name: u8,
        raw_bits: Vec<u8>,
        bit_len: usize,
    },
}

// The following per-directive structs define the data fields for each directive type.
#[derive(Debug, Clone, PartialEq)]
pub struct Type4LinkEstablishmentAndControl {
    pub link_direction: bool,
    pub demand_query: bool,
    pub query_response: bool,
    pub rnmd: bool,
    pub token: bool,
    pub duplex_simplex: u8,
    pub frequency: u8,
    pub polarization: bool,
    pub modulation_index: u8,
    pub modulation: u8,
    pub spares: u8,
    pub coding: u8,
    pub instantaneous_link_snr: u8,
    pub symbol_rate_raw: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type4ReportRequest {
    pub pcid0_plcw_request: bool,
    pub pcid1_plcw_request: bool,
    pub time_tag_sample_request: u8,
    pub status_report_request: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type4SetVR {
    pub seq_ctrl_fsn: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type4ReportSourceScid {
    pub source_scid: u16,
}

// The following impl block define the bit-field implementations for the FirstGenLunar SPDU.

impl FirstGenLunar {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() > 15 {
            return Err("Type 4 SPDU data field may contain at most 15 octets".to_string());
        }
        let mut r = BitReader::new(data);
        let mut directives = Vec::new();

        while r.remaining_bits() >= 3 {
            let name = r.read_bits_u64(3)? as u8;
            match name {
                0b000 => {
                    if r.remaining_bits() < 53 {
                        return Err("Type 4 LEC directive truncated".to_string());
                    }
                    let d = Type4LinkEstablishmentAndControl {
                        link_direction: r.read_bits_u64(1)? != 0,
                        demand_query: r.read_bits_u64(1)? != 0,
                        query_response: r.read_bits_u64(1)? != 0,
                        rnmd: r.read_bits_u64(1)? != 0,
                        token: r.read_bits_u64(1)? != 0,
                        duplex_simplex: r.read_bits_u64(3)? as u8,
                        frequency: r.read_bits_u64(5)? as u8,
                        polarization: r.read_bits_u64(1)? != 0,
                        modulation_index: r.read_bits_u64(3)? as u8,
                        modulation: r.read_bits_u64(4)? as u8,
                        spares: r.read_bits_u64(2)? as u8,
                        coding: r.read_bits_u64(6)? as u8,
                        instantaneous_link_snr: r.read_bits_u64(8)? as u8,
                        symbol_rate_raw: r.read_bits_u64(16)? as u16,
                    };
                    directives.push(Type4Directive::LinkEstablishmentAndControl(d));
                }
                0b001 => {
                    if r.remaining_bits() < 13 {
                        return Err("Type 4 report request directive truncated".to_string());
                    }
                    let d = Type4ReportRequest {
                        pcid0_plcw_request: r.read_bits_u64(1)? != 0,
                        pcid1_plcw_request: r.read_bits_u64(1)? != 0,
                        time_tag_sample_request: r.read_bits_u64(6)? as u8,
                        status_report_request: r.read_bits_u64(5)? as u8,
                    };
                    directives.push(Type4Directive::ReportRequest(d));
                }
                0b010 => {
                    if r.remaining_bits() < 13 {
                        return Err("Type 4 set V(R) directive truncated".to_string());
                    }
                    let _spare = r.read_bits_u64(5)?;
                    let fsn = r.read_bits_u64(8)? as u8;
                    directives.push(Type4Directive::SetVR(Type4SetVR { seq_ctrl_fsn: fsn }));
                }
                0b011 => {
                    if r.remaining_bits() < 29 {
                        return Err("Type 4 report source SCID directive truncated".to_string());
                    }
                    let _reserved = r.read_bits_u64(13)?;
                    let scid = r.read_bits_u64(16)? as u16;
                    directives.push(Type4Directive::ReportSourceScid(Type4ReportSourceScid {
                        source_scid: scid,
                    }));
                }
                other => {
                    let bit_len = r.remaining_bits();
                    let raw_bits = r.read_bits_bytes(bit_len)?;
                    directives.push(Type4Directive::Reserved {
                        directive_name: other,
                        raw_bits,
                        bit_len,
                    });
                }
            }
        }

        Ok(Self { directives })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut w = BitWriter::new();
        for d in &self.directives {
            match d {
                Type4Directive::LinkEstablishmentAndControl(x) => {
                    w.write_bits_u64(0b000, 3);
                    w.write_bits_u64(x.link_direction as u64, 1);
                    w.write_bits_u64(x.demand_query as u64, 1);
                    w.write_bits_u64(x.query_response as u64, 1);
                    w.write_bits_u64(x.rnmd as u64, 1);
                    w.write_bits_u64(x.token as u64, 1);
                    w.write_bits_u64((x.duplex_simplex & 0x07) as u64, 3);
                    w.write_bits_u64((x.frequency & 0x1F) as u64, 5);
                    w.write_bits_u64(x.polarization as u64, 1);
                    w.write_bits_u64((x.modulation_index & 0x07) as u64, 3);
                    w.write_bits_u64((x.modulation & 0x0F) as u64, 4);
                    w.write_bits_u64((x.spares & 0x03) as u64, 2);
                    w.write_bits_u64((x.coding & 0x3F) as u64, 6);
                    w.write_bits_u64(x.instantaneous_link_snr as u64, 8);
                    w.write_bits_u64(x.symbol_rate_raw as u64, 16);
                }
                Type4Directive::ReportRequest(x) => {
                    w.write_bits_u64(0b001, 3);
                    w.write_bits_u64(x.pcid0_plcw_request as u64, 1);
                    w.write_bits_u64(x.pcid1_plcw_request as u64, 1);
                    w.write_bits_u64((x.time_tag_sample_request & 0x3F) as u64, 6);
                    w.write_bits_u64((x.status_report_request & 0x1F) as u64, 5);
                }
                Type4Directive::SetVR(x) => {
                    w.write_bits_u64(0b010, 3);
                    w.write_bits_u64(0, 5);
                    w.write_bits_u64(x.seq_ctrl_fsn as u64, 8);
                }
                Type4Directive::ReportSourceScid(x) => {
                    w.write_bits_u64(0b011, 3);
                    w.write_bits_u64(0, 13);
                    w.write_bits_u64(x.source_scid as u64, 16);
                }
                Type4Directive::Reserved {
                    directive_name,
                    raw_bits,
                    bit_len,
                } => {
                    w.write_bits_u64((*directive_name & 0x07) as u64, 3);
                    w.write_bits_bytes(raw_bits, *bit_len);
                }
            }
        }

        if w.bit_len() > 120 {
            return Err("Type 4 SPDU data field may not exceed 120 bits".to_string());
        }
        Ok(w.into_bytes_padded())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type4_roundtrip_basic_directives() {
        let lunar = FirstGenLunar {
            directives: vec![
                Type4Directive::ReportRequest(Type4ReportRequest {
                    pcid0_plcw_request: true,
                    pcid1_plcw_request: false,
                    time_tag_sample_request: 0x12,
                    status_report_request: 0x03,
                }),
                Type4Directive::SetVR(Type4SetVR { seq_ctrl_fsn: 0x5A }),
            ],
        };

        let bytes = lunar.to_bytes().unwrap();
        let parsed = FirstGenLunar::from_bytes(&bytes).unwrap();
        assert_eq!(lunar, parsed);
    }
}
