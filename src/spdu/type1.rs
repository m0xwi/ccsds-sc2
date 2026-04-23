// The container-level logic for the whole Type-1 SPDU is DirectivesOrReportUHF.
// It turns a Type-1 body into structured directives and back.

// The Type1Directive is the dispatch layer between the raw 16-bit words and specific directive structs.
// Its role is to decode/encode the directive tag and route it to the right fornat.

// Each <DirectiveStruct> blocks has the bitfield mapping for its specific layout:
// It is the exact wire layout for each directive type.

#[derive(Debug, Clone, PartialEq)]
pub struct DirectivesOrReportsUHF {
    pub directives: Vec<Type1Directive>,
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

// The following per-directive structs define the data fields for each directive type.
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

// The following impl blocks define the bit-field implementations for each directive type.

// The container for the Type 1 SPDU directives.
impl DirectivesOrReportsUHF {
    pub fn validate(&self) -> Result<(), String> {
        if self.directives.len() > 7 {
            return Err("Type 1 SPDU data field may contain at most 7 directives".to_string());
        }
        for d in &self.directives {
            d.validate()?;
        }
        Ok(())
    }

    // The from_bytes function returns a Result which is an enumeration that can be in one of two possible states: Ok or Err.

    // The from_bytes function validates the body is a multiple of 2 bytes (because each directive is 16 bits).
    // The from_bytes function validates the body is no more than 7 directives.
    // It reads each pair of bytes as a 16-bit word, and then converts that word to a Type1Directive.
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

    // The to_bytes function converts the Type1Directive objects to a vector of bytes.
    // The to_bytes function validates the body is no more than 7 directives.
    // It converts each Type1Directive object to a 16-bit word, and then converts that word to a vector of bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut out = Vec::with_capacity(self.directives.len() * 2);
        for d in &self.directives {
            out.extend_from_slice(&d.to_u16().to_be_bytes());
        }
        Ok(out)
    }
}

// The Type1Directive identifies which 16-bit directive/report you have.
// The variants correspond to the spec's Directive Type field (bits 13-15)
impl Type1Directive {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Type1Directive::SetTransmitterParameters(d) => d.validate(),
            Type1Directive::SetControlParameters(d) => d.validate(),
            Type1Directive::SetReceiverParameters(d) => d.validate(),
            Type1Directive::SetVR(d) => d.validate(),
            Type1Directive::ReportRequest(d) => d.validate(),
            Type1Directive::SetPLExtensions(d) => d.validate(),
            Type1Directive::ReportSourceSCID(d) => d.validate(),
            Type1Directive::Reserved { .. } => Ok(()),
        }
    }

    pub fn from_u16(word: u16) -> Self {
        let directive_type = ((word >> 13) & 0x07) as u8;
        match directive_type {
            0b000 => {
                Type1Directive::SetTransmitterParameters(SetTransmitterParameters::from_u16(word))
            }
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

    // The to_u16 function converts the Type1Directive object to a 16-bit word.
    // It uses the match statement to determine which directive type it is, and then calls the to_u16 function to convert that variant into a 16-bit word.
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

impl SetTransmitterParameters {
    pub fn validate(&self) -> Result<(), String> {
        if self.tx_mode > 0x07 {
            return Err("SetTransmitterParameters.tx_mode must be 0..7".to_string());
        }
        if self.tx_data_rate > 0x0F {
            return Err("SetTransmitterParameters.tx_data_rate must be 0..15".to_string());
        }
        if self.tx_data_encoding > 0x03 {
            return Err("SetTransmitterParameters.tx_data_encoding must be 0..3".to_string());
        }
        if self.tx_frequency > 0x07 {
            return Err("SetTransmitterParameters.tx_frequency must be 0..7".to_string());
        }
        Ok(())
    }

    pub fn from_u16(word: u16) -> Self {
        SetTransmitterParameters {
            tx_mode: (word & 0x0007) as u8,                 // bits 0-2
            tx_data_rate: ((word >> 3) & 0x000F) as u8,     // bits 3-6
            tx_modulation: (word & (1 << 7)) != 0,          // bit 7
            tx_data_encoding: ((word >> 8) & 0x0003) as u8, // bits 8-9
            tx_frequency: ((word >> 10) & 0x0007) as u8,    // bits 10-12
        }
    }

    // The to_u16 function converts the SetTransmitterParameters object to a 16-bit word.
    // It uses the bitwise OR operator to set the bits of the 16-bit word.
    pub fn to_u16(&self) -> u16 {
        let mut word = 0u16;
        word |= (self.tx_mode as u16) & 0x0007; // bits 0-2
        word |= ((self.tx_data_rate as u16) & 0x000F) << 3; // bits 3-6
        if self.tx_modulation {
            word |= 1u16 << 7; // bit 7
        }
        word |= ((self.tx_data_encoding as u16) & 0x0003) << 8; // bits 8-9
        word |= ((self.tx_frequency as u16) & 0x0007) << 10;
        word |= 0b000 << 13; // bits 13-15
        word
    }
}

impl SetControlParameters {
    pub fn validate(&self) -> Result<(), String> {
        if self.time_sample > 0x3F {
            return Err("SetControlParameters.time_sample must be 0..63".to_string());
        }
        if self.duplex > 0x07 {
            return Err("SetControlParameters.duplex must be 0..7".to_string());
        }
        if self.reserved > 0x03 {
            return Err("SetControlParameters.reserved must be 0..3".to_string());
        }
        Ok(())
    }

    pub fn from_u16(word: u16) -> Self {
        SetControlParameters {
            time_sample: (word & 0x003F) as u8,           // bits 0-5
            duplex: ((word >> 6) & 0x07) as u8,           // bits 6-8
            reserved: ((word >> 9) & 0x03) as u8,         // bits 9-10
            remote_no_more_data: (word & (1 << 11)) != 0, // bit 11
            token: (word & (1 << 12)) != 0,               // bit 12
        }
    }

    // The to_u16 function converts the SetControlParameters object to a 16-bit word.
    // It uses the bitwise OR operator to set the bits of the 16-bit word.
    pub fn to_u16(&self) -> u16 {
        let mut word = (self.time_sample as u16) & 0x003F; // bits 0-5
        word |= ((self.duplex as u16) & 0x07) << 6; // bits 6-8
        if self.remote_no_more_data {
            word |= 1 << 11; // bit 11
        }
        if self.token {
            word |= 1 << 12; // bit 12
        }
        word |= 0b001 << 13; // bits 13-15
        word
    }
}

impl SetReceiverParameters {
    pub fn validate(&self) -> Result<(), String> {
        if self.rx_mode > 0x07 {
            return Err("SetReceiverParameters.rx_mode must be 0..7".to_string());
        }
        if self.rx_data_rate > 0x0F {
            return Err("SetReceiverParameters.rx_data_rate must be 0..15".to_string());
        }
        if self.rx_data_decoding > 0x03 {
            return Err("SetReceiverParameters.rx_data_decoding must be 0..3".to_string());
        }
        if self.rx_frequency > 0x07 {
            return Err("SetReceiverParameters.rx_frequency must be 0..7".to_string());
        }
        Ok(())
    }

    pub fn from_u16(word: u16) -> Self {
        SetReceiverParameters {
            rx_mode: (word & 0x0007) as u8,                 // bits 0-2
            rx_data_rate: ((word >> 3) & 0x000F) as u8,     // bits 3-6
            rx_modulation: (word & (1 << 7)) != 0,          // bit 7
            rx_data_decoding: ((word >> 8) & 0x0003) as u8, // bits 8-9
            rx_frequency: ((word >> 10) & 0x0007) as u8,    // bits 10-12
        }
    }

    // The to_u16 function converts the SetReceiverParameters object to a 16-bit word.
    // It uses the bitwise OR operator to set the bits of the 16-bit word.
    pub fn to_u16(&self) -> u16 {
        let mut word = 0u16;
        word |= (self.rx_mode as u16) & 0x0007; // bits 0-2
        word |= ((self.rx_data_rate as u16) & 0x000F) << 3; // bits 3-6
        if self.rx_modulation {
            word |= 1u16 << 7; // bit 7
        }
        word |= ((self.rx_data_decoding as u16) & 0x0003) << 8; // bits 8-9
        word |= ((self.rx_frequency as u16) & 0x0007) << 10; // bits 10-12
        word |= 0b010 << 13; // bits 13-15
        word
    }
}

impl SetVR {
    pub fn validate(&self) -> Result<(), String> {
        // seq_ctrl_fsn is 8-bit; any u8 is valid by type.
        Ok(())
    }

    pub fn from_u16(word: u16) -> Self {
        SetVR {
            seq_ctrl_fsn: (word & 0x00FF) as u8, // bits 0-7
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = (self.seq_ctrl_fsn as u16) & 0x00FF; // bits 0-7
        word |= 0b011 << 13; // bits 13-15
        word
    }
}

impl ReportRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.spare > 0x07 {
            return Err("ReportRequest.spare must be 0..7".to_string());
        }
        if self.status_report_request > 0x1F {
            return Err("ReportRequest.status_report_request must be 0..31".to_string());
        }
        if self.time_tag_request > 0x07 {
            return Err("ReportRequest.time_tag_request must be 0..7".to_string());
        }
        Ok(())
    }

    pub fn from_u16(word: u16) -> Self {
        ReportRequest {
            spare: (word & 0x0007) as u8, // bits 0-2
            status_report_request: ((word >> 3) & 0x1F) as u8,
            time_tag_request: ((word >> 8) & 0x07) as u8, // bits 8-10
            pcid0_plcw_request: (word & (1 << 11)) != 0,  // bit 11
            pcid1_plcw_request: (word & (1 << 12)) != 0,  // bit 12
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = ((self.status_report_request as u16) & 0x1F) << 3;
        word |= ((self.time_tag_request as u16) & 0x07) << 8;
        if self.pcid0_plcw_request {
            word |= 1 << 11; // bit 11
        }
        if self.pcid1_plcw_request {
            word |= 1 << 12; // bit 12
        }
        word |= 0b100 << 13; // bits 13-15
        word
    }
}

impl SetPLExtensions {
    pub fn validate(&self) -> Result<(), String> {
        if self.carrier_mod > 0x03 {
            return Err("SetPLExtensions.carrier_mod must be 0..3".to_string());
        }
        if self.data_mod > 0x03 {
            return Err("SetPLExtensions.data_mod must be 0..3".to_string());
        }
        if self.mode_select > 0x03 {
            return Err("SetPLExtensions.mode_select must be 0..3".to_string());
        }
        if self.scrambler > 0x03 {
            return Err("SetPLExtensions.scrambler must be 0..3".to_string());
        }
        Ok(())
    }

    pub fn from_u16(word: u16) -> Self {
        SetPLExtensions {
            direction: (word & (1 << 0)) != 0,           // bit 0
            freq_table: (word & (1 << 1)) != 0,          // bit 1
            rate_table: (word & (1 << 2)) != 0,          // bit 2
            carrier_mod: ((word >> 3) & 0x03) as u8,     // bits 3-4
            data_mod: ((word >> 5) & 0x03) as u8,        // bits 5-6
            mode_select: ((word >> 7) & 0x03) as u8,     // bits 7-8
            scrambler: ((word >> 9) & 0x03) as u8,       // bits 9-10
            diff_mark_encoding: (word & (1 << 11)) != 0, // bit 11
            rs_code: (word & (1 << 12)) != 0,            // bit 12
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = 0u16;
        if self.direction {
            word |= 1 << 0; // bit 0
        }
        if self.freq_table {
            word |= 1 << 1; // bit 1
        }
        if self.rate_table {
            word |= 1 << 2; // bit 2
        }
        word |= ((self.carrier_mod as u16) & 0x03) << 3; // bits 3-4
        word |= ((self.data_mod as u16) & 0x03) << 5; // bits 5-6
        word |= ((self.mode_select as u16) & 0x03) << 7; // bits 7-8
        word |= ((self.scrambler as u16) & 0x03) << 9; // bits 9-10
        if self.diff_mark_encoding {
            word |= 1 << 11; // bit 11
        }
        if self.rs_code {
            word |= 1 << 12; // bit 12
        }
        // This sets the Directive Type field (bits 13-15) to 0b110 (Set PLExtensions).
        word |= 0b110 << 13; // bits 13-15
        word
    }
}

impl ReportSourceSCID {
    pub fn validate(&self) -> Result<(), String> {
        if (self.source_scid & !0x03FF) != 0 {
            return Err("ReportSourceSCID.source_scid must be 10-bit (0..1023)".to_string());
        }
        Ok(())
    }

    pub fn from_u16(word: u16) -> Self {
        ReportSourceSCID {
            source_scid: word & 0x03FF, // bits 0-9
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut word = self.source_scid & 0x03FF; // bits 0-9
        // This sets the Directive Type field (bits 13-15) to 0b111 (Report Source SCID).
        word |= 0b111 << 13; // bits 13-15
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
