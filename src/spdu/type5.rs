use super::bits::{BitReader, BitWriter};

#[derive(Debug, Clone, PartialEq)]
pub struct SecondGenLunar {
    pub directives: Vec<Type5Directive>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type5Directive {
    LinkEstablishmentAndControl(Type5LinkEstablishmentAndControl), // 104 bits
    ReportRequest(Type5ReportRequest),                             // 16 bits
    SetVR(Type5SetVR),                                             // 16 bits
    ReportSourceScid(Type5ReportSourceScid),                       // 32 bits
    PnRanging(Type5PnRanging),                                     // 96 bits
    Reserved { directive_name: u8, raw_bits: Vec<u8>, bit_len: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type5LinkEstablishmentAndControl {
    pub link_direction: bool,
    pub directive_function: u8,
    pub rnmd: bool,
    pub token: bool,
    pub duplex_simplex: u8,
    pub polarization: bool,
    pub coherent_noncoherent: bool,
    pub modulation: u8,
    pub modulation_index: u8,
    pub coding: u8,
    pub instantaneous_link_snr: i8,
    pub transceiver_mode: u8,
    pub aos_frame_length: u16,
    pub symbol_rate_raw: u16,
    pub frequency_raw: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type5ReportRequest {
    pub pcid0_plcw_request: bool,
    pub pcid1_plcw_request: bool,
    pub time_tag_sample_request: u8,
    pub status_report_request: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type5SetVR {
    pub seq_ctrl_fsn: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type5ReportSourceScid {
    pub source_scid: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type5PnRanging {
    pub mode_type: u8,
    pub ranging_code: u8,
    pub chip_rate_k: u8,
    pub chip_rate_l: u16,
    pub chip_rate_m: u16,
    pub ranging_mod_index: u8,
    pub pn_epoch_time_tag: [u8; 6],
    pub status_report_request: u8,
}

impl SecondGenLunar {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() > 15 {
            return Err("Type 5 SPDU data field may contain at most 15 octets".to_string());
        }
        let mut r = BitReader::new(data);
        let mut directives = Vec::new();

        while r.remaining_bits() >= 3 {
            let name = r.read_bits_u64(3)? as u8;
            match name {
                0b000 => {
                    if r.remaining_bits() < 101 {
                        return Err("Type 5 LEC directive truncated".to_string());
                    }
                    let _spare1 = r.read_bits_u64(1)?;
                    let link_direction = r.read_bits_u64(1)? != 0;
                    let directive_function = r.read_bits_u64(3)? as u8;
                    let rnmd = r.read_bits_u64(1)? != 0;
                    let token = r.read_bits_u64(1)? != 0;
                    let duplex_simplex = r.read_bits_u64(3)? as u8;
                    let polarization = r.read_bits_u64(1)? != 0;
                    let coherent_noncoherent = r.read_bits_u64(1)? != 0;
                    let _spare2 = r.read_bits_u64(1)?;
                    let modulation = r.read_bits_u64(4)? as u8;
                    let modulation_index = r.read_bits_u64(3)? as u8;
                    let _spare3 = r.read_bits_u64(1)?;
                    let coding = r.read_bits_u64(5)? as u8;
                    let snr_u8 = r.read_bits_u64(8)? as u8;
                    let instantaneous_link_snr = snr_u8 as i8;
                    let transceiver_mode = r.read_bits_u64(3)? as u8;
                    let aos_frame_length = r.read_bits_u64(16)? as u16;
                    let symbol_rate_raw = r.read_bits_u64(16)? as u16;
                    let frequency_raw = r.read_bits_u64(32)? as u32;

                    directives.push(Type5Directive::LinkEstablishmentAndControl(
                        Type5LinkEstablishmentAndControl {
                            link_direction,
                            directive_function,
                            rnmd,
                            token,
                            duplex_simplex,
                            polarization,
                            coherent_noncoherent,
                            modulation,
                            modulation_index,
                            coding,
                            instantaneous_link_snr,
                            transceiver_mode,
                            aos_frame_length,
                            symbol_rate_raw,
                            frequency_raw,
                        },
                    ));
                }
                0b001 => {
                    if r.remaining_bits() < 13 {
                        return Err("Type 5 report request directive truncated".to_string());
                    }
                    directives.push(Type5Directive::ReportRequest(Type5ReportRequest {
                        pcid0_plcw_request: r.read_bits_u64(1)? != 0,
                        pcid1_plcw_request: r.read_bits_u64(1)? != 0,
                        time_tag_sample_request: r.read_bits_u64(6)? as u8,
                        status_report_request: r.read_bits_u64(5)? as u8,
                    }));
                }
                0b010 => {
                    if r.remaining_bits() < 13 {
                        return Err("Type 5 set V(R) directive truncated".to_string());
                    }
                    let _spare = r.read_bits_u64(5)?;
                    let fsn = r.read_bits_u64(8)? as u8;
                    directives.push(Type5Directive::SetVR(Type5SetVR { seq_ctrl_fsn: fsn }));
                }
                0b011 => {
                    if r.remaining_bits() < 29 {
                        return Err("Type 5 report source SCID directive truncated".to_string());
                    }
                    let _reserved = r.read_bits_u64(13)?;
                    let scid = r.read_bits_u64(16)? as u16;
                    directives.push(Type5Directive::ReportSourceScid(Type5ReportSourceScid { source_scid: scid }));
                }
                0b100 => {
                    if r.remaining_bits() < 93 {
                        return Err("Type 5 PN ranging directive truncated".to_string());
                    }
                    let mode_type = r.read_bits_u64(2)? as u8;
                    let ranging_code = r.read_bits_u64(2)? as u8;
                    let chip_rate_k = r.read_bits_u64(3)? as u8;
                    let chip_rate_l = r.read_bits_u64(14)? as u16;
                    let chip_rate_m = r.read_bits_u64(14)? as u16;
                    let ranging_mod_index = r.read_bits_u64(3)? as u8;
                    let epoch_bits = r.read_bits_bytes(48)?;
                    let mut pn_epoch_time_tag = [0u8; 6];
                    pn_epoch_time_tag.copy_from_slice(&epoch_bits[..6]);
                    let status_report_request = r.read_bits_u64(5)? as u8;
                    let _spares = r.read_bits_u64(2)?;
                    directives.push(Type5Directive::PnRanging(Type5PnRanging {
                        mode_type,
                        ranging_code,
                        chip_rate_k,
                        chip_rate_l,
                        chip_rate_m,
                        ranging_mod_index,
                        pn_epoch_time_tag,
                        status_report_request,
                    }));
                }
                other => {
                    let bit_len = r.remaining_bits();
                    let raw_bits = r.read_bits_bytes(bit_len)?;
                    directives.push(Type5Directive::Reserved { directive_name: other, raw_bits, bit_len });
                }
            }
        }

        Ok(Self { directives })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut w = BitWriter::new();
        for d in &self.directives {
            match d {
                Type5Directive::LinkEstablishmentAndControl(x) => {
                    w.write_bits_u64(0b000, 3);
                    w.write_bits_u64(0, 1); // spare bit 3
                    w.write_bits_u64(x.link_direction as u64, 1);
                    w.write_bits_u64((x.directive_function & 0x07) as u64, 3);
                    w.write_bits_u64(x.rnmd as u64, 1);
                    w.write_bits_u64(x.token as u64, 1);
                    w.write_bits_u64((x.duplex_simplex & 0x07) as u64, 3);
                    w.write_bits_u64(x.polarization as u64, 1);
                    w.write_bits_u64(x.coherent_noncoherent as u64, 1);
                    w.write_bits_u64(0, 1); // spare bit 15
                    w.write_bits_u64((x.modulation & 0x0F) as u64, 4);
                    w.write_bits_u64((x.modulation_index & 0x07) as u64, 3);
                    w.write_bits_u64(0, 1); // spare bit 23
                    w.write_bits_u64((x.coding & 0x1F) as u64, 5);
                    w.write_bits_u64((x.instantaneous_link_snr as i8 as u8) as u64, 8);
                    w.write_bits_u64((x.transceiver_mode & 0x07) as u64, 3);
                    w.write_bits_u64(x.aos_frame_length as u64, 16);
                    w.write_bits_u64(x.symbol_rate_raw as u64, 16);
                    w.write_bits_u64(x.frequency_raw as u64, 32);
                }
                Type5Directive::ReportRequest(x) => {
                    w.write_bits_u64(0b001, 3);
                    w.write_bits_u64(x.pcid0_plcw_request as u64, 1);
                    w.write_bits_u64(x.pcid1_plcw_request as u64, 1);
                    w.write_bits_u64((x.time_tag_sample_request & 0x3F) as u64, 6);
                    w.write_bits_u64((x.status_report_request & 0x1F) as u64, 5);
                }
                Type5Directive::SetVR(x) => {
                    w.write_bits_u64(0b010, 3);
                    w.write_bits_u64(0, 5);
                    w.write_bits_u64(x.seq_ctrl_fsn as u64, 8);
                }
                Type5Directive::ReportSourceScid(x) => {
                    w.write_bits_u64(0b011, 3);
                    w.write_bits_u64(0, 13);
                    w.write_bits_u64(x.source_scid as u64, 16);
                }
                Type5Directive::PnRanging(x) => {
                    w.write_bits_u64(0b100, 3);
                    w.write_bits_u64((x.mode_type & 0x03) as u64, 2);
                    w.write_bits_u64((x.ranging_code & 0x03) as u64, 2);
                    w.write_bits_u64((x.chip_rate_k & 0x07) as u64, 3);
                    w.write_bits_u64((x.chip_rate_l & 0x3FFF) as u64, 14);
                    w.write_bits_u64((x.chip_rate_m & 0x3FFF) as u64, 14);
                    w.write_bits_u64((x.ranging_mod_index & 0x07) as u64, 3);
                    w.write_bits_bytes(&x.pn_epoch_time_tag, 48);
                    w.write_bits_u64((x.status_report_request & 0x1F) as u64, 5);
                    w.write_bits_u64(0, 2); // spares
                }
                Type5Directive::Reserved { directive_name, raw_bits, bit_len } => {
                    w.write_bits_u64((*directive_name & 0x07) as u64, 3);
                    w.write_bits_bytes(raw_bits, *bit_len);
                }
            }
        }

        if w.bit_len() > 120 {
            return Err("Type 5 SPDU data field may not exceed 120 bits".to_string());
        }
        Ok(w.into_bytes_padded())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type5_roundtrip_basic_directives() {
        let lunar = SecondGenLunar {
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
        };

        let bytes = lunar.to_bytes().unwrap();
        let parsed = SecondGenLunar::from_bytes(&bytes).unwrap();
        assert_eq!(lunar, parsed);
    }
}

