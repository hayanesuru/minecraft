#![no_std]

use alloc::vec::Vec;
use haya_collection::List;
use haya_ident::Ident;
use haya_nbt::Tag;
use mser::{Either, Error, Read, Reader, Utf8, V21, V32, Write, Writer};

pub mod advancement;
pub mod attribute;
pub mod clientbound;
pub mod command;
pub mod food;
pub mod item;
pub mod profile;
pub mod serverbound;
pub mod sound;
pub mod stat;
pub mod types;

#[macro_use]
extern crate mser_macro;
extern crate alloc;

#[derive(Clone, Copy, Debug)]
pub struct Translatable<'a>(pub &'a str);

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ClientIntent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

impl Write for ClientIntent {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            w.write_byte(*self as u8);
        }
    }

    fn len_s(&self) -> usize {
        1
    }
}

impl<'a> Read<'a> for ClientIntent {
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        match V21::read(buf)?.0 {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Transfer),
            _ => Err(Error),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ComponentJson<'a>(pub Utf8<'a, 262144>);

#[derive(Clone, Serialize, Deserialize)]
pub struct Component(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
pub struct Dialog(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
pub struct KnownPack<'a> {
    pub namespace: Utf8<'a>,
    pub id: Utf8<'a>,
    pub version: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerLinkUntrustedEntry<'a> {
    pub ty: Either<KnownLinkType, Component>,
    pub url: Utf8<'a>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum KnownLinkType {
    ReportBug,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}

impl KnownLinkType {
    pub const fn key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::ReportBug => "known_server_link.report_bug",
            Self::CommunityGuidelines => "known_server_link.community_guidelines",
            Self::Support => "known_server_link.support",
            Self::Status => "known_server_link.status",
            Self::Feedback => "known_server_link.feedback",
            Self::Community => "known_server_link.community",
            Self::Website => "known_server_link.website",
            Self::Forums => "known_server_link.forums",
            Self::News => "known_server_link.news",
            Self::Announcements => "known_server_link.announcements",
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientInformation<'a> {
    pub language: Utf8<'a, 16>,
    pub view_distance: u8,
    pub chat_visibility: ChatVisibility,
    pub chat_colors: bool,
    pub model_customisation: u8,
    pub main_hand: HumanoidArm,
    pub text_filtering_enabled: bool,
    pub allows_listing: bool,
    pub particle_status: ParticleStatus,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

impl ChatVisibility {
    pub const fn key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::Full => "options.chat.visibility.full",
            Self::System => "options.chat.visibility.system",
            Self::Hidden => "options.chat.visibility.hidden",
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum HumanoidArm {
    Left,
    Right,
}

impl HumanoidArm {
    pub const fn key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::Left => "options.mainHand.left",
            Self::Right => "options.mainHand.right",
        })
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ParticleStatus {
    All,
    Decreased,
    Minimal,
}

impl ParticleStatus {
    pub const fn key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::All => "options.particles.all",
            Self::Decreased => "options.particles.decreased",
            Self::Minimal => "options.particles.minimal",
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Peaceful => "peaceful",
            Self::Easy => "easy",
            Self::Normal => "normal",
            Self::Hard => "hard",
        }
    }

    pub const fn key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::Peaceful => "options.difficulty.peaceful",
            Self::Easy => "options.difficulty.easy",
            Self::Normal => "options.difficulty.normal",
            Self::Hard => "options.difficulty.hard",
        })
    }

    pub const fn parse(n: &[u8]) -> Option<Self> {
        match n {
            b"peaceful" => Some(Self::Peaceful),
            b"easy" => Some(Self::Easy),
            b"normal" => Some(Self::Normal),
            b"hard" => Some(Self::Hard),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ContainerId(pub V32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Enchntment(pub V32);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DamageType(pub V32);

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl Rarity {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::Epic => "epic",
        }
    }
}

#[derive(Clone)]
pub enum HolderSet<'a, T> {
    Named(Ident<'a>),
    Direct(List<'a, T>),
}

impl<'a, T: Read<'a>> Read<'a> for HolderSet<'a, T> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = V32::read(buf)?.0;
        if len == 0 {
            let name = Ident::read(buf)?;
            Ok(Self::Named(name))
        } else {
            let len = (len - 1) as usize;
            let mut vec = Vec::with_capacity(usize::min(len, 65536));
            for _ in 0..len {
                vec.push(T::read(buf)?);
            }
            Ok(Self::Direct(List::Owned(vec)))
        }
    }
}

impl<T: Write> Write for HolderSet<'_, T> {
    unsafe fn write(&self, w: &mut Writer) {
        match self {
            Self::Named(name) => unsafe {
                V32(0).write(w);
                name.write(w);
            },
            Self::Direct(direct) => unsafe {
                V32((direct.len() + 1) as u32).write(w);
                for holder in direct.as_slice() {
                    holder.write(w);
                }
            },
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Named(name) => V32(0).len_s() + name.len_s(),
            Self::Direct(direct) => {
                let mut len = V32((direct.len() + 1) as u32).len_s();
                for x in direct.as_slice() {
                    len += x.len_s();
                }
                len
            }
        }
    }
}

#[derive(Clone)]
pub enum Holder<T> {
    Reference(u32),
    Direct(T),
}

impl<'a, T: Read<'a>> Read<'a> for Holder<T> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let id = V32::read(buf)?.0;
        if id == 0 {
            Ok(Self::Direct(T::read(buf)?))
        } else {
            Ok(Self::Reference(id - 1))
        }
    }
}

impl<T: Write> Write for Holder<T> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match self {
                Self::Reference(id) => {
                    V32(*id + 1).write(w);
                }
                Self::Direct(direct) => {
                    V32(0).write(w);
                    direct.write(w);
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Reference(id) => V32(*id + 1).len_s(),
            Self::Direct(direct) => {
                let mut len = V32(0).len_s();
                len += direct.len_s();
                len
            }
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum EquipmentSlotGroup {
    Any,
    Mainhand,
    Offhand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
    Saddle,
}

impl EquipmentSlotGroup {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Mainhand => "mainhand",
            Self::Offhand => "offhand",
            Self::Hand => "hand",
            Self::Feet => "feet",
            Self::Legs => "legs",
            Self::Chest => "chest",
            Self::Head => "head",
            Self::Armor => "armor",
            Self::Body => "body",
            Self::Saddle => "saddle",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clientbound::login::LoginFinished;
    use crate::profile::GameProfileRef;
    use crate::types::Id as _;
    use haya_collection::List;
    use minecraft_data::clientbound__login;
    use uuid::Uuid;

    #[test]
    fn test_write() {
        let packet: LoginFinished = LoginFinished {
            game_profile: GameProfileRef {
                id: Uuid::nil(),
                name: Utf8("abc"),
                properties: List::Borrowed(&[]),
            },
        };

        let id = LoginFinished::ID;
        let len1 = id.len_s();
        let len2 = packet.len_s() + len1;
        let data = unsafe {
            let mut data = alloc::vec::Vec::with_capacity(len2);
            mser::write_unchecked(data.as_mut_ptr(), &id);
            mser::write_unchecked(data.as_mut_ptr().add(len1), &packet);
            data.set_len(len2);
            data.into_boxed_slice()
        };
        let mut data = Reader::new(&data);
        let id = V32::read(&mut data).unwrap().0;
        assert_eq!(clientbound__login::new(id as _).unwrap(), LoginFinished::ID);
        assert_eq!(Uuid::read(&mut data).unwrap(), Uuid::nil());
        assert_eq!(Utf8::<16>::read(&mut data).unwrap().0, "abc");
        assert_eq!(V32::read(&mut data).unwrap().0, 0);
        assert!(data.is_empty());
    }
}
