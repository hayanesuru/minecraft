use crate::{Error, cold_path};

pub struct Reader<'a> {
    buf: &'a [u8],
}

impl<'a> Reader<'a> {
    pub const fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    pub fn read(&mut self) -> Result<u8, Error> {
        match self.buf {
            [a, rest @ ..] => {
                self.buf = rest;
                Ok(*a)
            }
            _ => {
                cold_path();
                Err(Error)
            }
        }
    }

    pub fn read_array<const L: usize>(&mut self) -> Result<[u8; L], Error> {
        match self.buf.get(L..) {
            Some(rest) => unsafe {
                let a = *(self.buf.as_ptr() as *const [u8; L]);
                self.buf = rest;
                Ok(a)
            },
            None => {
                cold_path();
                Err(Error)
            }
        }
    }

    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8], Error> {
        match self.buf.split_at_checked(len) {
            Some((a, rest)) => {
                self.buf = rest;
                Ok(a)
            }
            None => {
                cold_path();
                Err(Error)
            }
        }
    }

    pub fn peek(&self) -> Result<u8, Error> {
        match self.buf {
            [a, ..] => Ok(*a),
            _ => {
                cold_path();
                Err(Error)
            }
        }
    }

    /// # Safety
    pub unsafe fn advance(&mut self, len: usize) {
        self.buf = unsafe { self.buf.get_unchecked(len..) };
    }

    pub fn peek_array<const L: usize>(&self) -> Result<[u8; L], Error> {
        if L <= self.buf.len() {
            unsafe { Ok(*(self.buf.as_ptr() as *const [u8; L])) }
        } else {
            cold_path();
            Err(Error)
        }
    }

    pub fn peek_slice(&self, len: usize) -> Result<&[u8], Error> {
        match self.buf.get(0..len) {
            Some(s) => Ok(s),
            None => {
                cold_path();
                Err(Error)
            }
        }
    }

    pub const fn len(&self) -> usize {
        self.buf.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}
