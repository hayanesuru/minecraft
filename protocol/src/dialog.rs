use crate::chat::Component;
use crate::nbt::Compound;
use crate::str::SmolStr;
use crate::Identifier;
use alloc::alloc::{Allocator, Global};
use alloc::vec::Vec;

#[derive(Clone)]
pub enum Dialog<A: Allocator = Global> {
    Notice { common: CommonDialog<A> },
    Confirmation { common: CommonDialog<A> },
    MultiAction { common: CommonDialog<A> },
    ServerLinks { common: CommonDialog<A> },
    DialogList { common: CommonDialog<A> },
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
        id: Identifier<A>,
        count: u32,
        components: Compound,
        description: Description<A>,
        show_decoration: Option<bool>,
        show_tooltip: Option<bool>,
        width: Option<u32>,
        height: Option<u32>,
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
        key: SmolStr<A>,
        label: Component<A>,
        width: u32,
        label_visible: bool,
        initial: Option<SmolStr<A>>,
        max_length: u32,
        multiline: Option<Multiline>,
    },
    Boolean {
        key: SmolStr<A>,
        label: Component<A>,
        initial: bool,
        on_true: Option<SmolStr<A>>,
        on_false: Option<SmolStr<A>>,
    },
    SingleOption {
        key: SmolStr<A>,
        label: Component<A>,
        width: u32,
        label_visible: bool,
        options: Vec<SingleOptionEntry, A>,
    },
    NumberRange {
        key: SmolStr<A>,
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
