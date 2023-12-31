use crate::UnsafeWriter;
use alloc::string::String;
use alloc::vec::Vec;

const CHAR_WIDTH: &[u8; 256] = &[
    // 1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F
];

#[must_use]
pub fn is_valid(bytes: &[u8]) -> bool {
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        let w = unsafe { *CHAR_WIDTH.get_unchecked(byte as usize) };
        if w == 0 {
            return false;
        }
        index += w as usize;
    }
    true
}

pub fn len(str: &str) -> usize {
    let mut l = 0;
    let bytes = str.as_bytes();
    let mut index = 0;

    while let Some(&byte) = bytes.get(index) {
        let w = unsafe { *CHAR_WIDTH.get_unchecked(byte as usize) };
        index += w as usize;
        if w == 0 {
            if byte == 0 {
                l += 2 - 1;
                index += 1;
            } else {
                l += 6 - 4;
                index += 4;
            }
        }
    }
    l + index
}

#[inline(never)]
pub fn encode(str: &str, w: &mut UnsafeWriter) {
    let bytes = str.as_bytes();
    let mut index = 0;
    let mut start = 0;

    unsafe {
        while let Some(&byte) = bytes.get(index) {
            let x = *CHAR_WIDTH.get_unchecked(byte as usize);
            index += x as usize;
            if x != 0 {
                continue;
            }
            if byte == 0 {
                w.write(bytes.get_unchecked(start..index));
                w.write(&[0xc0, 0x80]);
                index += 1;
                start = index;
            } else {
                let code_point = core::str::from_utf8_unchecked(&bytes[index..index + 4])
                    .chars()
                    .next()
                    .unwrap() as u32;
                let code_point = code_point - 0x10000;
                let first = ((code_point >> 10) as u16) | 0xD800;
                let second = ((code_point & 0x3FF) as u16) | 0xDC00;

                w.write(bytes.get_unchecked(start..index));
                w.write(&[
                    0xE0 | ((first & 0xF000) >> 12) as u8,
                    0x80 | ((first & 0xFC0) >> 6) as u8,
                    0x80 | ((first & 0x3F) as u8),
                    0xE0 | ((second & 0xF000) >> 12) as u8,
                    0x80 | ((second & 0xFC0) >> 6) as u8,
                    0x80 | (second & 0x3F) as u8,
                ]);
                index += 4;
                start = index;
            }
        }
        w.write(bytes.get_unchecked(start..index));
    }
}

#[inline(never)]
pub fn decode(bytes: &[u8]) -> Option<String> {
    let mut buf = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut start = 0;

    while let Some(&byte) = bytes.get(index) {
        match byte {
            0x00..=0x7F => index += 1,
            0xC2..=0xDF => unsafe {
                let sec = match bytes.get(index + 1) {
                    Some(&byte) => byte,
                    _ => return None,
                };

                if !(byte == 0xC0 && sec == 0x80) {
                    index += 2;
                } else {
                    buf.extend(bytes.get_unchecked(start..index));
                    buf.push(0);
                    index += 2;
                    start = index;
                }
            },
            0xE0..=0xEF => {
                let sec = match bytes.get(index + 1) {
                    Some(&byte) if byte & 0xC0 == 0x80 => byte,
                    _ => return None,
                };
                let third = match bytes.get(index + 2) {
                    Some(&byte) if byte & 0xC0 == 0x80 => byte,
                    _ => return None,
                };
                match (byte, sec) {
                    (0xE0, 0xA0..=0xBF)
                    | (0xE1..=0xEC | 0xEE | 0xEF, 0x80..=0xBF)
                    | (0xED, 0x80..=0x9F) => {
                        index += 3;
                    }
                    (0xED, 0xA0..=0xAF) => {
                        match bytes.get(index + 3) {
                            Some(0xED) => (),
                            _ => return None,
                        };
                        let fifth = match bytes.get(index + 4) {
                            Some(&x @ 0xB0..=0xBF) => x & 0x3F,
                            _ => return None,
                        };
                        let sixth = match bytes.get(index + 5) {
                            Some(&x) if x & 0xC0 == 0x80 => x & 0x3F,
                            _ => return None,
                        };
                        let s1 = 0xD000 | u32::from(sec & 0x3F) << 6 | u32::from(third & 0x3F);
                        let s2 = 0xD000 | u32::from(fifth) << 6 | u32::from(sixth);
                        let point = 0x10000 + ((s1 - 0xD800) << 10 | (s2 - 0xDC00));
                        buf.extend([
                            0xF0 | ((point & 0x1C0000) >> 18) as u8,
                            0x80 | ((point & 0x3F000) >> 12) as u8,
                            0x80 | ((point & 0xFC0) >> 6) as u8,
                            0x80 | (point & 0x3F) as u8,
                        ]);
                    }
                    _ => return None,
                }
            }
            _ => return None,
        }
    }

    unsafe {
        buf.extend(bytes.get_unchecked(start..index));
        buf.shrink_to_fit();
        Some(String::from_utf8_unchecked(buf))
    }
}
