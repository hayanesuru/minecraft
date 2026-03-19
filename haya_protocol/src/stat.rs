use minecraft_data::{block, custom_stat, entity_type, item, stat_type};

#[derive(Clone, Copy)]
pub enum Stat {
    Mined(block),
    Crafted(item),
    Used(item),
    Broken(item),
    PickedUp(item),
    Dropped(item),
    Killed(entity_type),
    KilledBy(entity_type),
    Custom(custom_stat),
}

impl<'a> mser::Read<'a> for Stat {
    fn read(buf: &mut mser::Reader<'a>) -> Result<Self, mser::Error> {
        let stat_type = stat_type::read(buf)?;
        match stat_type {
            stat_type::mined => Ok(Stat::Mined(block::read(buf)?)),
            stat_type::crafted => Ok(Stat::Crafted(item::read(buf)?)),
            stat_type::used => Ok(Stat::Used(item::read(buf)?)),
            stat_type::broken => Ok(Stat::Broken(item::read(buf)?)),
            stat_type::picked_up => Ok(Stat::PickedUp(item::read(buf)?)),
            stat_type::dropped => Ok(Stat::Dropped(item::read(buf)?)),
            stat_type::killed => Ok(Stat::Killed(entity_type::read(buf)?)),
            stat_type::killed_by => Ok(Stat::KilledBy(entity_type::read(buf)?)),
            stat_type::custom => Ok(Stat::Custom(custom_stat::read(buf)?)),
        }
    }
}

impl mser::Write for Stat {
    unsafe fn write(&self, w: &mut mser::Writer) {
        unsafe {
            match self {
                Self::Mined { .. } => stat_type::mined,
                Self::Crafted { .. } => stat_type::crafted,
                Self::Used { .. } => stat_type::used,
                Self::Broken { .. } => stat_type::broken,
                Self::PickedUp { .. } => stat_type::picked_up,
                Self::Dropped { .. } => stat_type::dropped,
                Self::Killed { .. } => stat_type::killed,
                Self::KilledBy { .. } => stat_type::killed_by,
                Self::Custom { .. } => stat_type::custom,
            }
            .write(w);
            match self {
                Self::Mined(x) => x.write(w),
                Self::Crafted(x) => x.write(w),
                Self::Used(x) => x.write(w),
                Self::Broken(x) => x.write(w),
                Self::PickedUp(x) => x.write(w),
                Self::Dropped(x) => x.write(w),
                Self::Killed(x) => x.write(w),
                Self::KilledBy(x) => x.write(w),
                Self::Custom(x) => x.write(w),
            }
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Mined { .. } => stat_type::mined,
            Self::Crafted { .. } => stat_type::crafted,
            Self::Used { .. } => stat_type::used,
            Self::Broken { .. } => stat_type::broken,
            Self::PickedUp { .. } => stat_type::picked_up,
            Self::Dropped { .. } => stat_type::dropped,
            Self::Killed { .. } => stat_type::killed,
            Self::KilledBy { .. } => stat_type::killed_by,
            Self::Custom { .. } => stat_type::custom,
        }
        .len_s()
            + match self {
                Self::Mined(x) => x.len_s(),
                Self::Crafted(x) => x.len_s(),
                Self::Used(x) => x.len_s(),
                Self::Broken(x) => x.len_s(),
                Self::PickedUp(x) => x.len_s(),
                Self::Dropped(x) => x.len_s(),
                Self::Killed(x) => x.len_s(),
                Self::KilledBy(x) => x.len_s(),
                Self::Custom(x) => x.len_s(),
            }
    }
}
