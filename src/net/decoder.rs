use ser::Write;

use crate::cfb8::decode;
use crate::{UnsafeWriter, V21};
use core::ptr::copy_nonoverlapping;

pub struct PacketDecoder {
    pub buf: Vec<u8>,
    zlib: Option<Decompressor>,
    cipher: Option<([u32; 44], [u8; 16])>,
    n: usize,
    m: usize,
}

impl PacketDecoder {
    #[inline]
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            zlib: None,
            buf,
            n: 0,
            m: 0,
            cipher: None,
        }
    }

    #[inline]
    pub fn enable_compress(&mut self) {
        self.zlib = Some(Decompressor::new());
    }

    #[inline]
    pub fn enable_crypt(&mut self, key: [u8; 16]) {
        self.cipher = Some((crate::cfb8::key(key), key));
        self.m = self.buf.len();
    }

    #[inline]
    pub fn try_clear_direct(&mut self) {
        if self.n == self.buf.len() {
            self.n = 0;
            self.m = 0;
            self.buf.clear();
        }
    }

    #[inline]
    pub fn try_clear(&mut self) {
        self.buf.truncate(self.m);
        self.try_clear_direct();
    }

    #[inline]
    pub fn cipher(&self) -> bool {
        self.cipher.is_some()
    }

    pub fn decode_direct(&mut self) -> Option<&[u8]> {
        if !self.cipher() {
            self.m = self.buf.len();
        }
        let mut start = self.n;
        let frame = match *unsafe { self.buf.get_unchecked(self.n..) } {
            [a, ..] if (a & 0x80) == 0 => {
                start += 1;
                usize::from(a)
            }
            [a, b, ..] if (b & 0x80) == 0 => {
                start += 2;
                usize::from(a & 0x7F) | usize::from(b) << 7
            }
            [a, b, c, ..] if (c & 0x80) == 0 => {
                start += 3;
                usize::from(a & 0x7F) | usize::from(b & 0x7F) << 7 | usize::from(c) << 14
            }
            [_, _, _, ..] => return Some(&[]), // packet len > 2097151
            _ => return None,
        };
        let end = start + frame;
        if self.m < end {
            return None;
        }
        self.n = end;

        if self.zlib.is_none() {
            unsafe { return Some(self.buf.get_unchecked(start..end)) }
        }

        let uncompressed = match unsafe { self.buf.get_unchecked(start..end) } {
            [a, ..] if (a & 0x80) == 0 => {
                start += 1;
                *a as usize
            }
            [a, b, ..] if (b & 0x80) == 0 => {
                start += 2;
                *a as usize & 0x7F | (*b as usize) << 7
            }
            [a, b, c, ..] if (c & 0x80) == 0 => {
                start += 3;
                *a as usize & 0x7F | ((*b & 0x7F) as usize) << 7 | (*c as usize) << 14
            }
            _ => return Some(&[]),
        };
        if uncompressed == 0 {
            unsafe { return Some(self.buf.get_unchecked(start..end)) }
        }
        None
    }

    pub fn has_next(&self) -> bool {
        if self.buf.is_empty() {
            return false;
        }
        if self.cipher.is_some() {
            return true;
        }
        let mut start = self.n;
        let frame = match *unsafe { self.buf.get_unchecked(start..) } {
            [a, ..] if (a & 0x80) == 0 => {
                start += 1;
                usize::from(a)
            }
            [a, b, ..] if (b & 0x80) == 0 => {
                start += 2;
                usize::from(a & 0x7F) | usize::from(b) << 7
            }
            [a, b, c, ..] if (c & 0x80) == 0 => {
                start += 3;
                usize::from(a & 0x7F) | usize::from(b & 0x7F) << 7 | usize::from(c) << 14
            }
            [_, _, _, ..] => return true, // packet len > 2097151
            _ => return false,
        };
        let end = start + frame;
        self.buf.len() >= end
    }

    fn decrypt(&mut self) {
        if self.m >= self.buf.len() {
            return;
        }
        if let Some((key, iv)) = &mut self.cipher {
            let buf = unsafe { self.buf.get_unchecked_mut(self.m..) };
            decode(key, iv, buf);
        }
        self.m = self.buf.len();
    }

    pub fn decode(&mut self) -> bool {
        self.decrypt();
        loop {
            let mut start = self.n;
            let frame = match *unsafe { self.buf.get_unchecked(self.n..self.m) } {
                [a, ..] if (a & 0x80) == 0 => {
                    start += 1;
                    usize::from(a)
                }
                [a, b, ..] if (b & 0x80) == 0 => {
                    start += 2;
                    usize::from(a & 0x7F) | usize::from(b) << 7
                }
                [a, b, c, ..] if (c & 0x80) == 0 => {
                    start += 3;
                    usize::from(a & 0x7F) | usize::from(b & 0x7F) << 7 | usize::from(c) << 14
                }
                [_, _, _, ..] => return false, // packet len > 2097151
                _ => return true,
            };
            let end = start + frame;
            if self.m < end {
                return true;
            }
            self.n = end;

            let zlib = match &mut self.zlib {
                Some(x) => x,
                None => unsafe {
                    let clen = end - start;
                    let frame = V21(clen as u32);
                    let len_all = frame.len() + clen;
                    self.buf.reserve(len_all);
                    frame.write(&mut UnsafeWriter(self.buf.as_mut_ptr().add(self.buf.len())));
                    copy_nonoverlapping(
                        self.buf.as_ptr().add(start),
                        self.buf.as_mut_ptr().add(self.buf.len() + frame.len()),
                        clen,
                    );
                    self.buf.set_len(self.buf.len() + len_all);
                    continue;
                },
            };

            let uncompressed = match unsafe { self.buf.get_unchecked(start..end) } {
                [a, ..] if (a & 0x80) == 0 => {
                    start += 1;
                    *a as usize
                }
                [a, b, ..] if (b & 0x80) == 0 => {
                    start += 2;
                    *a as usize & 0x7F | (*b as usize) << 7
                }
                [a, b, c, ..] if (c & 0x80) == 0 => {
                    start += 3;
                    *a as usize & 0x7F | ((*b & 0x7F) as usize) << 7 | (*c as usize) << 14
                }
                _ => return false,
            };

            let clen = end - start;
            if uncompressed == 0 {
                unsafe {
                    let frame = V21(clen as u32);
                    let len_all = frame.len() + clen;
                    self.buf.reserve(len_all);
                    frame.write(&mut UnsafeWriter(self.buf.as_mut_ptr().add(self.buf.len())));
                    copy_nonoverlapping(
                        self.buf.as_ptr().add(start),
                        self.buf.as_mut_ptr().add(self.buf.len() + frame.len()),
                        clen,
                    );
                    self.buf.set_len(self.buf.len() + len_all);
                    continue;
                }
            }

            let frame = V21(uncompressed as u32);
            let len_all = frame.len() + uncompressed;
            self.buf.reserve(len_all);
            unsafe {
                frame.write(&mut UnsafeWriter(self.buf.as_mut_ptr().add(self.buf.len())));
                self.buf.set_len(self.buf.len() + frame.len());
            }

            unsafe {
                self.buf.reserve(uncompressed);
                let len = zlib.decompress(
                    self.buf.as_ptr().add(start),
                    clen,
                    self.buf.as_mut_ptr().add(self.buf.len()),
                    self.buf.capacity() - self.buf.len(),
                );

                if len != uncompressed {
                    continue;
                } else {
                    self.buf.set_len(self.buf.len() + len);
                }
            }
        }
    }

    pub fn decoded(&self) -> &[u8] {
        unsafe { self.buf.get_unchecked(self.m..) }
    }
}

struct Decompressor {
    n: miniz_oxide::inflate::core::DecompressorOxide,
}

const FLAGS: u32 = miniz_oxide::inflate::core::inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
    | miniz_oxide::inflate::core::inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;

impl Decompressor {
    #[inline]
    fn new() -> Self {
        Self {
            n: miniz_oxide::inflate::core::DecompressorOxide::new(),
        }
    }

    unsafe fn decompress(
        &mut self,
        oin: *const u8,
        inlen: usize,
        oout: *mut u8,
        outlen: usize,
    ) -> usize {
        if inlen == 0 || outlen == 0 {
            return 0;
        }
        let mut xin = unsafe { core::slice::from_raw_parts(oin, inlen) };
        let mut xout = unsafe { core::slice::from_raw_parts_mut(oout, outlen) };

        self.n.init();
        let (status, in_consumed, out_consumed) =
            miniz_oxide::inflate::core::decompress(&mut self.n, xin, xout, 0, FLAGS);
        unsafe {
            xin = xin.get_unchecked(in_consumed..xin.len());
            xout = xout.get_unchecked_mut(out_consumed..xout.len());
        }
        match status {
            miniz_oxide::inflate::TINFLStatus::Done => {
                if xin.is_empty() {
                    outlen - xout.len()
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}
