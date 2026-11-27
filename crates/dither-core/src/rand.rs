//! Deterministic seeding for avatars: FNV-1a over UTF-16 code units, then xorshift32.

/// 32-bit FNV-1a over a string's UTF-16 code units, matching JS `charCodeAt`.
pub fn fnv1a(s: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for unit in s.encode_utf16() {
        h ^= unit as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// Xorshift32; `next_f64` returns floats in `[0, 1)`.
#[derive(Clone, Copy, Debug)]
pub struct XorShift32 {
    state: u32,
}

impl XorShift32 {
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x9e37_79b9 } else { seed },
        }
    }

    pub fn next_f64(&mut self) -> f64 {
        let mut s = self.state;
        s ^= s.wrapping_shl(13);
        s ^= s >> 17;
        s ^= s.wrapping_shl(5);
        self.state = s;
        s as f64 / 4_294_967_296.0
    }
}
