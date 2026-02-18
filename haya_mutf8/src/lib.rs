#![no_std]

use mser::{Error, UnsafeWriter};

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
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F
];

#[must_use]
pub const fn as_mutf8_ascii(bytes: &[u8]) -> Option<&str> {
    if !contains_zero_or_nonascii(bytes) {
        unsafe { Some(core::str::from_utf8_unchecked(bytes)) }
    } else {
        None
    }
}

const fn contains_zero_or_nonascii(bytes: &[u8]) -> bool {
    let mut i = 0;
    let mut flag = false;
    while i < bytes.len() {
        let x = bytes[i];
        flag |= x > 127 || x == 0;
        i += 1;
    }
    flag
}

pub const fn encode_mutf8_len(bytes: &str) -> usize {
    let mut l = 0;
    let mut index = 0;
    let bytes = bytes.as_bytes();
    while index < bytes.len() {
        let byte = bytes[index];
        let w = unsafe { *CHAR_WIDTH.as_ptr().add(byte as usize) };
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

#[derive(Clone, Copy)]
pub struct Mutf8<'a>(&'a [u8]);

impl<'a> Mutf8<'a> {
    /// # Safety
    ///
    /// `n` must be valid mutf-8.
    pub const fn new_unchecked(n: &'a [u8]) -> Self {
        Self(n)
    }
}

pub fn encode_mutf8(s: &str, w: &mut UnsafeWriter) {
    let mut index = 0;
    let mut start = 0;
    let bytes = s.as_bytes();
    while let Some(&byte) = bytes.get(index) {
        let x = unsafe { *CHAR_WIDTH.get_unchecked(byte as usize) };
        index += x as usize;
        if x != 0 {
            continue;
        }
        if byte == 0 {
            unsafe {
                w.write(bytes.get_unchecked(start..index));
                w.write(&[0xc0, 0x80]);
            }
            index += 1;
            start = index;
        } else {
            let b2 = unsafe { *bytes.get_unchecked(index + 1) };
            let b3 = unsafe { *bytes.get_unchecked(index + 2) };
            let b4 = unsafe { *bytes.get_unchecked(index + 3) };
            let mut bits: u32 = ((byte as u32) & 0x07) << 18;
            bits += ((b2 as u32) & 0x3F) << 12;
            bits += ((b3 as u32) & 0x3F) << 6;
            bits += (b4 as u32) & 0x3F;
            unsafe {
                w.write(bytes.get_unchecked(start..index));
                w.write(&[
                    0xED,
                    (0xA0 + (((bits >> 16) - 1) & 0x0F)) as u8,
                    (0x80 + ((bits >> 10) & 0x3F)) as u8,
                    0xED,
                    (0xB0 + ((bits >> 6) & 0x0F)) as u8,
                    b4,
                ]);
            }
            index += 4;
            start = index;
        }
    }
    unsafe { w.write(bytes.get_unchecked(start..index)) }
}

pub fn decode_mutf8_len(mut bytes: &[u8]) -> Result<usize, Error> {
    let mut len = 0usize;

    while let [byte, ref rest @ ..] = bytes[..] {
        bytes = rest;
        match byte {
            0x01..=0x7F => len += 1,
            0xC0 => {
                if let [sec, ref rest @ ..] = bytes[..] {
                    bytes = rest;
                    if sec == 0x80 {
                        len += 1;
                    } else {
                        return Err(Error);
                    }
                } else {
                    return Err(Error);
                }
            }
            0xC2..=0xDF => {
                if let [sec, ref rest @ ..] = bytes[..] {
                    bytes = rest;
                    if sec as i8 >= -64 {
                        return Err(Error);
                    }
                    len += 2;
                } else {
                    return Err(Error);
                }
            }
            0xE0..=0xEF => match bytes[..] {
                [sec, third, ref rest @ ..] => {
                    bytes = rest;
                    if sec & 0xC0 != 0x80 || third & 0xC0 != 0x80 {
                        return Err(Error);
                    }
                    match (byte, sec) {
                        (0xE0, 0xA0..=0xBF)
                        | (0xE1..=0xEC | 0xEE | 0xEF, 0x80..=0xBF)
                        | (0xED, 0x80..=0x9F) => {
                            len += 3;
                        }
                        (0xED, 0xA0..=0xAF) => match bytes[..] {
                            [fourth, fifth, sixth, ref rest @ ..] => {
                                bytes = rest;
                                if fourth != 0xED {
                                    return Err(Error);
                                }
                                match fifth {
                                    0xB0..=0xBF => (),
                                    _ => return Err(Error),
                                }
                                if sixth & 0xC0 != 0x80 {
                                    return Err(Error);
                                }
                                len += 4;
                            }
                            _ => return Err(Error),
                        },
                        _ => return Err(Error),
                    }
                }
                _ => return Err(Error),
            },
            _ => return Err(Error),
        }
    }
    Ok(len)
}

/// # Safety
/// `w` is valid for write
pub unsafe fn decode_mutf8(Mutf8(bytes): Mutf8, w: &mut UnsafeWriter) -> Result<(), Error> {
    let mut index = 0;
    let mut start = 0;

    while let Some(&byte) = bytes.get(index) {
        match byte {
            0x01..=0x7F => index += 1,
            0xC0 => unsafe {
                let sec = match bytes.get(index + 1) {
                    Some(&byte) => byte,
                    _ => return Err(Error),
                };
                if sec == 0x80 {
                    w.write(bytes.get_unchecked(start..index));
                    w.write_byte(b'\0');
                    index += 2;
                    start = index;
                } else {
                    return Err(Error);
                }
            },
            0xC2..=0xDF => {
                let sec = match bytes.get(index + 1) {
                    Some(&byte) => byte,
                    _ => return Err(Error),
                };
                if sec as i8 >= -64 {
                    return Err(Error);
                }
                index += 2;
            }
            0xE0..=0xEF => unsafe {
                let sec = match bytes.get(index + 1) {
                    Some(&byte) if byte & 0xC0 == 0x80 => byte,
                    _ => return Err(Error),
                };
                let third = match bytes.get(index + 2) {
                    Some(&byte) if byte & 0xC0 == 0x80 => byte,
                    _ => return Err(Error),
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
                            _ => return Err(Error),
                        };
                        let fifth = match bytes.get(index + 4) {
                            Some(&x @ 0xB0..=0xBF) => x & 0x3F,
                            _ => return Err(Error),
                        };
                        let sixth = match bytes.get(index + 5) {
                            Some(&x) if x & 0xC0 == 0x80 => x & 0x3F,
                            _ => return Err(Error),
                        };
                        let s1 = 0xD000 | (u32::from(sec & 0x3F) << 6) | u32::from(third & 0x3F);
                        let s2 = 0xD000 | (u32::from(fifth) << 6) | u32::from(sixth);
                        let point = 0x10000 + (((s1 - 0xD800) << 10) | (s2 - 0xDC00));
                        w.write(bytes.get_unchecked(start..index));
                        w.write(&[
                            0xF0 | ((point & 0x1C0000) >> 18) as u8,
                            0x80 | ((point & 0x3F000) >> 12) as u8,
                            0x80 | ((point & 0xFC0) >> 6) as u8,
                            0x80 | (point & 0x3F) as u8,
                        ]);
                        index += 6;
                        start = index;
                    }
                    _ => return Err(Error),
                }
            },
            _ => return Err(Error),
        }
    }

    unsafe { w.write(bytes.get_unchecked(start..index)) }

    Ok(())
}
