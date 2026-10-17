use crate::registry::ChatTypeRef;
use crate::{BitSet, ComponentRaw, Holder, Instant, PublicKey, StyleRaw};
use haya_collection::List;
use mser::{ByteArray, FixedByteArray, Read, Utf8, V32, Write};
use uuid::Uuid;

extern crate alloc;

pub mod click_event;
pub mod dialog;
pub mod hover_event;
pub mod text;

#[derive(Clone, Serialize, Deserialize)]
pub struct Bound<'a> {
    pub chat_type: Holder<ChatType<'a>, ChatTypeRef>,
    pub name: ComponentRaw,
    pub target_name: Option<ComponentRaw>,
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
    pub style: StyleRaw,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum Parameter {
    Sender,
    Target,
    Content,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SignedMessageBodyPacked<'a> {
    pub content: Utf8<'a, 256>,
    pub timestamp: Instant,
    pub salt: u64,
    pub last_seen: LastSeenMessagesPacked<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LastSeenMessagesPacked<'a> {
    pub entries: List<'a, MessageSignaturePacked<'a>, 20>,
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = FilterMaskType, PascalCase)]
pub enum FilterMask<'a> {
    PassThrough,
    FullyFiltered,
    PartiallyFiltered { mask: BitSet<'a> },
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum FilterMaskType {
    PassThrough,
    FullyFiltered,
    PartiallyFiltered,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RemoteChatSession<'a> {
    pub session_id: Uuid,
    pub profile_public_key: ProfilePublicKey<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProfilePublicKey<'a> {
    pub expires_at: Instant,
    pub key: PublicKey<'a>,
    pub key_signature: ByteArray<'a, 4096>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LastSeenMessagesUpdate<'a> {
    #[mser(varint)]
    pub offset: u32,
    pub acknowledged: FixedByteArray<'a, 3>,
    pub checksum: u8,
}

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
            Self::Index(id - 1)
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum ChatFormatting {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Obfuscated,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Reset,
}
