use alloc::boxed::Box;
use mser::{Read, V32, Write};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MessageSignature {
    pub bytes: [u8; 256],
}

#[derive(Clone)]
pub enum MessageSignaturePacked {
    FullSignature(Box<MessageSignature>),
    Index(u32),
}

impl Write for MessageSignaturePacked {
    unsafe fn write(&self, w: &mut mser::Writer) {
        unsafe {
            match self {
                Self::FullSignature(message_signature) => {
                    V32(0).write(w);
                    message_signature.write(w);
                }
                Self::Index(x) => {
                    V32(*x + 1).write(w);
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::FullSignature(message_signature) => V32(0).len_s() + message_signature.len_s(),
            Self::Index(x) => V32(*x + 1).len_s(),
        }
    }
}

impl<'a> Read<'a> for MessageSignaturePacked {
    fn read(buf: &mut mser::Reader<'a>) -> Result<Self, mser::Error> {
        let id = V32::read(buf)?.0;
        Ok(if id == 0 {
            Self::FullSignature(Box::new(MessageSignature::read(buf)?))
        } else {
            Self::Index(id)
        })
    }
}
