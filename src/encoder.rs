pub fn encode_fill() -> Vec<u16> {
    vec![0x00]
}

pub fn encode_blkw(c: &u16) -> Vec<u16> {
    vec![0x00; *c as usize]
}

pub fn encode_stringz(s: &str) -> Vec<u16> {
    let mut bin: Vec<_> = s.bytes().map(|e| e as u16).collect();
    bin.push(0); // Null-terminated strings
    bin
}

pub fn encode_orig(origin: &u16) -> Vec<u16> {
    vec![*origin]
}
