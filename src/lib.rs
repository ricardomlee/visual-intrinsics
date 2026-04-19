mod utils;

use wasm_bindgen::prelude::*;

// ── Low-level helpers operating on byte slices ───────────────────────────────
// All helpers take/return slices so they can be reused across register widths.

#[inline]
fn sat_add_i8(a: i8, b: i8) -> i8 {
    a.saturating_add(b)
}
#[inline]
fn sat_sub_i8(a: i8, b: i8) -> i8 {
    a.saturating_sub(b)
}
#[inline]
fn sat_add_i16(a: i16, b: i16) -> i16 {
    a.saturating_add(b)
}
#[inline]
fn sat_sub_i16(a: i16, b: i16) -> i16 {
    a.saturating_sub(b)
}
#[inline]
fn sat_add_u8(a: u8, b: u8) -> u8 {
    a.saturating_add(b)
}
#[inline]
fn sat_sub_u8(a: u8, b: u8) -> u8 {
    a.saturating_sub(b)
}
#[inline]
fn sat_add_u16(a: u16, b: u16) -> u16 {
    a.saturating_add(b)
}
#[inline]
fn sat_sub_u16(a: u16, b: u16) -> u16 {
    a.saturating_sub(b)
}

// Signed saturating pack i16 → i8
#[inline]
fn pack_i16_to_i8_sat(v: i16) -> i8 {
    v.max(i8::MIN as i16).min(i8::MAX as i16) as i8
}
// Unsigned saturating pack i16 → u8
#[inline]
fn pack_i16_to_u8_sat(v: i16) -> u8 {
    v.max(0).min(u8::MAX as i16) as u8
}
// Signed saturating pack i32 → i16
#[inline]
fn pack_i32_to_i16_sat(v: i32) -> i16 {
    v.max(i16::MIN as i32).min(i16::MAX as i32) as i16
}
// Unsigned saturating pack i32 → u16
#[inline]
fn pack_i32_to_u16_sat(v: i32) -> u16 {
    v.max(0).min(u16::MAX as i32) as u16
}

// ── Macro: generate M128i / M256i / M512i with a shared operation set ────────
//
// Per-128-bit-lane semantics (hadd, hsub, shuffle_epi32, shuffle_epi8,
// unpack*, packs*, alignr, blendv_epi8) are handled by iterating in
// 16-byte chunks, matching AVX2 / AVX-512 "in-lane" behaviour.

macro_rules! impl_simd {
    ($T:ident, $N:expr) => {
        #[wasm_bindgen]
        pub struct $T {
            bytes: [u8; $N],
        }

        #[wasm_bindgen]
        impl $T {
            // ── Construction ─────────────────────────────────────────────────

            /// Create a zero-initialised register.
            pub fn new() -> $T {
                utils::set_panic_hook();
                $T { bytes: [0u8; $N] }
            }

            /// Parse a big-endian hex string (with or without "0x") into the register.
            pub fn from_hex(hex: &str) -> Result<$T, JsValue> {
                let hex = hex.trim().trim_start_matches("0x").trim_start_matches("0X");
                let nibbles = $N * 2;
                let padded = format!("{:0>width$}", hex, width = nibbles);
                if padded.len() != nibbles {
                    return Err(JsValue::from_str(&format!(
                        "hex string too long (max {} nibbles)", nibbles
                    )));
                }
                let mut bytes = [0u8; $N];
                for i in 0..$N {
                    bytes[$N - 1 - i] =
                        u8::from_str_radix(&padded[i * 2..i * 2 + 2], 16)
                            .map_err(|e| JsValue::from_str(&e.to_string()))?;
                }
                Ok($T { bytes })
            }

            /// Deep copy of this register.
            pub fn clone_reg(&self) -> $T {
                $T { bytes: self.bytes }
            }

            // ── Hex / bit I/O ────────────────────────────────────────────────

            /// Big-endian hex string "0x…" with exactly $N*2 hex digits.
            pub fn to_hex(&self) -> String {
                let mut s = String::with_capacity(2 + $N * 2);
                s.push_str("0x");
                for i in (0..$N).rev() {
                    s.push_str(&format!("{:02x}", self.bytes[i]));
                }
                s
            }

            /// Binary string MSB-first, length = $N*8.
            pub fn get_bits(&self) -> String {
                let mut s = String::with_capacity($N * 8);
                for i in (0..$N).rev() {
                    for j in (0..8).rev() {
                        s.push(if (self.bytes[i] >> j) & 1 == 1 { '1' } else { '0' });
                    }
                }
                s
            }

            // ── Lane value accessors (JSON arrays, index 0 = LS lane) ────────

            /// JSON array of signed i8 lanes ($N lanes).
            pub fn get_epi8(&self) -> String {
                let strs: Vec<String> =
                    self.bytes.iter().map(|&b| (b as i8).to_string()).collect();
                format!("[{}]", strs.join(","))
            }

            /// JSON array of unsigned u8 lanes ($N lanes).
            pub fn get_epu8(&self) -> String {
                let strs: Vec<String> =
                    self.bytes.iter().map(|&b| b.to_string()).collect();
                format!("[{}]", strs.join(","))
            }

            /// JSON array of signed i16 lanes ($N/2 lanes).
            pub fn get_epi16(&self) -> String {
                let n = $N / 2;
                let mut vals = Vec::with_capacity(n);
                for i in 0..n {
                    let v = i16::from_le_bytes([self.bytes[i * 2], self.bytes[i * 2 + 1]]);
                    vals.push(v.to_string());
                }
                format!("[{}]", vals.join(","))
            }

            /// JSON array of unsigned u16 lanes ($N/2 lanes).
            pub fn get_epu16(&self) -> String {
                let n = $N / 2;
                let mut vals = Vec::with_capacity(n);
                for i in 0..n {
                    let v = u16::from_le_bytes([self.bytes[i * 2], self.bytes[i * 2 + 1]]);
                    vals.push(v.to_string());
                }
                format!("[{}]", vals.join(","))
            }

            /// JSON array of signed i32 lanes ($N/4 lanes).
            pub fn get_epi32(&self) -> String {
                let n = $N / 4;
                let mut vals = Vec::with_capacity(n);
                for i in 0..n {
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

            /// JSON array of unsigned u32 lanes ($N/4 lanes).
            pub fn get_epu32(&self) -> String {
                let n = $N / 4;
                let mut vals = Vec::with_capacity(n);
                for i in 0..n {
                    let v = u32::from_le_bytes([
                        self.bytes[i * 4],
                        self.bytes[i * 4 + 1],
                        self.bytes[i * 4 + 2],
                        self.bytes[i * 4 + 3],
                    ]);
                    vals.push(v.to_string());
                }
                format!("[{}]", vals.join(","))
            }

            /// JSON array of signed i64 lanes ($N/8 lanes), values as quoted strings.
            pub fn get_epi64(&self) -> String {
                let n = $N / 8;
                let mut vals = Vec::with_capacity(n);
                for i in 0..n {
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

            // ── Lane mutation ─────────────────────────────────────────────────

            /// Set i8 lane (lane 0 = least-significant byte).
            pub fn set_epi8_lane(&mut self, lane: usize, value: i32) {
                if lane < $N {
                    self.bytes[lane] = (value as i8) as u8;
                }
            }

            /// Set i16 lane (lane 0 = LS word).
            pub fn set_epi16_lane(&mut self, lane: usize, value: i32) {
                if lane < $N / 2 {
                    let b = (value as i16).to_le_bytes();
                    self.bytes[lane * 2] = b[0];
                    self.bytes[lane * 2 + 1] = b[1];
                }
            }

            /// Set i32 lane (lane 0 = LS dword).
            pub fn set_epi32_lane(&mut self, lane: usize, value: i32) {
                if lane < $N / 4 {
                    let b = value.to_le_bytes();
                    self.bytes[lane * 4..lane * 4 + 4].copy_from_slice(&b);
                }
            }

            // ── Bitwise ───────────────────────────────────────────────────────

            /// Bitwise AND.
            pub fn and(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N { r[i] = self.bytes[i] & other.bytes[i]; }
                $T { bytes: r }
            }

            /// Bitwise OR.
            pub fn or(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N { r[i] = self.bytes[i] | other.bytes[i]; }
                $T { bytes: r }
            }

            /// Bitwise XOR.
            pub fn xor(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N { r[i] = self.bytes[i] ^ other.bytes[i]; }
                $T { bytes: r }
            }

            /// Bitwise AND NOT: (NOT self) AND other  (_mm_andnot_si128 semantics).
            pub fn andnot(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N { r[i] = (!self.bytes[i]) & other.bytes[i]; }
                $T { bytes: r }
            }

            /// Bitwise NOT.
            pub fn not(&self) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N { r[i] = !self.bytes[i]; }
                $T { bytes: r }
            }

            // ── Full-register logical shifts ──────────────────────────────────

            /// Logical left shift of the whole register by `bits` bits (fills 0).
            pub fn shift_left_bits(&self, bits: u32) -> $T {
                if bits as usize >= $N * 8 {
                    return $T { bytes: [0u8; $N] };
                }
                let byte_shift = (bits / 8) as usize;
                let bit_shift  = bits % 8;
                let mut r = [0u8; $N];
                for i in byte_shift..$N {
                    r[i] = self.bytes[i - byte_shift] << bit_shift;
                    if bit_shift > 0 && i > byte_shift {
                        r[i] |= self.bytes[i - byte_shift - 1] >> (8 - bit_shift);
                    }
                }
                $T { bytes: r }
            }

            /// Logical right shift of the whole register by `bits` bits (fills 0).
            pub fn shift_right_bits(&self, bits: u32) -> $T {
                if bits as usize >= $N * 8 {
                    return $T { bytes: [0u8; $N] };
                }
                let byte_shift = (bits / 8) as usize;
                let bit_shift  = bits % 8;
                let mut r = [0u8; $N];
                for i in 0..($N - byte_shift) {
                    r[i] = self.bytes[i + byte_shift] >> bit_shift;
                    if bit_shift > 0 && i + byte_shift + 1 < $N {
                        r[i] |= self.bytes[i + byte_shift + 1] << (8 - bit_shift);
                    }
                }
                $T { bytes: r }
            }

            // ── Per-lane logical/arithmetic shifts ────────────────────────────

            /// Shift left each i16 lane by `count` bits (logical).
            pub fn slli_epi16(&self, count: u32) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let v = u16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let res = if count >= 16 { 0 } else { v << count };
                    let b = res.to_le_bytes();
                    r[i*2] = b[0]; r[i*2+1] = b[1];
                }
                $T { bytes: r }
            }

            /// Shift right each i16 lane by `count` bits (logical, zero-fills).
            pub fn srli_epi16(&self, count: u32) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let v = u16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let res = if count >= 16 { 0 } else { v >> count };
                    let b = res.to_le_bytes();
                    r[i*2] = b[0]; r[i*2+1] = b[1];
                }
                $T { bytes: r }
            }

            /// Shift right each i16 lane by `count` bits (arithmetic, sign-extends).
            pub fn srai_epi16(&self, count: u32) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let v = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let saturated = if v < 0 { -1i16 } else { 0 };
                    let res = if count >= 16 { saturated } else { v >> count };
                    let b = res.to_le_bytes();
                    r[i*2] = b[0]; r[i*2+1] = b[1];
                }
                $T { bytes: r }
            }

            /// Shift left each i32 lane by `count` bits (logical).
            pub fn slli_epi32(&self, count: u32) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let v = u32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let res = if count >= 32 { 0 } else { v << count };
                    let b = res.to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&b);
                }
                $T { bytes: r }
            }

            /// Shift right each i32 lane by `count` bits (logical, zero-fills).
            pub fn srli_epi32(&self, count: u32) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let v = u32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let res = if count >= 32 { 0 } else { v >> count };
                    let b = res.to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&b);
                }
                $T { bytes: r }
            }

            /// Shift right each i32 lane by `count` bits (arithmetic, sign-extends).
            pub fn srai_epi32(&self, count: u32) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let v = i32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let saturated = if v < 0 { -1i32 } else { 0 };
                    let res = if count >= 32 { saturated } else { v >> count };
                    let b = res.to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&b);
                }
                $T { bytes: r }
            }

            /// Shift left each i64 lane by `count` bits (logical).
            pub fn slli_epi64(&self, count: u32) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 8;
                for i in 0..n {
                    let v = u64::from_le_bytes([
                        self.bytes[i*8], self.bytes[i*8+1], self.bytes[i*8+2], self.bytes[i*8+3],
                        self.bytes[i*8+4], self.bytes[i*8+5], self.bytes[i*8+6], self.bytes[i*8+7],
                    ]);
                    let res = if count >= 64 { 0 } else { v << count };
                    let b = res.to_le_bytes();
                    r[i*8..i*8+8].copy_from_slice(&b);
                }
                $T { bytes: r }
            }

            /// Shift right each i64 lane by `count` bits (logical, zero-fills).
            pub fn srli_epi64(&self, count: u32) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 8;
                for i in 0..n {
                    let v = u64::from_le_bytes([
                        self.bytes[i*8], self.bytes[i*8+1], self.bytes[i*8+2], self.bytes[i*8+3],
                        self.bytes[i*8+4], self.bytes[i*8+5], self.bytes[i*8+6], self.bytes[i*8+7],
                    ]);
                    let res = if count >= 64 { 0 } else { v >> count };
                    let b = res.to_le_bytes();
                    r[i*8..i*8+8].copy_from_slice(&b);
                }
                $T { bytes: r }
            }

            // ── Add / Sub (wrapping) ──────────────────────────────────────────

            /// Add packed i8 lanes (wrapping).
            pub fn add_epi8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = (self.bytes[i] as i8).wrapping_add(other.bytes[i] as i8) as u8;
                }
                $T { bytes: r }
            }

            /// Add packed i16 lanes (wrapping).
            pub fn add_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = i16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let res = a.wrapping_add(b).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Add packed i32 lanes (wrapping).
            pub fn add_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let a = i32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let b = i32::from_le_bytes([
                        other.bytes[i*4], other.bytes[i*4+1],
                        other.bytes[i*4+2], other.bytes[i*4+3],
                    ]);
                    let res = a.wrapping_add(b).to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&res);
                }
                $T { bytes: r }
            }

            /// Add packed i64 lanes (wrapping).
            pub fn add_epi64(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 8;
                for i in 0..n {
                    let a = i64::from_le_bytes([
                        self.bytes[i*8], self.bytes[i*8+1], self.bytes[i*8+2], self.bytes[i*8+3],
                        self.bytes[i*8+4], self.bytes[i*8+5], self.bytes[i*8+6], self.bytes[i*8+7],
                    ]);
                    let b = i64::from_le_bytes([
                        other.bytes[i*8], other.bytes[i*8+1], other.bytes[i*8+2], other.bytes[i*8+3],
                        other.bytes[i*8+4], other.bytes[i*8+5], other.bytes[i*8+6], other.bytes[i*8+7],
                    ]);
                    let res = a.wrapping_add(b).to_le_bytes();
                    r[i*8..i*8+8].copy_from_slice(&res);
                }
                $T { bytes: r }
            }

            /// Subtract packed i8 lanes (wrapping).
            pub fn sub_epi8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = (self.bytes[i] as i8).wrapping_sub(other.bytes[i] as i8) as u8;
                }
                $T { bytes: r }
            }

            /// Subtract packed i16 lanes (wrapping).
            pub fn sub_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = i16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let res = a.wrapping_sub(b).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Subtract packed i32 lanes (wrapping).
            pub fn sub_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let a = i32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let b = i32::from_le_bytes([
                        other.bytes[i*4], other.bytes[i*4+1],
                        other.bytes[i*4+2], other.bytes[i*4+3],
                    ]);
                    let res = a.wrapping_sub(b).to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&res);
                }
                $T { bytes: r }
            }

            /// Subtract packed i64 lanes (wrapping).
            pub fn sub_epi64(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 8;
                for i in 0..n {
                    let a = i64::from_le_bytes([
                        self.bytes[i*8], self.bytes[i*8+1], self.bytes[i*8+2], self.bytes[i*8+3],
                        self.bytes[i*8+4], self.bytes[i*8+5], self.bytes[i*8+6], self.bytes[i*8+7],
                    ]);
                    let b = i64::from_le_bytes([
                        other.bytes[i*8], other.bytes[i*8+1], other.bytes[i*8+2], other.bytes[i*8+3],
                        other.bytes[i*8+4], other.bytes[i*8+5], other.bytes[i*8+6], other.bytes[i*8+7],
                    ]);
                    let res = a.wrapping_sub(b).to_le_bytes();
                    r[i*8..i*8+8].copy_from_slice(&res);
                }
                $T { bytes: r }
            }

            // ── Saturating add / sub ──────────────────────────────────────────

            /// Saturating add i8 lanes (signed).
            pub fn adds_epi8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = sat_add_i8(self.bytes[i] as i8, other.bytes[i] as i8) as u8;
                }
                $T { bytes: r }
            }

            /// Saturating sub i8 lanes (signed).
            pub fn subs_epi8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = sat_sub_i8(self.bytes[i] as i8, other.bytes[i] as i8) as u8;
                }
                $T { bytes: r }
            }

            /// Saturating add i16 lanes (signed).
            pub fn adds_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = i16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let res = sat_add_i16(a, b).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Saturating sub i16 lanes (signed).
            pub fn subs_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = i16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let res = sat_sub_i16(a, b).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Saturating add u8 lanes (unsigned).
            pub fn adds_epu8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = sat_add_u8(self.bytes[i], other.bytes[i]);
                }
                $T { bytes: r }
            }

            /// Saturating sub u8 lanes (unsigned).
            pub fn subs_epu8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = sat_sub_u8(self.bytes[i], other.bytes[i]);
                }
                $T { bytes: r }
            }

            /// Saturating add u16 lanes (unsigned).
            pub fn adds_epu16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = u16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = u16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let res = sat_add_u16(a, b).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Saturating sub u16 lanes (unsigned).
            pub fn subs_epu16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = u16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = u16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let res = sat_sub_u16(a, b).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            // ── Multiply ──────────────────────────────────────────────────────

            /// Multiply packed i16 lanes, keep low 16 bits (_mm_mullo_epi16).
            pub fn mullo_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]) as i32;
                    let b = i16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]) as i32;
                    let res = ((a * b) as i16).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Multiply packed i16 lanes, keep high 16 bits (_mm_mulhi_epi16).
            pub fn mulhi_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]) as i32;
                    let b = i16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]) as i32;
                    let res = (((a * b) >> 16) as i16).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Multiply packed i32 lanes, keep low 32 bits (_mm_mullo_epi32 / SSE4.1).
            pub fn mullo_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let a = i32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]) as i64;
                    let b = i32::from_le_bytes([
                        other.bytes[i*4], other.bytes[i*4+1],
                        other.bytes[i*4+2], other.bytes[i*4+3],
                    ]) as i64;
                    let res = ((a * b) as i32).to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&res);
                }
                $T { bytes: r }
            }

            // ── Absolute value ────────────────────────────────────────────────

            /// Absolute value of packed i8 lanes (_mm_abs_epi8 / SSSE3).
            pub fn abs_epi8(&self) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = (self.bytes[i] as i8).unsigned_abs();
                }
                $T { bytes: r }
            }

            /// Absolute value of packed i16 lanes (_mm_abs_epi16 / SSSE3).
            pub fn abs_epi16(&self) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let v = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let res = v.unsigned_abs().to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Absolute value of packed i32 lanes (_mm_abs_epi32 / SSSE3).
            pub fn abs_epi32(&self) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let v = i32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let res = v.unsigned_abs().to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&res);
                }
                $T { bytes: r }
            }

            // ── Min / Max (signed) ────────────────────────────────────────────

            /// Minimum of packed i8 lanes.
            pub fn min_epi8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = (self.bytes[i] as i8).min(other.bytes[i] as i8) as u8;
                }
                $T { bytes: r }
            }

            /// Maximum of packed i8 lanes.
            pub fn max_epi8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = (self.bytes[i] as i8).max(other.bytes[i] as i8) as u8;
                }
                $T { bytes: r }
            }

            /// Minimum of packed i16 lanes.
            pub fn min_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = i16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let res = a.min(b).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Maximum of packed i16 lanes.
            pub fn max_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = i16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let res = a.max(b).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Minimum of packed i32 lanes.
            pub fn min_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let a = i32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let b = i32::from_le_bytes([
                        other.bytes[i*4], other.bytes[i*4+1],
                        other.bytes[i*4+2], other.bytes[i*4+3],
                    ]);
                    let res = a.min(b).to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&res);
                }
                $T { bytes: r }
            }

            /// Maximum of packed i32 lanes.
            pub fn max_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let a = i32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let b = i32::from_le_bytes([
                        other.bytes[i*4], other.bytes[i*4+1],
                        other.bytes[i*4+2], other.bytes[i*4+3],
                    ]);
                    let res = a.max(b).to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&res);
                }
                $T { bytes: r }
            }

            // ── Min / Max (unsigned) ──────────────────────────────────────────

            /// Minimum of packed u8 lanes.
            pub fn min_epu8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = self.bytes[i].min(other.bytes[i]);
                }
                $T { bytes: r }
            }

            /// Maximum of packed u8 lanes.
            pub fn max_epu8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = self.bytes[i].max(other.bytes[i]);
                }
                $T { bytes: r }
            }

            /// Minimum of packed u16 lanes.
            pub fn min_epu16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = u16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = u16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let res = a.min(b).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Maximum of packed u16 lanes.
            pub fn max_epu16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = u16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = u16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let res = a.max(b).to_le_bytes();
                    r[i*2] = res[0]; r[i*2+1] = res[1];
                }
                $T { bytes: r }
            }

            /// Minimum of packed u32 lanes.
            pub fn min_epu32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let a = u32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let b = u32::from_le_bytes([
                        other.bytes[i*4], other.bytes[i*4+1],
                        other.bytes[i*4+2], other.bytes[i*4+3],
                    ]);
                    let res = a.min(b).to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&res);
                }
                $T { bytes: r }
            }

            /// Maximum of packed u32 lanes.
            pub fn max_epu32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let a = u32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let b = u32::from_le_bytes([
                        other.bytes[i*4], other.bytes[i*4+1],
                        other.bytes[i*4+2], other.bytes[i*4+3],
                    ]);
                    let res = a.max(b).to_le_bytes();
                    r[i*4..i*4+4].copy_from_slice(&res);
                }
                $T { bytes: r }
            }

            // ── Compare (returns all-1s / all-0s per lane) ────────────────────

            /// Compare eq packed i8 lanes (all-1s if equal, all-0s otherwise).
            pub fn cmpeq_epi8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = if self.bytes[i] == other.bytes[i] { 0xff } else { 0x00 };
                }
                $T { bytes: r }
            }

            /// Compare gt (signed) packed i8 lanes.
            pub fn cmpgt_epi8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = if (self.bytes[i] as i8) > (other.bytes[i] as i8) { 0xff } else { 0x00 };
                }
                $T { bytes: r }
            }

            /// Compare eq packed i16 lanes.
            pub fn cmpeq_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = i16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let fill: u8 = if a == b { 0xff } else { 0x00 };
                    r[i*2] = fill; r[i*2+1] = fill;
                }
                $T { bytes: r }
            }

            /// Compare gt (signed) packed i16 lanes.
            pub fn cmpgt_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 2;
                for i in 0..n {
                    let a = i16::from_le_bytes([self.bytes[i*2], self.bytes[i*2+1]]);
                    let b = i16::from_le_bytes([other.bytes[i*2], other.bytes[i*2+1]]);
                    let fill: u8 = if a > b { 0xff } else { 0x00 };
                    r[i*2] = fill; r[i*2+1] = fill;
                }
                $T { bytes: r }
            }

            /// Compare eq packed i32 lanes.
            pub fn cmpeq_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let a = i32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let b = i32::from_le_bytes([
                        other.bytes[i*4], other.bytes[i*4+1],
                        other.bytes[i*4+2], other.bytes[i*4+3],
                    ]);
                    let fill: u8 = if a == b { 0xff } else { 0x00 };
                    r[i*4] = fill; r[i*4+1] = fill;
                    r[i*4+2] = fill; r[i*4+3] = fill;
                }
                $T { bytes: r }
            }

            /// Compare gt (signed) packed i32 lanes.
            pub fn cmpgt_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 4;
                for i in 0..n {
                    let a = i32::from_le_bytes([
                        self.bytes[i*4], self.bytes[i*4+1],
                        self.bytes[i*4+2], self.bytes[i*4+3],
                    ]);
                    let b = i32::from_le_bytes([
                        other.bytes[i*4], other.bytes[i*4+1],
                        other.bytes[i*4+2], other.bytes[i*4+3],
                    ]);
                    let fill: u8 = if a > b { 0xff } else { 0x00 };
                    r[i*4] = fill; r[i*4+1] = fill;
                    r[i*4+2] = fill; r[i*4+3] = fill;
                }
                $T { bytes: r }
            }

            /// Compare eq packed i64 lanes.
            pub fn cmpeq_epi64(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 8;
                for i in 0..n {
                    let a = i64::from_le_bytes([
                        self.bytes[i*8], self.bytes[i*8+1], self.bytes[i*8+2], self.bytes[i*8+3],
                        self.bytes[i*8+4], self.bytes[i*8+5], self.bytes[i*8+6], self.bytes[i*8+7],
                    ]);
                    let b = i64::from_le_bytes([
                        other.bytes[i*8], other.bytes[i*8+1], other.bytes[i*8+2], other.bytes[i*8+3],
                        other.bytes[i*8+4], other.bytes[i*8+5], other.bytes[i*8+6], other.bytes[i*8+7],
                    ]);
                    let fill: u8 = if a == b { 0xff } else { 0x00 };
                    for k in 0..8 { r[i*8+k] = fill; }
                }
                $T { bytes: r }
            }

            /// Compare gt (signed) packed i64 lanes.
            pub fn cmpgt_epi64(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                let n = $N / 8;
                for i in 0..n {
                    let a = i64::from_le_bytes([
                        self.bytes[i*8], self.bytes[i*8+1], self.bytes[i*8+2], self.bytes[i*8+3],
                        self.bytes[i*8+4], self.bytes[i*8+5], self.bytes[i*8+6], self.bytes[i*8+7],
                    ]);
                    let b = i64::from_le_bytes([
                        other.bytes[i*8], other.bytes[i*8+1], other.bytes[i*8+2], other.bytes[i*8+3],
                        other.bytes[i*8+4], other.bytes[i*8+5], other.bytes[i*8+6], other.bytes[i*8+7],
                    ]);
                    let fill: u8 = if a > b { 0xff } else { 0x00 };
                    for k in 0..8 { r[i*8+k] = fill; }
                }
                $T { bytes: r }
            }

            // ── Horizontal add / sub (in-lane, per 128-bit chunk) ─────────────

            /// Horizontal add adjacent i16 pairs within each 128-bit lane
            /// (_mm_hadd_epi16 / _mm256_hadd_epi16 in-lane semantics).
            pub fn hadd_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    // lower 8 bytes: hadd from self
                    for i in 0..4 {
                        let a0 = i16::from_le_bytes([self.bytes[base+i*4], self.bytes[base+i*4+1]]);
                        let a1 = i16::from_le_bytes([self.bytes[base+i*4+2], self.bytes[base+i*4+3]]);
                        let res = a0.wrapping_add(a1).to_le_bytes();
                        r[base+i*2] = res[0]; r[base+i*2+1] = res[1];
                    }
                    // upper 8 bytes: hadd from other
                    for i in 0..4 {
                        let b0 = i16::from_le_bytes([other.bytes[base+i*4], other.bytes[base+i*4+1]]);
                        let b1 = i16::from_le_bytes([other.bytes[base+i*4+2], other.bytes[base+i*4+3]]);
                        let res = b0.wrapping_add(b1).to_le_bytes();
                        r[base+8+i*2] = res[0]; r[base+8+i*2+1] = res[1];
                    }
                }
                $T { bytes: r }
            }

            /// Horizontal sub adjacent i16 pairs within each 128-bit lane.
            pub fn hsub_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..4 {
                        let a0 = i16::from_le_bytes([self.bytes[base+i*4], self.bytes[base+i*4+1]]);
                        let a1 = i16::from_le_bytes([self.bytes[base+i*4+2], self.bytes[base+i*4+3]]);
                        let res = a0.wrapping_sub(a1).to_le_bytes();
                        r[base+i*2] = res[0]; r[base+i*2+1] = res[1];
                    }
                    for i in 0..4 {
                        let b0 = i16::from_le_bytes([other.bytes[base+i*4], other.bytes[base+i*4+1]]);
                        let b1 = i16::from_le_bytes([other.bytes[base+i*4+2], other.bytes[base+i*4+3]]);
                        let res = b0.wrapping_sub(b1).to_le_bytes();
                        r[base+8+i*2] = res[0]; r[base+8+i*2+1] = res[1];
                    }
                }
                $T { bytes: r }
            }

            /// Horizontal add adjacent i32 pairs within each 128-bit lane.
            pub fn hadd_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    // lower 8 bytes: hadd from self
                    for i in 0..2 {
                        let a0 = i32::from_le_bytes([
                            self.bytes[base+i*8], self.bytes[base+i*8+1],
                            self.bytes[base+i*8+2], self.bytes[base+i*8+3],
                        ]);
                        let a1 = i32::from_le_bytes([
                            self.bytes[base+i*8+4], self.bytes[base+i*8+5],
                            self.bytes[base+i*8+6], self.bytes[base+i*8+7],
                        ]);
                        let res = a0.wrapping_add(a1).to_le_bytes();
                        r[base+i*4..base+i*4+4].copy_from_slice(&res);
                    }
                    // upper 8 bytes: hadd from other
                    for i in 0..2 {
                        let b0 = i32::from_le_bytes([
                            other.bytes[base+i*8], other.bytes[base+i*8+1],
                            other.bytes[base+i*8+2], other.bytes[base+i*8+3],
                        ]);
                        let b1 = i32::from_le_bytes([
                            other.bytes[base+i*8+4], other.bytes[base+i*8+5],
                            other.bytes[base+i*8+6], other.bytes[base+i*8+7],
                        ]);
                        let res = b0.wrapping_add(b1).to_le_bytes();
                        r[base+8+i*4..base+8+i*4+4].copy_from_slice(&res);
                    }
                }
                $T { bytes: r }
            }

            /// Horizontal sub adjacent i32 pairs within each 128-bit lane.
            pub fn hsub_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..2 {
                        let a0 = i32::from_le_bytes([
                            self.bytes[base+i*8], self.bytes[base+i*8+1],
                            self.bytes[base+i*8+2], self.bytes[base+i*8+3],
                        ]);
                        let a1 = i32::from_le_bytes([
                            self.bytes[base+i*8+4], self.bytes[base+i*8+5],
                            self.bytes[base+i*8+6], self.bytes[base+i*8+7],
                        ]);
                        let res = a0.wrapping_sub(a1).to_le_bytes();
                        r[base+i*4..base+i*4+4].copy_from_slice(&res);
                    }
                    for i in 0..2 {
                        let b0 = i32::from_le_bytes([
                            other.bytes[base+i*8], other.bytes[base+i*8+1],
                            other.bytes[base+i*8+2], other.bytes[base+i*8+3],
                        ]);
                        let b1 = i32::from_le_bytes([
                            other.bytes[base+i*8+4], other.bytes[base+i*8+5],
                            other.bytes[base+i*8+6], other.bytes[base+i*8+7],
                        ]);
                        let res = b0.wrapping_sub(b1).to_le_bytes();
                        r[base+8+i*4..base+8+i*4+4].copy_from_slice(&res);
                    }
                }
                $T { bytes: r }
            }

            // ── Unpack (interleave, in-lane per 128-bit chunk) ─────────────────

            /// Interleave low i8 halves of each 128-bit lane (unpacklo_epi8).
            pub fn unpacklo_epi8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..8 {
                        r[base + i*2]     = self.bytes[base + i];
                        r[base + i*2 + 1] = other.bytes[base + i];
                    }
                }
                $T { bytes: r }
            }

            /// Interleave high i8 halves of each 128-bit lane (unpackhi_epi8).
            pub fn unpackhi_epi8(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..8 {
                        r[base + i*2]     = self.bytes[base + 8 + i];
                        r[base + i*2 + 1] = other.bytes[base + 8 + i];
                    }
                }
                $T { bytes: r }
            }

            /// Interleave low i16 halves of each 128-bit lane (unpacklo_epi16).
            pub fn unpacklo_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..4 {
                        r[base + i*4]     = self.bytes[base + i*2];
                        r[base + i*4 + 1] = self.bytes[base + i*2 + 1];
                        r[base + i*4 + 2] = other.bytes[base + i*2];
                        r[base + i*4 + 3] = other.bytes[base + i*2 + 1];
                    }
                }
                $T { bytes: r }
            }

            /// Interleave high i16 halves of each 128-bit lane (unpackhi_epi16).
            pub fn unpackhi_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..4 {
                        r[base + i*4]     = self.bytes[base + 8 + i*2];
                        r[base + i*4 + 1] = self.bytes[base + 8 + i*2 + 1];
                        r[base + i*4 + 2] = other.bytes[base + 8 + i*2];
                        r[base + i*4 + 3] = other.bytes[base + 8 + i*2 + 1];
                    }
                }
                $T { bytes: r }
            }

            /// Interleave low i32 halves of each 128-bit lane (unpacklo_epi32).
            pub fn unpacklo_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..2 {
                        r[base + i*8..base + i*8 + 4]
                            .copy_from_slice(&self.bytes[base + i*4..base + i*4 + 4]);
                        r[base + i*8 + 4..base + i*8 + 8]
                            .copy_from_slice(&other.bytes[base + i*4..base + i*4 + 4]);
                    }
                }
                $T { bytes: r }
            }

            /// Interleave high i32 halves of each 128-bit lane (unpackhi_epi32).
            pub fn unpackhi_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..2 {
                        r[base + i*8..base + i*8 + 4]
                            .copy_from_slice(&self.bytes[base + 8 + i*4..base + 8 + i*4 + 4]);
                        r[base + i*8 + 4..base + i*8 + 8]
                            .copy_from_slice(&other.bytes[base + 8 + i*4..base + 8 + i*4 + 4]);
                    }
                }
                $T { bytes: r }
            }

            /// Interleave low i64 halves of each 128-bit lane (unpacklo_epi64).
            pub fn unpacklo_epi64(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    r[base..base+8].copy_from_slice(&self.bytes[base..base+8]);
                    r[base+8..base+16].copy_from_slice(&other.bytes[base..base+8]);
                }
                $T { bytes: r }
            }

            /// Interleave high i64 halves of each 128-bit lane (unpackhi_epi64).
            pub fn unpackhi_epi64(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    r[base..base+8].copy_from_slice(&self.bytes[base+8..base+16]);
                    r[base+8..base+16].copy_from_slice(&other.bytes[base+8..base+16]);
                }
                $T { bytes: r }
            }

            // ── Pack (with saturation, in-lane per 128-bit chunk) ─────────────

            /// Pack i16 → i8 with signed saturation per 128-bit lane (packs_epi16).
            /// Lower half of result = packed self; upper half = packed other (per lane).
            pub fn packs_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..8 {
                        let v = i16::from_le_bytes([self.bytes[base + i*2], self.bytes[base + i*2+1]]);
                        r[base + i] = pack_i16_to_i8_sat(v) as u8;
                    }
                    for i in 0..8 {
                        let v = i16::from_le_bytes([other.bytes[base + i*2], other.bytes[base + i*2+1]]);
                        r[base + 8 + i] = pack_i16_to_i8_sat(v) as u8;
                    }
                }
                $T { bytes: r }
            }

            /// Pack i16 → u8 with unsigned saturation per 128-bit lane (packus_epi16).
            pub fn packus_epi16(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..8 {
                        let v = i16::from_le_bytes([self.bytes[base + i*2], self.bytes[base + i*2+1]]);
                        r[base + i] = pack_i16_to_u8_sat(v);
                    }
                    for i in 0..8 {
                        let v = i16::from_le_bytes([other.bytes[base + i*2], other.bytes[base + i*2+1]]);
                        r[base + 8 + i] = pack_i16_to_u8_sat(v);
                    }
                }
                $T { bytes: r }
            }

            /// Pack i32 → i16 with signed saturation per 128-bit lane (packs_epi32).
            pub fn packs_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..4 {
                        let v = i32::from_le_bytes([
                            self.bytes[base+i*4], self.bytes[base+i*4+1],
                            self.bytes[base+i*4+2], self.bytes[base+i*4+3],
                        ]);
                        let res = pack_i32_to_i16_sat(v).to_le_bytes();
                        r[base+i*2] = res[0]; r[base+i*2+1] = res[1];
                    }
                    for i in 0..4 {
                        let v = i32::from_le_bytes([
                            other.bytes[base+i*4], other.bytes[base+i*4+1],
                            other.bytes[base+i*4+2], other.bytes[base+i*4+3],
                        ]);
                        let res = pack_i32_to_i16_sat(v).to_le_bytes();
                        r[base+8+i*2] = res[0]; r[base+8+i*2+1] = res[1];
                    }
                }
                $T { bytes: r }
            }

            /// Pack i32 → u16 with unsigned saturation per 128-bit lane (packus_epi32).
            pub fn packus_epi32(&self, other: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..4 {
                        let v = i32::from_le_bytes([
                            self.bytes[base+i*4], self.bytes[base+i*4+1],
                            self.bytes[base+i*4+2], self.bytes[base+i*4+3],
                        ]);
                        let res = pack_i32_to_u16_sat(v).to_le_bytes();
                        r[base+i*2] = res[0]; r[base+i*2+1] = res[1];
                    }
                    for i in 0..4 {
                        let v = i32::from_le_bytes([
                            other.bytes[base+i*4], other.bytes[base+i*4+1],
                            other.bytes[base+i*4+2], other.bytes[base+i*4+3],
                        ]);
                        let res = pack_i32_to_u16_sat(v).to_le_bytes();
                        r[base+8+i*2] = res[0]; r[base+8+i*2+1] = res[1];
                    }
                }
                $T { bytes: r }
            }

            // ── Shuffle ───────────────────────────────────────────────────────

            /// Shuffle i32 lanes within each 128-bit lane using an 8-bit immediate
            /// (_mm_shuffle_epi32 / _mm256_shuffle_epi32 in-lane semantics).
            /// imm8 selects which of the 4 dwords in the lane goes to each output.
            pub fn shuffle_epi32(&self, imm8: u8) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..4usize {
                        let src = ((imm8 >> (i * 2)) & 3) as usize;
                        r[base + i*4..base + i*4 + 4]
                            .copy_from_slice(&self.bytes[base + src*4..base + src*4 + 4]);
                    }
                }
                $T { bytes: r }
            }

            /// Byte shuffle within each 128-bit lane using `mask` register (PSHUFB).
            /// If the high bit of a mask byte is set, the output byte is zeroed.
            pub fn shuffle_epi8(&self, mask: &$T) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    for i in 0..16 {
                        let m = mask.bytes[base + i];
                        if m & 0x80 != 0 {
                            r[base + i] = 0;
                        } else {
                            let idx = (m & 0x0f) as usize;
                            r[base + i] = self.bytes[base + idx];
                        }
                    }
                }
                $T { bytes: r }
            }

            // ── Byte align (in-lane per 128-bit chunk) ────────────────────────

            /// Concatenate `[other | self]` (other in high bytes) per 128-bit lane,
            /// then shift right by `imm8` bytes (_mm_alignr_epi8 / _mm256_alignr_epi8 in-lane).
            pub fn alignr_epi8(&self, other: &$T, imm8: u32) -> $T {
                let mut r = [0u8; $N];
                for chunk in 0..($N / 16) {
                    let base = chunk * 16;
                    // concatenated 32-byte window: [other | self]
                    for i in 0..16usize {
                        let shift = imm8 as usize;
                        if shift >= 32 {
                            r[base + i] = 0;
                        } else {
                            let src = shift + i;
                            r[base + i] = if src < 16 {
                                other.bytes[base + src]
                            } else if src < 32 {
                                self.bytes[base + src - 16]
                            } else {
                                0
                            };
                        }
                    }
                }
                $T { bytes: r }
            }

            // ── Blend ─────────────────────────────────────────────────────────

            /// Byte blend using `mask` register: select self byte if mask MSB=0,
            /// other byte if mask MSB=1 (_mm_blendv_epi8 / _mm256_blendv_epi8).
            pub fn blendv_epi8(&self, other: &$T, mask: &$T) -> $T {
                let mut r = [0u8; $N];
                for i in 0..$N {
                    r[i] = if mask.bytes[i] & 0x80 != 0 {
                        other.bytes[i]
                    } else {
                        self.bytes[i]
                    };
                }
                $T { bytes: r }
            }
        }
    };
}

// ── Generate the three register types ────────────────────────────────────────

impl_simd!(M128i, 16);
impl_simd!(M256i, 32);
impl_simd!(M512i, 64);

// ── M128i-only extras ────────────────────────────────────────────────────────

#[wasm_bindgen]
impl M128i {
    /// Create from four 32-bit signed integers (_mm_set_epi32 order:
    /// e3 = most-significant lane, e0 = least-significant).
    pub fn from_epi32(e3: i32, e2: i32, e1: i32, e0: i32) -> M128i {
        let mut bytes = [0u8; 16];
        for (i, &w) in [e0, e1, e2, e3].iter().enumerate() {
            bytes[i*4..i*4+4].copy_from_slice(&w.to_le_bytes());
        }
        M128i { bytes }
    }
}
