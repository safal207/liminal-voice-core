#[allow(dead_code)]
pub fn normalize_text(text: &str) -> String {
    text.trim().to_lowercase()
}

#[allow(dead_code)]
pub fn hash01(s: &str) -> (f32, f32) {
    let mut h: u64 = 0xcbf2_29d1_821f_1cfd; // 1469598103934665603 FNV offset basis
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }

    let a = ((h >> 11) & 0xFFFF) as f32 / 65535.0;
    let b = ((h >> 27) & 0xFFFF) as f32 / 65535.0;
    (a, b)
}
