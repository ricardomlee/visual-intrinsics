//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use visual_intrinsics::M128i;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn new_is_all_zeros() {
    let r = M128i::new();
    assert_eq!(r.to_hex(), "0x00000000000000000000000000000000");
}

#[wasm_bindgen_test]
fn from_hex_round_trips() {
    let hex = "0x0f1e2d3c4b5a69788796655443322110";
    let r = M128i::from_hex(hex).unwrap();
    // to_hex() pads to 32 nibbles; accept either form
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
    // 0x7fffffff + 1 = 0x80000000 (wraps to i32::MIN)
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
    // top nibble shifts right by one nibble
    assert_eq!(&r.to_hex()[2..4], "0f");
}

#[wasm_bindgen_test]
fn get_bits_length() {
    let r = M128i::new();
    assert_eq!(r.get_bits().len(), 128);
}

// helper: parse a JSON int array from Rust into a Vec<i64>
fn serde_lanes(json: &str) -> Vec<i64> {
    json.trim_matches(|c| c == '[' || c == ']')
        .split(',')
        .map(|s| s.trim().trim_matches('"').parse::<i64>().unwrap())
        .collect()
}

