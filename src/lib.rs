mod utils;

use std::fmt::{self, Write};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct M128i {
    epi8: [u8; 16],
}

#[wasm_bindgen]
impl M128i {
    pub fn new() -> M128i {
        utils::set_panic_hook();
        let mut epi8 = [0 as u8; 16];
        for i in 0..16 {
            epi8[i] += i as u8;
        }
        M128i { epi8 }
    }

    pub fn add_one(&mut self) {
        for i in 0..16 {
            self.epi8[i] += 1;
        }
    }
    pub fn minus_one(&mut self) {
        for i in 0..16 {
            self.epi8[i] -= 1;
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn print_hex(&self) -> String {
        let mut s = String::new();
        write!(s, "0x").unwrap_throw();
        for i in (0..16).rev() {
            write!(s, "{:02x}", self.epi8[i]).unwrap_throw();
        }
        s
    }
}

impl fmt::Display for M128i {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in (0..16).rev() {
            let epi = self.epi8[i];
            for j in (0..8).rev() {
                let bit = (epi >> j) & 0x1;
                let symbol = if bit == 0x0 { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }

            if i % 4 == 0 {
                write!(f, "\n")?;
            } else {
                write!(f, " ")?;
            }
        }

        Ok(())
    }
}
