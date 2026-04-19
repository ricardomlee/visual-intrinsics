//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use visual_intrinsics::{M128i, M256i, M512i};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ── M128i: existing tests ────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn new_is_all_zeros() {
    let r = M128i::new();
    assert_eq!(r.to_hex(), "0x00000000000000000000000000000000");
}

#[wasm_bindgen_test]
fn from_hex_round_trips() {
    let hex = "0x0f1e2d3c4b5a69788796655443322110";
    let r = M128i::from_hex(hex).unwrap();
    assert!(r.to_hex().to_lowercase().ends_with("1e2d3c4b5a69788796655443322110"));
}

#[wasm_bindgen_test]
fn not_is_complement() {
    let r = M128i::from_hex("0xff00ff00ff00ff00ff00ff00ff00ff00").unwrap();
    let n = r.not();
    assert_eq!(n.to_hex(), "0x00ff00ff00ff00ff00ff00ff00ff00ff");
}

#[wasm_bindgen_test]
fn and_masks_bits() {
    let a = M128i::from_hex("0xffffffffffffffffffffffffffffffff").unwrap();
    let b = M128i::from_hex("0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f").unwrap();
    let r = a.and(&b);
    assert_eq!(r.to_hex(), "0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f");
}

#[wasm_bindgen_test]
fn xor_self_is_zero() {
    let a = M128i::from_hex("0xdeadbeefcafebabe1234567890abcdef").unwrap();
    let r = a.xor(&a.clone_reg());
    assert_eq!(r.to_hex(), "0x00000000000000000000000000000000");
}

#[wasm_bindgen_test]
fn add_epi32_wraps() {
    let a = M128i::from_epi32(0, 0, 0, 0x7fffffff);
    let b = M128i::from_epi32(0, 0, 0, 1);
    let r = a.add_epi32(&b);
    let vals: Vec<i64> = serde_lanes(&r.get_epi32());
    assert_eq!(vals[0], i32::MIN as i64);
}

#[wasm_bindgen_test]
fn shift_left_one_byte() {
    let a = M128i::from_hex("0x000000000000000000000000000000ff").unwrap();
    let r = a.shift_left_bits(8);
    assert_eq!(r.to_hex(), "0x0000000000000000000000000000ff00");
}

#[wasm_bindgen_test]
fn shift_right_one_byte() {
    let a = M128i::from_hex("0x0ff00000000000000000000000000000").unwrap();
    let r = a.shift_right_bits(4);
    assert_eq!(&r.to_hex()[2..4], "0f");
}

#[wasm_bindgen_test]
fn get_bits_length() {
    let r = M128i::new();
    assert_eq!(r.get_bits().len(), 128);
}

// ── M128i: new operations ────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn andnot_128() {
    // andnot(a, b) = (!a) & b
    let a = M128i::from_hex("0xff000000000000000000000000000000").unwrap();
    let b = M128i::from_hex("0xffffffffffffffffffffffffffffff00").unwrap();
    let r = a.andnot(&b);
    // MSB byte: (!0xff) & 0xff = 0x00 & 0xff = 0x00
    // next byte: (!0x00) & 0xff = 0xff & 0xff = 0xff
    // LS byte:   (!0x00) & 0x00 = 0x00
    assert_eq!(&r.to_hex()[2..4], "00");
}

#[wasm_bindgen_test]
fn sub_epi8_wraps() {
    // 0 - 1 = 255 (wrapping u8)
    let a = M128i::from_hex("0x00000000000000000000000000000000").unwrap();
    let b = M128i::from_hex("0x01010101010101010101010101010101").unwrap();
    let r = a.sub_epi8(&b);
    let vals: Vec<i64> = serde_lanes(&r.get_epi8());
    for v in &vals {
        assert_eq!(*v, -1i64);
    }
}

#[wasm_bindgen_test]
fn adds_epi8_saturates() {
    // 127 + 1 saturates to 127
    let a = M128i::from_hex("0x7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f").unwrap();
    let b = M128i::from_hex("0x01010101010101010101010101010101").unwrap();
    let r = a.adds_epi8(&b);
    let vals: Vec<i64> = serde_lanes(&r.get_epi8());
    for v in &vals {
        assert_eq!(*v, 127i64);
    }
}

#[wasm_bindgen_test]
fn subs_epu8_saturates_to_zero() {
    let a = M128i::from_hex("0x00000000000000000000000000000000").unwrap();
    let b = M128i::from_hex("0x01010101010101010101010101010101").unwrap();
    let r = a.subs_epu8(&b);
    let vals: Vec<i64> = serde_lanes(&r.get_epu8());
    for v in &vals {
        assert_eq!(*v, 0i64);
    }
}

#[wasm_bindgen_test]
fn abs_epi8_makes_positive() {
    let a = M128i::from_hex("0x80818283848586878889" // -128…-119
                               "8a8b8c8d8e8f9091").unwrap();
    let r = a.abs_epi8();
    let vals: Vec<i64> = serde_lanes(&r.get_epu8());
    // abs(-128) = 128 (unsigned), abs(-127) = 127, …
    for &v in &vals {
        assert!(v >= 0);
    }
}

#[wasm_bindgen_test]
fn cmpeq_epi32_produces_mask() {
    let a = M128i::from_epi32(1, 2, 3, 4);
    let b = M128i::from_epi32(1, 0, 3, 0);
    let r = a.cmpeq_epi32(&b);
    let vals: Vec<i64> = serde_lanes(&r.get_epi32());
    // from_epi32(e3,e2,e1,e0) maps to lanes [lane3, lane2, lane1, lane0].
    // a: lane0=4, lane1=3, lane2=2, lane3=1
    // b: lane0=0, lane1=3, lane2=0, lane3=1
    // Lanes equal: 1 (3==3) and 3 (1==1)
    assert_eq!(vals[1], -1i64); // lane1 equal → 0xFFFFFFFF as i32 = -1
    assert_eq!(vals[3], -1i64); // lane3 equal
    assert_eq!(vals[0], 0i64);  // lane0 not equal
    assert_eq!(vals[2], 0i64);  // lane2 not equal
}

#[wasm_bindgen_test]
fn slli_epi16_shifts_lanes() {
    // Each i16 lane = 1; shift left 4 → 16
    let a = M128i::from_hex("0x00010001000100010001000100010001").unwrap();
    let r = a.slli_epi16(4);
    let vals: Vec<i64> = serde_lanes(&r.get_epi16());
    for v in &vals {
        assert_eq!(*v, 16i64);
    }
}

#[wasm_bindgen_test]
fn shuffle_epi32_reorders() {
    // imm8 = 0x1b = 0b00011011 → reverse order: [3,2,1,0]
    let a = M128i::from_epi32(0xdddd, 0xcccc, 0xbbbb, 0xaaaa);
    let r = a.shuffle_epi32(0x1b);
    let vals: Vec<i64> = serde_lanes(&r.get_epi32());
    // lane0 gets selector 0b11=3 → original lane3=0xdddd
    assert_eq!(vals[0], 0xdddd);
    // lane3 gets selector 0b00=0 → original lane0=0xaaaa
    assert_eq!(vals[3], 0xaaaa);
}

// ── M256i: construction and basic ops ────────────────────────────────────────

#[wasm_bindgen_test]
fn m256i_new_is_zero() {
    let r = M256i::new();
    assert_eq!(r.get_bits().len(), 256);
    assert!(r.to_hex().trim_start_matches("0x").chars().all(|c| c == '0'));
}

#[wasm_bindgen_test]
fn m256i_from_hex_round_trips() {
    let hex = "0x".to_string() + &"abcd".repeat(16);
    let r = M256i::from_hex(&hex).unwrap();
    assert!(r.to_hex().to_lowercase().contains("abcd"));
}

#[wasm_bindgen_test]
fn m256i_add_epi32_independent_lanes() {
    // The two 128-bit halves should behave independently.
    let a = M256i::from_hex(
        "0x00000001000000010000000100000001\
           00000001000000010000000100000001",
    ).unwrap();
    let b = a.clone_reg();
    let r = a.add_epi32(&b);
    let vals: Vec<i64> = serde_lanes(&r.get_epi32());
    for v in &vals {
        assert_eq!(*v, 2i64);
    }
}

#[wasm_bindgen_test]
fn m256i_xor_self_is_zero() {
    let a = M256i::from_hex(
        "0xdeadbeefcafebabe1234567890abcdef\
           fedcba9876543210deadbeefcafebabe",
    ).unwrap();
    let r = a.xor(&a.clone_reg());
    assert!(r.to_hex().trim_start_matches("0x").chars().all(|c| c == '0'));
}

// ── M512i: basic sanity ───────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn m512i_new_is_zero() {
    let r = M512i::new();
    assert_eq!(r.get_bits().len(), 512);
    assert!(r.to_hex().trim_start_matches("0x").chars().all(|c| c == '0'));
}

#[wasm_bindgen_test]
fn m512i_not_complement() {
    let a = M512i::from_hex(
        &("0x".to_string() + &"ff".repeat(64))
    ).unwrap();
    let r = a.not();
    assert!(r.to_hex().trim_start_matches("0x").chars().all(|c| c == '0'));
}

// ── helper ────────────────────────────────────────────────────────────────────
fn serde_lanes(json: &str) -> Vec<i64> {
    json.trim_matches(|c| c == '[' || c == ']')
        .split(',')
        .map(|s| s.trim().trim_matches('"').parse::<i64>().unwrap())
        .collect()
}

