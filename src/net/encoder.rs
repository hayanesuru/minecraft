use crate::{UnsafeWriter, Write, V21};

pub struct PacketEncoder {
    pub buf: Vec<u8>,
    n: usize,
    zlib: Option<Compressor>,
    threshold: usize,
    cipher: Option<([u32; 44], [u8; 16])>,
}

impl PacketEncoder {
    #[inline]
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            zlib: None,
            cipher: None,
            buf,
            threshold: 0,
            n: 0,
        }
    }

    #[inline]
    pub fn enable_compress(&mut self, threshold: usize) {
        self.zlib = Some(Compressor::new(4));
        self.threshold = threshold;
    }

    #[inline]
    pub fn enable_crypt(&mut self, key: [u8; 16]) {
        self.cipher = Some((crate::cfb8::key(key), key));
    }

    #[inline]
    pub fn cipher(&self) -> bool {
        self.cipher.is_some()
    }

    pub fn write(&mut self, packet: &[u8]) {
        debug_assert!(!packet.is_empty());

        if self.zlib.is_none() || packet.len() < 0x100 {
            self.write_direct(packet);
            return;
        }

        let c = unsafe { self.zlib.as_mut().unwrap_unchecked() };
        let start = self.buf.len();
        self.buf.extend([0; 6]);

        unsafe {
            let mut x = c.compress(
                packet.as_ptr(),
                packet.len(),
                self.buf.as_mut_ptr().add(self.buf.len()),
                self.buf.capacity() - self.buf.len(),
            );
            if x == 0 {
                self.buf.reserve(1024 + packet.len());
                x = c.compress(
                    packet.as_ptr(),
                    packet.len(),
                    self.buf.as_mut_ptr().add(self.buf.len()),
                    self.buf.capacity() - self.buf.len(),
                );
            }
            if x == 0 {
                return;
            }
            self.buf.set_len(self.buf.len() + x);

            let frame = self.buf.len() - start - 3;
            let uncompressed = packet.len();

            let [a, b, c] = V21(frame as u32).to_array();
            let [d, e, f] = V21(uncompressed as u32).to_array();
            UnsafeWriter(self.buf.as_mut_ptr().add(start)).write(&[a, b, c, d, e, f]);
        }
    }

    pub fn write_direct(&mut self, packet: &[u8]) -> bool {
        debug_assert!(!packet.is_empty());

        let frame = if self.zlib.is_some() {
            if packet.len() < self.threshold {
                V21(packet.len() as u32 + 1)
            } else {
                return false;
            }
        } else {
            V21(packet.len() as u32)
        };

        let len = frame.len() + frame.0 as usize;
        self.buf.reserve(len);
        unsafe {
            let mut writer = UnsafeWriter(self.buf.as_mut_ptr().add(self.buf.len()));
            frame.write(&mut writer);
            if frame.0 != packet.len() as u32 {
                writer.write_byte(0);
            }
            writer.write(packet);
            self.buf.set_len(self.buf.len() + len);
        }
        true
    }

    pub fn encrypt(&mut self) {
        let (key, iv) = match &mut self.cipher {
            None => return,
            Some(x) => x,
        };
        crate::cfb8::encode(key, iv, &mut self.buf);
    }

    pub fn flush(&mut self, n: usize) -> &[u8] {
        self.n += n;
        if self.n == self.buf.len() {
            self.buf.clear();
            self.n = 0;
        }
        unsafe { self.buf.get_unchecked(self.n..) }
    }
}

struct Compressor {
    n: miniz_oxide::deflate::core::CompressorOxide,
}

impl Compressor {
    #[inline]
    fn new(level: u8) -> Self {
        let flags = miniz_oxide::deflate::core::create_comp_flags_from_zip_params(
            level as i32,
            miniz_oxide::DataFormat::Zlib.to_window_bits(),
            miniz_oxide::deflate::core::CompressionStrategy::RLE as i32,
        );
        Self {
            n: miniz_oxide::deflate::core::CompressorOxide::new(flags),
        }
    }

    unsafe fn compress(
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
        self.n.reset();

        let (status, bytes_in, bytes_out) = miniz_oxide::deflate::core::compress(
            &mut self.n,
            xin,
            xout,
            miniz_oxide::deflate::core::TDEFLFlush::Finish,
        );
        unsafe {
            xin = xin.get_unchecked(bytes_in..xin.len());
            xout = xout.get_unchecked_mut(bytes_out..xout.len());
        }
        match status {
            miniz_oxide::deflate::core::TDEFLStatus::Done
            | miniz_oxide::deflate::core::TDEFLStatus::Okay => {
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
