use crate::effect::MobEffect;
use crate::registry::SoundEventRef;
use crate::sound::SoundEvent;
use crate::{Holder, HolderSet};
use haya_collection::List;
use minecraft_data::{consume_effect_type, mob_effect};

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = consume_effect_type)]
pub enum ConsumeEffect<'a> {
    ApplyEffects {
        effects: List<'a, MobEffect<'a>>,
        probability: f32,
    },
    RemoveEffects {
        effects: HolderSet<'a, mob_effect>,
    },
    ClearAllEffects,
    TeleportRandomly {
        diameter: f32,
    },
    PlaySound {
        sound: Holder<SoundEvent<'a>, SoundEventRef>,
    },
}
