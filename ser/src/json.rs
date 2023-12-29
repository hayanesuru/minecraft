use crate::{u8_to_hex, UnsafeWriter, Write};

const B: u8 = b'b'; // \x08
const T: u8 = b't'; // \x09
const N: u8 = b'n'; // \x0a
const F: u8 = b'f'; // \x0c
const R: u8 = b'r'; // \x0d
const Q: u8 = b'"'; // \x22
const S: u8 = b'\\'; // \x5c
const U: u8 = 0xff; // non-printable

const ESCAPE: [u8; 256] = [
    U, U, U, U, U, U, U, U, B, T, N, U, F, R, U, U, U, U, U, U, U, U, U, U, U, U, U, U, U, U, U, U,
    1, 1, Q, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, S, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
];

pub fn json_str_escape(buf: &mut String, b: &[u8]) {
    let e = JsonStr(b);
    let wlen = e.len();
    buf.reserve(wlen);
    unsafe {
        e.write(&mut UnsafeWriter(buf.as_mut_ptr().add(buf.len())));
        let len = buf.len() + wlen;
        buf.as_mut_vec().set_len(len);
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct JsonStr<'a>(&'a [u8]);

impl Write for JsonStr<'_> {
    fn write(&self, w: &mut UnsafeWriter) {
        let mut start = 0;
        let mut cur = 0;
        unsafe {
            while let Some(&byte) = self.0.get(cur) {
                let esc = *ESCAPE.get_unchecked(byte as usize);
                if esc <= 4 {
                    cur += esc as usize;
                    continue;
                }
                w.write(&self.0[start..cur]);
                if esc == U {
                    let (d1, d2) = u8_to_hex(byte);
                    w.write(&[b'\\', b'u', b'0', b'0', d1, d2]);
                } else {
                    w.write(&[b'\\', esc]);
                }
                cur += 1;
                start = cur;
            }
            w.write(self.0.get_unchecked(start..));
        }
    }

    fn len(&self) -> usize {
        let mut cur = 0usize;
        let mut len = 0usize;
        unsafe {
            while let Some(&byte) = self.0.get(cur) {
                let esc = *ESCAPE.get_unchecked(byte as usize);
                if esc <= 4 {
                    cur += esc as usize;
                    continue;
                }

                if esc == U {
                    len += 5;
                } else {
                    len += 1;
                }
                cur += 1;
            }
        }
        self.0.len() + len
    }
}
