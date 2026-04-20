/// CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF, xorout 0x0000, no reflection).
///
/// This is a common CRC-16 used with CCSDS frame trailers. If your specific profile uses a
/// different CRC-16, we can swap this out behind the same interface.
pub fn crc16_ccitt_false(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}
