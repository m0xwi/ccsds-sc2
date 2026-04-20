// Bit-level helpers (MSB-first within each octet, with "bit 0" as the first (MSB) bit).
// This matches the Annex D/E diagrams where fields are described from most-significant bit
// (Bit 0) to least-significant bit.

pub(crate) struct BitReader<'a> {
    data: &'a [u8],
    bit_pos: usize, // 0..=(data.len()*8)
}

impl<'a> BitReader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, bit_pos: 0 }
    }

    pub(crate) fn remaining_bits(&self) -> usize {
        self.data.len().saturating_mul(8).saturating_sub(self.bit_pos)
    }

    pub(crate) fn read_bits_u64(&mut self, n: usize) -> Result<u64, String> {
        if n > 64 {
            return Err("cannot read more than 64 bits into u64".to_string());
        }
        if self.remaining_bits() < n {
            return Err("not enough bits remaining".to_string());
        }
        let mut v = 0u64;
        for _ in 0..n {
            let byte_idx = self.bit_pos / 8;
            let bit_in_byte = self.bit_pos % 8; // 0 is MSB
            let bit = (self.data[byte_idx] >> (7 - bit_in_byte)) & 1;
            v = (v << 1) | (bit as u64);
            self.bit_pos += 1;
        }
        Ok(v)
    }

    pub(crate) fn read_bits_bytes(&mut self, n: usize) -> Result<Vec<u8>, String> {
        if self.remaining_bits() < n {
            return Err("not enough bits remaining".to_string());
        }
        let mut out = vec![0u8; (n + 7) / 8];
        for i in 0..n {
            let bit = self.read_bits_u64(1)? as u8;
            let out_byte = i / 8;
            let out_bit = i % 8;
            out[out_byte] |= bit << (7 - out_bit);
        }
        Ok(out)
    }
}

pub(crate) struct BitWriter {
    bits: Vec<u8>, // packed MSB-first
    bit_len: usize,
}

impl BitWriter {
    pub(crate) fn new() -> Self {
        Self { bits: Vec::new(), bit_len: 0 }
    }

    pub(crate) fn bit_len(&self) -> usize {
        self.bit_len
    }

    pub(crate) fn write_bits_u64(&mut self, v: u64, n: usize) {
        for i in (0..n).rev() {
            let bit = ((v >> i) & 1) as u8;
            self.write_bit(bit);
        }
    }

    pub(crate) fn write_bit(&mut self, bit: u8) {
        if (self.bit_len % 8) == 0 {
            self.bits.push(0);
        }
        let byte_idx = self.bit_len / 8;
        let bit_in_byte = self.bit_len % 8;
        self.bits[byte_idx] |= (bit & 1) << (7 - bit_in_byte);
        self.bit_len += 1;
    }

    pub(crate) fn write_bits_bytes(&mut self, bytes: &[u8], n: usize) {
        // bytes are interpreted MSB-first; only n bits are consumed.
        for i in 0..n {
            let byte_idx = i / 8;
            let bit_in_byte = i % 8;
            let bit = (bytes[byte_idx] >> (7 - bit_in_byte)) & 1;
            self.write_bit(bit);
        }
    }

    pub(crate) fn into_bytes_padded(self) -> Vec<u8> {
        self.bits
    }
}

