use crate::{
    boxed_slice_as_array_unchecked, item, Bytes, GameMode, Hand, Read, UnsafeWriter, Writable,
    Write,
};
use core::cell::Cell;
use glam::DVec3;

pub const EMPTY_SPACE_SLOT: u16 = -999i16 as u16;
pub const CUROSR_SLOT: u16 = -1i16 as u16;
const Q_BUTTON_CHARITABLE: u8 = 0;
const Q_BUTTON_GREEDY: u8 = 1;
const Q_BUTTON_CLONE: u8 = 2;
const Q_START: u8 = 0;
const Q_CONTINUE: u8 = 1;
const Q_END: u8 = 2;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ItemStack {
    pub item: item,
    pub count: u8,
}

impl ItemStack {
    pub fn set_count(&mut self, count: u8) {
        self.count = count;
        if self.count == 0 {
            self.item = item::air;
        }
    }
}

impl Write for ItemStack {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        if self.count == 0 {
            w.write_byte(0);
        } else {
            w.write_byte(1);
            self.item.write(w);
            w.write_byte(self.count);
            w.write_byte(0);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        if self.count == 0 {
            1
        } else {
            3 + self.item.len()
        }
    }
}

impl Read for ItemStack {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        if buf.u8()? != 1 {
            Some(Self {
                item: item::air,
                count: 0,
            })
        } else {
            let item = item::read(buf)?;
            let count = buf.u8()?;
            if buf.u8()? == 1 {
                ser::nbt::Compound::read(buf)?;
            }
            if item == item::air {
                Some(Self {
                    item: item::air,
                    count: 0,
                })
            } else {
                Some(Self { item, count })
            }
        }
    }
}

#[derive(Clone)]
pub struct Inventory {
    revision: Cell<u8>,
    selected: Cell<u16>,
    pub data: Box<[ItemStack; 46]>,
    pub cursor: ItemStack,
    sync_id: u8,
    quick_craft_stage: u8,
    quick_craft_button: u8,
    quick_craft_slots: Vec<u16>,
}

impl Default for Inventory {
    fn default() -> Self {
        let data = vec![
            ItemStack {
                item: item::air,
                count: 0,
            };
            46
        ]
        .into_boxed_slice();
        Self {
            cursor: ItemStack {
                item: item::air,
                count: 0,
            },
            selected: Cell::new(0),
            data: unsafe { boxed_slice_as_array_unchecked(data) },
            revision: Cell::new(0),
            sync_id: 0,
            quick_craft_stage: 0,
            quick_craft_button: 0,
            quick_craft_slots: Vec::new(),
        }
    }
}

impl Inventory {
    #[inline]
    pub fn sync_id(&self) -> u8 {
        self.sync_id
    }

    pub fn select(&self, selected: u16) {
        self.selected.set(selected)
    }

    #[inline]
    pub fn revision_current(&self) -> u32 {
        self.revision.get() as u32
    }

    #[inline]
    pub fn revision(&self) -> u32 {
        self.revision.set(self.revision.get().wrapping_add(1));
        self.revision.get() as u32
    }

    #[inline]
    pub fn main_slot(&self) -> u16 {
        9
    }

    #[inline]
    pub fn main_with_hotbar(&mut self) -> &mut [ItemStack; 36] {
        unsafe { &mut *self.data.as_mut_ptr().add(self.main_slot() as usize).cast() }
    }

    #[inline]
    pub fn main_with_hotbar_len() -> usize {
        36
    }

    #[inline]
    pub fn main_no_hotbar(&mut self) -> &mut [ItemStack; 27] {
        unsafe { &mut *self.data.as_mut_ptr().add(self.main_slot() as usize).cast() }
    }

    #[inline]
    pub fn hotbar_slot(&self) -> u16 {
        9 + 27
    }

    #[inline]
    pub fn hotbar(&mut self) -> &mut [ItemStack; 9] {
        unsafe {
            &mut *self
                .data
                .as_mut_ptr()
                .add(self.hotbar_slot() as usize)
                .cast()
        }
    }

    #[inline]
    pub fn off_hand(&mut self) -> &mut ItemStack {
        unsafe { &mut *self.data.as_mut_ptr().add(self.off_hand_slot() as usize) }
    }

    #[inline]
    pub fn off_hand_slot(&self) -> u16 {
        45
    }

    #[inline]
    pub fn main_hand(&mut self) -> &mut ItemStack {
        unsafe { &mut *self.data.as_mut_ptr().add(self.main_hand_slot() as usize) }
    }

    #[inline]
    pub fn main_hand_slot(&self) -> u16 {
        36 + self.selected.get()
    }

    #[inline]
    pub fn hand_slot(&self, hand: Hand) -> u16 {
        match hand {
            Hand::MainHand => self.main_hand_slot(),
            Hand::OffHand => self.off_hand_slot(),
        }
    }

    #[inline]
    pub fn hand(&mut self, hand: Hand) -> &mut ItemStack {
        match hand {
            Hand::MainHand => self.main_hand(),
            Hand::OffHand => self.off_hand(),
        }
    }

    #[inline]
    pub fn crafing(&mut self) -> &mut [ItemStack; 4] {
        unsafe {
            &mut *self
                .data
                .as_mut_ptr()
                .add(self.crafing_slot() as usize)
                .cast()
        }
    }

    #[inline]
    pub fn crafing_slot(&self) -> u16 {
        1
    }

    #[inline]
    pub fn crafing_len() -> usize {
        4
    }

    #[inline]
    pub fn crafing_result(&mut self) -> &mut ItemStack {
        unsafe { &mut *self.data.as_mut_ptr() }
    }

    #[inline]
    pub fn crafing_result_slot(&mut self) -> u16 {
        0
    }

    pub fn click(
        &mut self,
        item_add: &mut Vec<(DVec3, ItemStack)>,
        button: u8,
        slot: u16,
        action_type: SlotActionType,
        pos: DVec3,
        game_mode: GameMode,
    ) {
        if action_type != SlotActionType::QuickCraft && self.quick_craft_stage != Q_START {
            self.quick_craft_stage = Q_START;
            self.quick_craft_slots.clear();
            return;
        }
        match action_type {
            SlotActionType::Pickup | SlotActionType::QuickMove if slot == EMPTY_SPACE_SLOT => 'a: {
                if button != 0 && button != 1 {
                    break 'a;
                }
                let left = button == 0;
                if self.cursor.count != 0 {
                    if left {
                        item_add.push((pos, self.cursor));
                        self.cursor.count = 0;
                    } else {
                        item_add.push((
                            pos,
                            ItemStack {
                                item: self.cursor.item,
                                count: 1,
                            },
                        ));
                        self.cursor.count -= 1;
                    }
                    if self.cursor.count == 0 {
                        self.cursor.item = item::air;
                    }
                }
            }
            SlotActionType::Pickup => 'a: {
                if button != 0 && button != 1 {
                    break 'a;
                }
                let left = button == 0;

                let x = match self.data.get_mut(slot as usize) {
                    Some(x) => x,
                    None => break 'a,
                };
                if self.cursor.count == 0 {
                    if x.count == 0 {
                    } else if left || x.count == 1 {
                        self.cursor = *x;
                        x.count = 0;
                        x.item = item::air;
                    } else {
                        self.cursor.count = 1;
                        self.cursor.item = x.item;
                        x.count -= 1;
                    }
                } else if x.count == 0 {
                    if left || self.cursor.count == 1 {
                        *x = self.cursor;
                        self.cursor.count = 0;
                        self.cursor.item = item::air;
                    } else {
                        x.count = 1;
                        x.item = self.cursor.item;
                        self.cursor.count -= 1;
                    }
                } else if x.item == self.cursor.item {
                    let max_stack = x.item.max_count();
                    if x.count + self.cursor.count <= max_stack {
                        if left || self.cursor.count == 1 {
                            x.count += self.cursor.count;
                            self.cursor.count = 0;
                            self.cursor.item = item::air;
                        } else {
                            x.count += 1;
                            self.cursor.count -= 1;
                        }
                    } else if x.count != max_stack {
                        if left {
                            self.cursor.count -= max_stack - x.count;
                            x.count = max_stack;
                        } else {
                            self.cursor.count -= 1;
                            x.count += 1;
                        }
                    }
                } else {
                    core::mem::swap(x, &mut self.cursor);
                }
            }
            SlotActionType::QuickMove => 'a: {
                if button != 0 && button != 1 {
                    break 'a;
                }
                let _x = match self.data.get_mut(slot as usize) {
                    Some(x) => x,
                    None => break 'a,
                };
            }
            SlotActionType::Swap => {}
            SlotActionType::Clone => {}
            SlotActionType::Throw => {
                let left = button == 0;
                if self.cursor.count == 0 {
                    if let Some(x) = self.data.get_mut(slot as usize) {
                        if left || x.count == 1 {
                            item_add.push((pos, *x));
                            x.count = 0;
                            x.item = item::air;
                        } else {
                            item_add.push((
                                pos,
                                ItemStack {
                                    item: x.item,
                                    count: 1,
                                },
                            ));

                            x.count -= 1;
                        }
                    }
                }
            }
            SlotActionType::QuickCraft => 'a: {
                let left = button == 0;
                let prev = self.quick_craft_stage;
                self.quick_craft_stage = button & 3;
                if self.cursor.count == 0
                    || ((prev != Q_CONTINUE || self.quick_craft_stage != Q_END)
                        && prev != self.quick_craft_stage)
                    || self.quick_craft_stage > Q_END
                {
                    self.quick_craft_stage = Q_START;
                    self.quick_craft_slots.clear();
                } else if self.quick_craft_stage == Q_START {
                    self.quick_craft_button = (button >> 2) & 3;
                    if self.quick_craft_button != Q_BUTTON_CLONE || game_mode == GameMode::Creative
                    {
                        self.quick_craft_stage = Q_CONTINUE;
                    } else {
                        self.quick_craft_stage = Q_START;
                    }
                    self.quick_craft_slots.clear();
                } else if self.quick_craft_stage == Q_CONTINUE {
                    let stack = match self.data.get(slot as usize) {
                        Some(x) => *x,
                        None => return,
                    };
                    if (stack.count == 0 || stack.item == self.cursor.item)
                        && (self.quick_craft_button == Q_BUTTON_CLONE
                            || self.cursor.count as usize > self.quick_craft_slots.len())
                    {
                        self.quick_craft_slots.push(slot);
                    }
                } else if self.quick_craft_stage == Q_END {
                    if self.quick_craft_slots.is_empty() {
                    } else if let [slot] = self.quick_craft_slots[..] {
                        let x = match self.data.get_mut(slot as usize) {
                            Some(x) => x,
                            None => break 'a,
                        };
                        if self.cursor.count == 0 {
                            if x.count == 0 {
                            } else if left || x.count == 1 {
                                self.cursor = *x;
                                x.count = 0;
                                x.item = item::air;
                            } else {
                                self.cursor.count = 1;
                                self.cursor.item = x.item;
                                x.count -= 1;
                            }
                        } else if x.count == 0 {
                            if left || self.cursor.count == 1 {
                                *x = self.cursor;
                                self.cursor.count = 0;
                                self.cursor.item = item::air;
                            } else {
                                x.count = 1;
                                x.item = self.cursor.item;
                                self.cursor.count -= 1;
                            }
                        } else if x.item == self.cursor.item {
                            let max_stack = x.item.max_count();
                            if x.count + self.cursor.count <= max_stack {
                                if left || self.cursor.count == 1 {
                                    x.count += self.cursor.count;
                                    self.cursor.count = 0;
                                    self.cursor.item = item::air;
                                } else {
                                    x.count += 1;
                                    self.cursor.count -= 1;
                                }
                            } else if x.count != max_stack {
                                if left {
                                    self.cursor.count -= max_stack - x.count;
                                    x.count = max_stack;
                                } else {
                                    self.cursor.count -= 1;
                                    x.count += 1;
                                }
                            }
                        } else {
                            core::mem::swap(x, &mut self.cursor);
                        }
                    } else if self.cursor.count != 0 {
                        let slots_len = self.quick_craft_slots.len();
                        let mut slots = self.quick_craft_slots.iter();

                        'b: loop {
                            let mut slot2 = u16::MAX;
                            loop {
                                while slot2 == u16::MAX {
                                    match slots.next() {
                                        Some(&x) => {
                                            slot2 = x;
                                        }
                                        None => {
                                            break 'b;
                                        }
                                    }
                                }
                                if let Some(x) = self.data.get(slot2 as usize) {
                                    if (x.count == 0 || x.item == self.cursor.item)
                                        && (self.quick_craft_button == Q_BUTTON_CLONE
                                            || (self.cursor.count as usize) >= slots_len)
                                    {
                                        break;
                                    }
                                }
                            }
                            if let Some(x) = self.data.get_mut(slot2 as usize) {
                                let l = x.count;
                                let len = match self.quick_craft_button {
                                    Q_BUTTON_CHARITABLE => {
                                        (self.cursor.count as f64 / slots_len as f64).floor() as u8
                                    }
                                    Q_BUTTON_GREEDY => 1,
                                    Q_BUTTON_CLONE => self.cursor.item.max_count(),
                                    _ => self.cursor.count,
                                };
                                let n = self.cursor.item.max_count().min(len + l);
                                self.cursor.count -= n - l;
                                x.count = n;
                            }
                            if self.cursor.count == 0 {
                                self.cursor.item = item::air;
                                break 'b;
                            }
                        }
                    }
                    self.quick_craft_stage = Q_START;
                    self.quick_craft_slots.clear();
                }
            }
            SlotActionType::PickupAll => 'a: {
                let _x = match self.data.get_mut(slot as usize) {
                    Some(x) => x,
                    None => break 'a,
                };
                if self.cursor.count == 0 {
                    break 'a;
                }
                if button != 0 {
                    break 'a;
                }
                let max = self.cursor.item.max_count();
                for i in 0..2u8 {
                    for x in &mut *self.data {
                        if self.cursor.count >= max {
                            break;
                        }
                        if self.cursor.item == x.item && (x.count != max || i == 1) {
                            let rest = max - self.cursor.count;
                            if x.count <= rest {
                                self.cursor.count += x.count;
                                x.item = item::air;
                                x.count = 0;
                            } else {
                                self.cursor.count = max;
                                x.count -= rest;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Writable, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum SlotActionType {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

impl SlotActionType {
    #[inline]
    pub const fn new(n: u8) -> Self {
        if n > 6 {
            Self::Pickup
        } else {
            unsafe { core::mem::transmute(n) }
        }
    }
}

impl From<u8> for SlotActionType {
    #[inline]
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}
