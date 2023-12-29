use super::Color;
use crate::nbt::{encode_str, encode_str_unchecked, len_str, COMPOUND, END, LIST, STRING};
use crate::{UnsafeWriter, Write};
#[macro_export]
macro_rules! const_literal {
    ($($e:expr),* $(,)?) => {
        ::core::concat!("{\"text\":\"" $(,$e)?, "\"}")
    };
}

#[derive(Clone, Copy)]
pub struct Literal<'a>(pub &'a str, pub Option<Color>);

impl Write for Literal<'_> {
    fn write(&self, w: &mut UnsafeWriter) {
        if let Some(color) = self.1 {
            w.write_byte(COMPOUND);
            w.write_byte(STRING);
            encode_str_unchecked("text", w);
            encode_str(self.0, w);
            w.write_byte(STRING);
            encode_str_unchecked("color", w);
            w.write(b"\x00\x07#");
            w.write(&color.to_hex());
            w.write_byte(END);
        } else {
            w.write_byte(STRING);
            encode_str(self.0, w);
        }
    }

    fn len(&self) -> usize {
        len_str(self.0) + if self.1.is_some() { 26 } else { 1 }
    }
}

#[derive(Clone, Copy)]
pub struct Translate<'a>(pub &'a str, pub &'a [&'a str], pub Option<Color>);

impl Write for Translate<'_> {
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(COMPOUND);
        w.write_byte(STRING);
        encode_str_unchecked("translate", w);
        encode_str_unchecked(self.0, w);
        if !self.1.is_empty() {
            w.write_byte(LIST);
            encode_str_unchecked("with", w);
            w.write_byte(STRING);
            (self.1.len() as u32).write(w);
            for x in self.1 {
                encode_str(x, w);
            }
        }
        if let Some(color) = self.2 {
            w.write_byte(STRING);
            encode_str_unchecked("color", w);
            w.write(b"\x00\x07#");
            w.write(&color.to_hex());
        }
        w.write_byte(END);
    }

    fn len(&self) -> usize {
        let mut l = 7 + 9;
        l += self.0.len();
        if !self.1.is_empty() {
            l += 12;
            for x in self.1 {
                l += len_str(x);
            }
        }
        if self.2.is_some() {
            l += 17;
        }
        l
    }
}
