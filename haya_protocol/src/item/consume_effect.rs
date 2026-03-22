use crate::effect::MobEffect;
use crate::registry::SoundEventRef;
use crate::sound::SoundEvent;
use crate::{Holder, HolderSet};
use haya_collection::List;
use minecraft_data::{consume_effect_type, mob_effect};
use mser::{Error, Read, Reader, Write, Writer};

#[derive(Clone)]
pub enum ConsumeEffect<'a> {
    Apply {
        effects: List<'a, MobEffect<'a>>,
        probability: f32,
    },
    Remove {
        effects: HolderSet<'a, mob_effect>,
    },
    Clear,
    TeleportRandomly {
        diameter: f32,
    },
    PlaySound {
        sound: Holder<SoundEvent<'a>, SoundEventRef>,
    },
}

impl<'a> ConsumeEffect<'a> {
    pub const fn id(&self) -> consume_effect_type {
        match self {
            ConsumeEffect::Apply { .. } => consume_effect_type::apply_effects,
            ConsumeEffect::Remove { .. } => consume_effect_type::remove_effects,
            ConsumeEffect::Clear => consume_effect_type::clear_all_effects,
            ConsumeEffect::TeleportRandomly { .. } => consume_effect_type::teleport_randomly,
            ConsumeEffect::PlaySound { .. } => consume_effect_type::play_sound,
        }
    }
}

impl<'a> Read<'a> for ConsumeEffect<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(match consume_effect_type::read(buf)? {
            consume_effect_type::apply_effects => Self::Apply {
                effects: Read::read(buf)?,
                probability: Read::read(buf)?,
            },
            consume_effect_type::remove_effects => Self::Remove {
                effects: Read::read(buf)?,
            },
            consume_effect_type::clear_all_effects => Self::Clear,
            consume_effect_type::teleport_randomly => Self::TeleportRandomly {
                diameter: Read::read(buf)?,
            },
            consume_effect_type::play_sound => Self::PlaySound {
                sound: Read::read(buf)?,
            },
        })
    }
}

impl<'a> Write for ConsumeEffect<'a> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            self.id().write(w);
            match self {
                Self::Apply {
                    effects,
                    probability,
                } => {
                    effects.write(w);
                    probability.write(w);
                }
                Self::Remove { effects } => {
                    effects.write(w);
                }
                Self::Clear => (),
                Self::TeleportRandomly { diameter } => diameter.write(w),
                Self::PlaySound { sound } => sound.write(w),
            }
        }
    }

    fn len_s(&self) -> usize {
        self.id().len_s()
            + match self {
                Self::Apply {
                    effects,
                    probability,
                } => effects.len_s() + probability.len_s(),
                Self::Remove { effects } => effects.len_s(),
                Self::Clear => 0,
                Self::TeleportRandomly { diameter } => diameter.len_s(),
                Self::PlaySound { sound } => sound.len_s(),
            }
    }
}
