use crate::ComponentRaw;
use haya_ident::Ident;
use minecraft_data::{villager_profession, villager_type};
use mser::{Read, V21, Write};
use uuid::Uuid;

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum FoxVariant {
    Red,
    Snow,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint)]
pub enum SalmonVariant {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum ParrotVariant {
    RedBlue,
    Blue,
    Green,
    YellowBlue,
    Gray,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum TropicalFishPattern {
    Kob,
    Sunstreak,
    Snooper,
    Dasher,
    Brinely,
    Spotty,
    Flopper,
    Stripey,
    Glitter,
    Blockfish,
    Betty,
    Clayfish,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum MushroomCowVariant {
    Red,
    Brown,
}

#[derive(Clone, Copy, Name)]
#[repr(u8)]
#[mser(camel_case)]
pub enum RabbitVariant {
    Brown,
    White,
    Black,
    WhiteSplotched,
    Gold,
    Salt,
    Evil = 99,
}

impl Write for RabbitVariant {
    unsafe fn write(&self, w: &mut mser::Writer) {
        unsafe { w.write_byte(*self as u8) }
    }

    fn len_s(&self) -> usize {
        1
    }
}

impl<'a> Read<'a> for RabbitVariant {
    fn read(buf: &mut mser::Reader<'a>) -> Result<Self, mser::Error> {
        Ok(match V21::read(buf)?.0 {
            1 => Self::White,
            2 => Self::Black,
            3 => Self::WhiteSplotched,
            4 => Self::Gold,
            5 => Self::Salt,
            99 => Self::Evil,
            _ => Self::Brown,
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum EquineVariant {
    White,
    Creamy,
    Chestnut,
    Brown,
    Black,
    Gray,
    DarkBrown,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PaintingVariant<'a> {
    #[mser(varint)]
    pub width: u32,
    #[mser(varint)]
    pub height: u32,
    pub asset_id: Ident<'a>,
    pub title: Option<ComponentRaw>,
    pub author: Option<ComponentRaw>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum LlamaVariant {
    Creamy,
    White,
    Brown,
    Gray,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum AxolotlVariant {
    Lucy,
    Wild,
    Gold,
    Cyan,
    Blue,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EntityReference {
    pub entity: Uuid,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VillagerData {
    pub ty: villager_type,
    pub profession: villager_profession,
    #[mser(varint)]
    pub level: u32,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum ArmadilloState {
    Idle,
    Rolling,
    Scared,
    Unrolling,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum SnifferState {
    Idling,
    FeelingHappy,
    Scenting,
    Sniffing,
    Searching,
    Digging,
    Rising,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum CopperGolemState {
    Idle,
    GettingItem,
    GettingNoItem,
    DroppingItem,
    DroppingNoItem,
}

#[derive(Clone, Copy, Serialize, Deserialize, Name)]
#[repr(u8)]
#[mser(varint, camel_case)]
pub enum Pose {
    Standing,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Crouching,
    LongJumping,
    Dying,
    Croaking,
    UsingTongue,
    Sitting,
    Roaring,
    Sniffing,
    Emerging,
    Digging,
    Sliding,
    Shooting,
    Inhaling,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum EntityEvent {
    Unused0,
    Jump,
    KineticHit,
    Death,
    StartAttacking,
    StopAttacking,
    TamingFailed,
    TamingSucceeded,
    ShakeWetness,
    UseItemComplete,
    EatGrass,
    OfferFlower,
    LoveHearts,
    VillagerAngry,
    VillagerHappy,
    WitchHatMagic,
    ZombieConverting,
    FireworksExplode,
    InLoveHearts,
    SquidAnimSynch,
    SilverfishMergeAnim,
    GuardianAttackSound,
    ReducedDebugInfo,
    FullDebugInfo,
    PermissionLevelAll,
    PermissionLevelModerators,
    PermissionLevelGamemasters,
    PermissionLevelAdmins,
    PermissionLevelOwners,
    Unused29,
    Unused30,
    FishingRodReelIn,
    ArmorstandWobble,
    Unused33,
    StopOfferFlower,
    ProtectedFromDeath,
    Unused36,
    Unused37,
    DolphinLookingForTreasure,
    RavagerStunned,
    TrustingFailed,
    TrustingSucceeded,
    VillagerSweat,
    Unused43,
    Unused44,
    FoxEat,
    Teleport,
    MainhandBreak,
    OffhandBreak,
    HeadBreak,
    ChestBreak,
    LegsBreak,
    FeetBreak,
    HoneySlide,
    HoneyJump,
    SwapHands,
    CancelShakeWetness,
    Unused57,
    StartRam,
    EndRam,
    Poof,
    TendrilsShiver,
    SonicCharge,
    SnifferDiggingSound,
    ArmadilloPeek,
    BodyBreak,
    Shake,
    DrownParticles,
    SaddleBreak,
    RavagerRoared,
    TntPrime,
    BoatSink,
    BoatLaunch,
}
