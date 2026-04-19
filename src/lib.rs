mod utils;

use wasm_bindgen::prelude::*;

/// A 128-bit SIMD register (equivalent to __m128i).
///
/// Bytes are stored in little-endian order: `bytes[0]` holds bits 7:0
/// and `bytes[15]` holds bits 127:120, matching x86 memory layout.
#[wasm_bindgen]
pub struct M128i {
    bytes: [u8; 16],
}

#[wasm_bindgen]
impl M128i {
    // ── Construction ────────────────────────────────────────────────────────

    /// Create a zero-initialised register.
    pub fn new() -> M128i {
        utils::set_panic_hook();
        M128i { bytes: [0u8; 16] }
    }

    /// Parse a hex string (with or without leading "0x") into a register.
    /// The string is interpreted as a big-endian 128-bit number: the
    /// leftmost hex digits are the most-significant bytes.
    pub fn from_hex(hex: &str) -> Result<M128i, JsValue> {
        let hex = hex.trim().trim_start_matches("0x").trim_start_matches("0X");
        let padded = format!("{:0>32}", hex);
        if padded.len() != 32 {
            return Err(JsValue::from_str("hex string too long (max 32 nibbles)"));
        }
        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[15 - i] = u8::from_str_radix(&padded[i * 2..i * 2 + 2], 16)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }
        Ok(M128i { bytes })
    }

    /// Create a register from four 32-bit signed integers (SSE _mm_set_epi32
    /// order: e3 is the most-significant lane, e0 the least-significant).
    pub fn from_epi32(e3: i32, e2: i32, e1: i32, e0: i32) -> M128i {
        let mut bytes = [0u8; 16];
        let words = [e0, e1, e2, e3];
        for (i, &w) in words.iter().enumerate() {
            let b = w.to_le_bytes();
            bytes[i * 4..i * 4 + 4].copy_from_slice(&b);
        }
        M128i { bytes }
    }

    /// Create a deep copy of this register.
    pub fn clone_reg(&self) -> M128i {
        M128i { bytes: self.bytes }
    }

    // ── Hex I/O ─────────────────────────────────────────────────────────────

    /// Return the register as a 34-character hex string "0x…" (big-endian,
    /// most-significant byte first, exactly 32 hex digits).
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(34);
        s.push_str("0x");
        for i in (0..16).rev() {
            s.push_str(&format!("{:02x}", self.bytes[i]));
        }
        s
    }

    // ── Lane value accessors (returned as JSON arrays for JS) ────────────────

    /// Return a JSON array of 16 signed byte (i8) lane values,
    /// index 0 = least-significant lane.
    pub fn get_epi8(&self) -> String {
        let strs: Vec<String> = self.bytes.iter().map(|&b| (b as i8).to_string()).collect();
        format!("[{}]", strs.join(","))
    }

    /// Return a JSON array of 8 signed 16-bit (i16) lane values,
    /// index 0 = least-significant lane.
    pub fn get_epi16(&self) -> String {
        let mut vals = Vec::with_capacity(8);
        for i in 0..8 {
            let v = i16::from_le_bytes([self.bytes[i * 2], self.bytes[i * 2 + 1]]);
            vals.push(v.to_string());
        }
        format!("[{}]", vals.join(","))
    }

    /// Return a JSON array of 4 signed 32-bit (i32) lane values,
    /// index 0 = least-significant lane.
    pub fn get_epi32(&self) -> String {
        let mut vals = Vec::with_capacity(4);
        for i in 0..4 {
            let v = i32::from_le_bytes([
                self.bytes[i * 4],
                self.bytes[i * 4 + 1],
                self.bytes[i * 4 + 2],
                self.bytes[i * 4 + 3],
            ]);
            vals.push(v.to_string());
        }
        format!("[{}]", vals.join(","))
    }

    /// Return a JSON array of 2 signed 64-bit (i64) lane values as strings,
    /// index 0 = least-significant lane.
    pub fn get_epi64(&self) -> String {
        let mut vals = Vec::with_capacity(2);
        for i in 0..2 {
            let v = i64::from_le_bytes([
                self.bytes[i * 8],
                self.bytes[i * 8 + 1],
                self.bytes[i * 8 + 2],
                self.bytes[i * 8 + 3],
                self.bytes[i * 8 + 4],
                self.bytes[i * 8 + 5],
                self.bytes[i * 8 + 6],
                self.bytes[i * 8 + 7],
            ]);
            vals.push(format!("\"{}\"", v));
        }
        format!("[{}]", vals.join(","))
    }

    /// Return a 128-character binary string, most-significant bit first.
    pub fn get_bits(&self) -> String {
        let mut s = String::with_capacity(128);
        for i in (0..16).rev() {
            for j in (0..8).rev() {
                s.push(if (self.bytes[i] >> j) & 1 == 1 { '1' } else { '0' });
            }
        }
        s
    }

    // ── Lane mutation ────────────────────────────────────────────────────────

    /// Set an individual i8 lane (lane 0 = least-significant).
    pub fn set_epi8_lane(&mut self, lane: usize, value: i32) {
        if lane < 16 {
            self.bytes[lane] = (value as i8) as u8;
        }
    }

    /// Set an individual i16 lane (lane 0 = least-significant).
    pub fn set_epi16_lane(&mut self, lane: usize, value: i32) {
        if lane < 8 {
            let b = (value as i16).to_le_bytes();
            self.bytes[lane * 2] = b[0];
            self.bytes[lane * 2 + 1] = b[1];
        }
    }

    /// Set an individual i32 lane (lane 0 = least-significant).
    pub fn set_epi32_lane(&mut self, lane: usize, value: i32) {
        if lane < 4 {
            let b = value.to_le_bytes();
            self.bytes[lane * 4..lane * 4 + 4].copy_from_slice(&b);
        }
    }

    // ── Bitwise operations (return a new register) ───────────────────────────

    /// Bitwise AND of two registers.
    pub fn and(&self, other: &M128i) -> M128i {
        let mut r = [0u8; 16];
        for i in 0..16 {
            r[i] = self.bytes[i] & other.bytes[i];
        }
        M128i { bytes: r }
    }

    /// Bitwise OR of two registers.
    pub fn or(&self, other: &M128i) -> M128i {
        let mut r = [0u8; 16];
        for i in 0..16 {
            r[i] = self.bytes[i] | other.bytes[i];
        }
        M128i { bytes: r }
    }

    /// Bitwise XOR of two registers.
    pub fn xor(&self, other: &M128i) -> M128i {
        let mut r = [0u8; 16];
        for i in 0..16 {
            r[i] = self.bytes[i] ^ other.bytes[i];
        }
        M128i { bytes: r }
    }

    /// Bitwise NOT (complement) of this register.
    pub fn not(&self) -> M128i {
        let mut r = [0u8; 16];
        for i in 0..16 {
            r[i] = !self.bytes[i];
        }
        M128i { bytes: r }
    }

    // ── Integer arithmetic per lane type ─────────────────────────────────────

    /// Add packed signed 8-bit integers (wrapping, _mm_add_epi8 semantics).
    pub fn add_epi8(&self, other: &M128i) -> M128i {
        let mut r = [0u8; 16];
        for i in 0..16 {
            r[i] = (self.bytes[i] as i8).wrapping_add(other.bytes[i] as i8) as u8;
        }
        M128i { bytes: r }
    }

    /// Add packed signed 16-bit integers (wrapping, _mm_add_epi16 semantics).
    pub fn add_epi16(&self, other: &M128i) -> M128i {
        let mut r = [0u8; 16];
        for i in 0..8 {
            let a = i16::from_le_bytes([self.bytes[i * 2], self.bytes[i * 2 + 1]]);
            let b = i16::from_le_bytes([other.bytes[i * 2], other.bytes[i * 2 + 1]]);
            let res = a.wrapping_add(b).to_le_bytes();
            r[i * 2] = res[0];
            r[i * 2 + 1] = res[1];
        }
        M128i { bytes: r }
    }

    /// Add packed signed 32-bit integers (wrapping, _mm_add_epi32 semantics).
    pub fn add_epi32(&self, other: &M128i) -> M128i {
        let mut r = [0u8; 16];
        for i in 0..4 {
            let a = i32::from_le_bytes([
                self.bytes[i * 4],
                self.bytes[i * 4 + 1],
                self.bytes[i * 4 + 2],
                self.bytes[i * 4 + 3],
            ]);
            let b = i32::from_le_bytes([
                other.bytes[i * 4],
                other.bytes[i * 4 + 1],
                other.bytes[i * 4 + 2],
                other.bytes[i * 4 + 3],
            ]);
            let res = a.wrapping_add(b).to_le_bytes();
            r[i * 4..i * 4 + 4].copy_from_slice(&res);
        }
        M128i { bytes: r }
    }

    /// Shift the entire 128-bit value left by `bits` bits (logical, fills 0s).
    pub fn shift_left_bits(&self, bits: u32) -> M128i {
        if bits >= 128 {
            return M128i { bytes: [0u8; 16] };
        }
        let byte_shift = (bits / 8) as usize;
        let bit_shift = bits % 8;
        let mut r = [0u8; 16];
        for i in byte_shift..16 {
            r[i] = self.bytes[i - byte_shift] << bit_shift;
            if bit_shift > 0 && i > byte_shift {
                r[i] |= self.bytes[i - byte_shift - 1] >> (8 - bit_shift);
            }
        }
        M128i { bytes: r }
    }

    /// Shift the entire 128-bit value right by `bits` bits (logical, fills 0s).
    pub fn shift_right_bits(&self, bits: u32) -> M128i {
        if bits >= 128 {
            return M128i { bytes: [0u8; 16] };
        }
        let byte_shift = (bits / 8) as usize;
        let bit_shift = bits % 8;
        let mut r = [0u8; 16];
        for i in 0..(16 - byte_shift) {
            r[i] = self.bytes[i + byte_shift] >> bit_shift;
            if bit_shift > 0 && i + byte_shift + 1 < 16 {
                r[i] |= self.bytes[i + byte_shift + 1] << (8 - bit_shift);
            }
        }
        M128i { bytes: r }
    }
}
