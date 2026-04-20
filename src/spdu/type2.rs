/// The TIME DISTRIBUTION SPDU is designed for time synchronization and distribution between two transceivers in a Proximity-1 session.

// Octet 0 describes the type time distribution directive

// Octet 1-8 describes the transceiver's internal clock at the exact moment of the trailing edge of the last bit of the ASM (Attached Sync Marker) of the transmitted crosses the internal clock capture point.
// It is a high-precision timestamp when the the time distribution SPDU was being transmitted.
// It is specifically tied to the transmission instant of the PLTU that carries the SPDU.

// Octet 9-11 is the send side delay
// It describes the measured delay between the internal clock capture point (moment capture in the transceiver clock field), and the moment the trailing edge of the last bit of the Sync-Marked Transfer Frame (SMTF) crosses the time reference point (the RF output / antenna reference point).
// It corrects for the known hardware/processing delay that occurs after the clock capture but before the signal actually leaves the transmitter.

// Octet 12-14 is the one-way light time
// This is the calculated propagation delay (light travel time) from:
// the instant the trailing edge of the last bit of the transmited SMTF crosses the initiator's time reference point, to the instant that same signal reaches the destination node's time reference point.

// In short, the transceiver clock is the timestamp at internal capture point (during PLTU transmission)
// The send-side delay is how long it took from capture to actual transmit reference point (SMTF out the door)
// The one-way light time is how long it takes the signal to fly through space to the other side.

// Together, the receiver can compute a very accurate time offset between the two transceivers' clocks.

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
