#[derive(Debug, Clone, PartialEq)]
pub struct DirectivesOrReportsUHF {
    pub directives: Vec<Type1Directive>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusReports {
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type1Directive {
    SetTransmitterParameters(SetTransmitterParameters),
    SetControlParameters(SetControlParameters),
    SetReceiverParameters(SetReceiverParameters),
    SetVR(SetVR),
    ReportRequest(ReportRequest),
    SetPLExtensions(SetPLExtensions),
    ReportSourceSCID(ReportSourceSCID),
    Reserved { directive_type: u8, raw_value: u16 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetTransmitterParameters {
    pub tx_mode: u8,          // bits 0-2
    pub tx_data_rate: u8,     // bits 3-6
    pub tx_modulation: bool,  // bit 7
    pub tx_data_encoding: u8, // bits 8-9
    pub tx_frequency: u8,     // bits 10-12
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetControlParameters {
    pub time_sample: u8,           // bits 0-5
    pub duplex: u8,                // bits 6-8
    pub reserved: u8,              // bits 9-10
    pub remote_no_more_data: bool, // bit 11
    pub token: bool,               // bit 12
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetReceiverParameters {
    pub rx_mode: u8,          // bits 0-2
    pub rx_data_rate: u8,     // bits 3-6
    pub rx_modulation: bool,  // bit 7
    pub rx_data_decoding: u8, // bits 8-9
    pub rx_frequency: u8,     // bits 10-12
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetVR {
    pub seq_ctrl_fsn: u8, // bits 0-7
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportRequest {
    pub spare: u8,                 // bits 0-2
    pub status_report_request: u8, // bits 3-7
    pub time_tag_request: u8,      // bits 8-10
    pub pcid0_plcw_request: bool,  // bit 11
    pub pcid1_plcw_request: bool,  // bit 12
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetPLExtensions {
    pub direction: bool,          // bit 0
    pub freq_table: bool,         // bit 1
    pub rate_table: bool,         // bit 2
    pub carrier_mod: u8,          // bits 3-4
    pub data_mod: u8,             // bits 5-6
    pub mode_select: u8,          // bits 7-8
    pub scrambler: u8,            // bits 9-10
    pub diff_mark_encoding: bool, // bit 11
    pub rs_code: bool,            // bit 12
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportSourceSCID {
    pub source_scid: u16, // bits 0-9
}

impl SetVR {
    pub fn from_u16(word: u16) -> Self {
        SetVR {
            seq_ctrl_fsn: (word & 0x00FF) as u8,
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = (self.seq_ctrl_fsn as u16) & 0x00FF;
        word |= 0b011 << 13;
        word
    }
}

impl Type1Directive {
    pub fn from_u16(word: u16) -> Self {
        let directive_type = ((word >> 13) & 0x07) as u8;
        match directive_type {
            0b000 => Type1Directive::SetTransmitterParameters(SetTransmitterParameters::from_u16(word)),
            0b001 => Type1Directive::SetControlParameters(SetControlParameters::from_u16(word)),
            0b010 => Type1Directive::SetReceiverParameters(SetReceiverParameters::from_u16(word)),
            0b011 => Type1Directive::SetVR(SetVR::from_u16(word)),
            0b100 => Type1Directive::ReportRequest(ReportRequest::from_u16(word)),
            0b110 => Type1Directive::SetPLExtensions(SetPLExtensions::from_u16(word)),
            0b111 => Type1Directive::ReportSourceSCID(ReportSourceSCID::from_u16(word)),
            other => Type1Directive::Reserved {
                directive_type: other,
                raw_value: word,
            },
        }
    }

    pub fn to_u16(&self) -> u16 {
        match self {
            Type1Directive::SetTransmitterParameters(d) => d.to_u16(),
            Type1Directive::SetControlParameters(d) => d.to_u16(),
            Type1Directive::SetReceiverParameters(d) => d.to_u16(),
            Type1Directive::SetVR(d) => d.to_u16(),
            Type1Directive::ReportRequest(d) => d.to_u16(),
            Type1Directive::SetPLExtensions(d) => d.to_u16(),
            Type1Directive::ReportSourceSCID(d) => d.to_u16(),
            Type1Directive::Reserved { raw_value, .. } => *raw_value,
        }
    }
}

impl DirectivesOrReportsUHF {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if (data.len() % 2) != 0 {
            return Err("Type 1 SPDU data field length must be a multiple of 2 bytes".to_string());
        }
        let word_count = data.len() / 2;
        if word_count > 7 {
            return Err("Type 1 SPDU data field may contain at most 7 directives".to_string());
        }

        let mut directives = Vec::with_capacity(word_count);
        for i in 0..word_count {
            let word = u16::from_be_bytes([data[2 * i], data[2 * i + 1]]);
            directives.push(Type1Directive::from_u16(word));
        }

        Ok(Self { directives })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        if self.directives.len() > 7 {
            return Err("Type 1 SPDU data field may contain at most 7 directives".to_string());
        }
        let mut out = Vec::with_capacity(self.directives.len() * 2);
        for d in &self.directives {
            out.extend_from_slice(&d.to_u16().to_be_bytes());
        }
        Ok(out)
    }
}

impl SetControlParameters {
    pub fn from_u16(word: u16) -> Self {
        SetControlParameters {
            time_sample: (word & 0x003F) as u8,
            duplex: ((word >> 6) & 0x07) as u8,
            reserved: ((word >> 9) & 0x03) as u8,
            remote_no_more_data: (word & (1 << 11)) != 0,
            token: (word & (1 << 12)) != 0,
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = (self.time_sample as u16) & 0x003F;
        word |= ((self.duplex as u16) & 0x07) << 6;
        if self.remote_no_more_data {
            word |= 1 << 11;
        }
        if self.token {
            word |= 1 << 12;
        }
        word |= 0b001 << 13;
        word
    }
}

impl ReportRequest {
    pub fn from_u16(word: u16) -> Self {
        ReportRequest {
            spare: (word & 0x0007) as u8,
            status_report_request: ((word >> 3) & 0x1F) as u8,
            time_tag_request: ((word >> 8) & 0x07) as u8,
            pcid0_plcw_request: (word & (1 << 11)) != 0,
            pcid1_plcw_request: (word & (1 << 12)) != 0,
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = ((self.status_report_request as u16) & 0x1F) << 3;
        word |= ((self.time_tag_request as u16) & 0x07) << 8;
        if self.pcid0_plcw_request {
            word |= 1 << 11;
        }
        if self.pcid1_plcw_request {
            word |= 1 << 12;
        }
        word |= 0b100 << 13;
        word
    }
}

impl ReportSourceSCID {
    pub fn from_u16(word: u16) -> Self {
        ReportSourceSCID {
            source_scid: word & 0x03FF,
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = self.source_scid & 0x03FF;
        word |= 0b111 << 13;
        word
    }
}

impl SetTransmitterParameters {
    pub fn from_u16(word: u16) -> Self {
        SetTransmitterParameters {
            tx_mode: (word & 0x0007) as u8,
            tx_data_rate: ((word >> 3) & 0x000F) as u8,
            tx_modulation: (word & (1 << 7)) != 0,
            tx_data_encoding: ((word >> 8) & 0x0003) as u8,
            tx_frequency: ((word >> 10) & 0x0007) as u8,
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = 0u16;
        word |= (self.tx_mode as u16) & 0x0007;
        word |= ((self.tx_data_rate as u16) & 0x000F) << 3;
        if self.tx_modulation {
            word |= 1u16 << 7;
        }
        word |= ((self.tx_data_encoding as u16) & 0x0003) << 8;
        word |= ((self.tx_frequency as u16) & 0x0007) << 10;
        word
    }
}

impl SetReceiverParameters {
    pub fn from_u16(word: u16) -> Self {
        SetReceiverParameters {
            rx_mode: (word & 0x0007) as u8,
            rx_data_rate: ((word >> 3) & 0x000F) as u8,
            rx_modulation: (word & (1 << 7)) != 0,
            rx_data_decoding: ((word >> 8) & 0x0003) as u8,
            rx_frequency: ((word >> 10) & 0x0007) as u8,
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = 0u16;
        word |= (self.rx_mode as u16) & 0x0007;
        word |= ((self.rx_data_rate as u16) & 0x000F) << 3;
        if self.rx_modulation {
            word |= 1u16 << 7;
        }
        word |= ((self.rx_data_decoding as u16) & 0x0003) << 8;
        word |= ((self.rx_frequency as u16) & 0x0007) << 10;
        word |= 0b010 << 13;
        word
    }
}

impl SetPLExtensions {
    pub fn from_u16(word: u16) -> Self {
        SetPLExtensions {
            direction: (word & (1 << 0)) != 0,
            freq_table: (word & (1 << 1)) != 0,
            rate_table: (word & (1 << 2)) != 0,
            carrier_mod: ((word >> 3) & 0x03) as u8,
            data_mod: ((word >> 5) & 0x03) as u8,
            mode_select: ((word >> 7) & 0x03) as u8,
            scrambler: ((word >> 9) & 0x03) as u8,
            diff_mark_encoding: (word & (1 << 11)) != 0,
            rs_code: (word & (1 << 12)) != 0,
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = 0u16;
        if self.direction {
            word |= 1 << 0;
        }
        if self.freq_table {
            word |= 1 << 1;
        }
        if self.rate_table {
            word |= 1 << 2;
        }
        word |= ((self.carrier_mod as u16) & 0x03) << 3;
        word |= ((self.data_mod as u16) & 0x03) << 5;
        word |= ((self.mode_select as u16) & 0x03) << 7;
        word |= ((self.scrambler as u16) & 0x03) << 9;
        if self.diff_mark_encoding {
            word |= 1 << 11;
        }
        if self.rs_code {
            word |= 1 << 12;
        }
        word |= 0b110 << 13;
        word
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type1_container_roundtrip_bytes() {
        let container = DirectivesOrReportsUHF {
            directives: vec![
                Type1Directive::SetTransmitterParameters(SetTransmitterParameters {
                    tx_mode: 1,
                    tx_data_rate: 4,
                    tx_modulation: false,
                    tx_data_encoding: 0,
                    tx_frequency: 2,
                }),
                Type1Directive::SetVR(SetVR { seq_ctrl_fsn: 0x5A }),
            ],
        };

        let bytes = container.to_bytes().unwrap();
        let parsed = DirectivesOrReportsUHF::from_bytes(&bytes).unwrap();
        assert_eq!(container, parsed);
    }
}