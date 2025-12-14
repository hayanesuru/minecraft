use crate::chat::{ClickEvent, Component};
use crate::item::ItemStack;
use crate::nbt::Compound;
use crate::str::SmolStr;
use crate::{HolderSet, Identifier};
use alloc::alloc::{Allocator, Global};
use alloc::vec::Vec;

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Dialog<A: Allocator = Global> {
    Notice {
        common: CommonDialog<A>,
        action: ActionButton<A>,
    },
    Confirmation {
        common: CommonDialog<A>,
        yes_button: ActionButton<A>,
        no_button: ActionButton<A>,
    },
    MultiAction {
        common: CommonDialog<A>,
        actions: Vec<ActionButton<A>, A>,
        exit_action: Option<ActionButton<A>>,
        columns: u32,
    },
    ServerLinks {
        common: CommonDialog<A>,
        exit_action: Option<ActionButton<A>>,
        columns: u32,
        button_width: u32,
    },
    DialogList {
        common: CommonDialog<A>,
        dialogs: HolderSet<Dialog, A>,
        exit_action: Option<ActionButton<A>>,
        columns: u32,
        button_width: u32,
    },
}

#[derive(Clone)]
pub struct CommonDialog<A: Allocator = Global> {
    pub title: Component<A>,
    pub external_title: Option<Component<A>>,
    pub body: Vec<DialogBody<A>, A>,
    pub inputs: Vec<Input<A>, A>,
    pub can_close_with_escape: bool,
    pub pause: bool,
    pub after_action: AfterAction,
}

#[derive(Clone)]
pub enum DialogBody<A: Allocator = Global> {
    PlainMessage {
        contents: Component<A>,
        width: u32,
    },
    Item {
        item: ItemStack<A>,
        description: Description<A>,
        show_decoration: bool,
        show_tooltip: bool,
        width: u32,
        height: u32,
    },
}

#[derive(Clone)]
pub struct Description<A: Allocator = Global> {
    pub contents: Option<Component<A>>,
    pub width: Option<u32>,
}

#[derive(Clone)]
pub enum Input<A: Allocator = Global> {
    Text {
        key: ParsedTemplate<A>,
        label: Component<A>,
        width: u32,
        label_visible: bool,
        initial: Option<SmolStr<A>>,
        max_length: u32,
        multiline: Option<Multiline>,
    },
    Boolean {
        key: ParsedTemplate<A>,
        label: Component<A>,
        initial: bool,
        on_true: Option<SmolStr<A>>,
        on_false: Option<SmolStr<A>>,
    },
    SingleOption {
        key: ParsedTemplate<A>,
        label: Component<A>,
        width: u32,
        label_visible: bool,
        options: Vec<SingleOptionEntry, A>,
    },
    NumberRange {
        key: ParsedTemplate<A>,
        label: Component<A>,
        width: u32,
        label_format: Option<SmolStr<A>>,
        start: f32,
        end: f32,
        initial: Option<f32>,
        step: Option<f32>,
    },
}

#[derive(Clone, Copy)]
pub struct Multiline {
    pub max_lines: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Clone)]
pub struct SingleOptionEntry<A: Allocator = Global> {
    pub id: SmolStr<A>,
    pub display: Component<A>,
    pub initial: bool,
}

#[derive(Clone, Copy)]
pub enum AfterAction {
    Close,
    None,
    WaitForResponse,
}

#[derive(Clone)]
pub struct ActionButton<A: Allocator = Global> {
    pub botton: Botton<A>,
    pub action: Option<Action<A>>,
}

#[derive(Clone)]
pub struct Botton<A: Allocator = Global> {
    pub label: Component<A>,
    pub tooltip: Option<Component<A>>,
    pub width: u32,
}

#[derive(Clone)]
pub enum Action<A: Allocator = Global> {
    CommandTemplate {
        template: ParsedTemplate<A>,
    },
    CustomAll {
        id: Identifier<A>,
        additions: Option<Compound<A>>,
    },
    Static {
        value: ClickEvent<A>,
    },
}

#[derive(Clone)]
pub struct ParsedTemplate<A: Allocator = Global> {
    pub raw: SmolStr<A>,
}
