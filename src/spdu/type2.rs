#[derive(Debug, Clone, PartialEq)]
pub struct TimeDistributionPDU {
    pub directive_type: u8,          // octet 0
    pub transceiver_clock: [u8; 8],  // octets 1-8
    pub send_side_delay: [u8; 3],    // octets 9-11
    pub one_way_light_time: [u8; 3], // octets 12-14
}

impl TimeDistributionPDU {
    pub const LENGTH_OCTETS: usize = 15;

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() != Self::LENGTH_OCTETS {
            return Err("Type 2 SPDU data field must be exactly 15 octets".to_string());
        }
        let directive_type = data[0];
        let mut transceiver_clock = [0u8; 8];
        transceiver_clock.copy_from_slice(&data[1..9]);
        let mut send_side_delay = [0u8; 3];
        send_side_delay.copy_from_slice(&data[9..12]);
        let mut one_way_light_time = [0u8; 3];
        one_way_light_time.copy_from_slice(&data[12..15]);

        Ok(Self {
            directive_type,
            transceiver_clock,
            send_side_delay,
            one_way_light_time,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::LENGTH_OCTETS] {
        let mut out = [0u8; Self::LENGTH_OCTETS];
        out[0] = self.directive_type;
        out[1..9].copy_from_slice(&self.transceiver_clock);
        out[9..12].copy_from_slice(&self.send_side_delay);
        out[12..15].copy_from_slice(&self.one_way_light_time);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type2_time_distribution_roundtrip() {
        let pdu = TimeDistributionPDU {
            directive_type: 1,
            transceiver_clock: [1, 2, 3, 4, 5, 6, 7, 8],
            send_side_delay: [9, 10, 11],
            one_way_light_time: [12, 13, 14],
        };

        let bytes = pdu.to_bytes();
        let parsed = TimeDistributionPDU::from_bytes(&bytes).unwrap();
        assert_eq!(pdu, parsed);
    }
}

