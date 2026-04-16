use crate::registry::ChatTypeRef;
use crate::{Component, FixedByteArray, Holder, Style};
use haya_collection::List;
use mser::{Read, Utf8, V32, Write};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MessageSignature<'a> {
    pub bytes: FixedByteArray<'a, 256>,
}

#[derive(Clone)]
pub enum MessageSignaturePacked<'a> {
    FullSignature(MessageSignature<'a>),
    Index(u32),
}

impl<'a> Write for MessageSignaturePacked<'a> {
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

impl<'a> Read<'a> for MessageSignaturePacked<'a> {
    fn read(buf: &mut mser::Reader<'a>) -> Result<Self, mser::Error> {
        let id = V32::read(buf)?.0;
        Ok(if id == 0 {
            Self::FullSignature(MessageSignature::read(buf)?)
        } else {
            Self::Index(id)
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Bound<'a> {
    pub chat_type: Holder<ChatType<'a>, ChatTypeRef>,
    pub name: Component,
    pub target_name: Option<Component>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatType<'a> {
    pub chat: ChatTypeDecoration<'a>,
    pub narration: ChatTypeDecoration<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatTypeDecoration<'a> {
    pub translation_key: Utf8<'a>,
    pub parameters: List<'a, Parameter>,
    pub style: Style,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum Parameter {
    Sender,
    Target,
    Content,
}

impl Parameter {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sender => "sender",
            Self::Target => "target",
            Self::Content => "content",
        }
    }
}
