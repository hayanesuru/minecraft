use crate::chat::{ClickEvent, Component};
use crate::item::ItemStack;
use crate::nbt::Compound;
use crate::str::BoxStr;
use crate::{HolderSet, Identifier};
use alloc::boxed::Box;
use alloc::vec::Vec;

#[derive(Clone)]
pub struct Dialog {
    pub common: CommonDialog,
    pub data: DialogData,
}

#[derive(Clone)]
pub enum DialogData {
    Notice {
        action: ActionButton,
    },
    Confirmation {
        yes_button: ActionButton,
        no_button: ActionButton,
    },
    MultiAction {
        actions: Vec<ActionButton>,
        exit_action: Option<ActionButton>,
        columns: u32,
    },
    ServerLinks {
        exit_action: Option<ActionButton>,
        columns: u32,
        button_width: u32,
    },
    DialogList {
        dialogs: HolderSet<Dialog>,
        exit_action: Option<ActionButton>,
        columns: u32,
        button_width: u32,
    },
}

#[derive(Clone)]
pub struct CommonDialog {
    pub title: Component,
    pub external_title: Option<Component>,
    pub body: Vec<DialogBody>,
    pub inputs: Vec<Input>,
    pub can_close_with_escape: bool,
    pub pause: bool,
    pub after_action: AfterAction,
}

#[derive(Clone)]
pub enum DialogBody {
    PlainMessage {
        contents: Component,
        width: u32,
    },
    Item {
        item: ItemStack,
        description: Description,
        show_decoration: bool,
        show_tooltip: bool,
        width: u32,
        height: u32,
    },
}

#[derive(Clone)]
pub struct Description {
    pub contents: Option<Component>,
    pub width: Option<u32>,
}

#[derive(Clone)]
pub enum Input {
    Text {
        key: ParsedTemplate,
        label: Box<Component>,
        width: u32,
        label_visible: bool,
        initial: Option<BoxStr>,
        max_length: u32,
        multiline: Option<Multiline>,
    },
    Boolean {
        key: ParsedTemplate,
        label: Component,
        initial: bool,
        on_true: Option<BoxStr>,
        on_false: Option<BoxStr>,
    },
    SingleOption {
        key: ParsedTemplate,
        label: Component,
        width: u32,
        label_visible: bool,
        options: Vec<SingleOptionEntry>,
    },
    NumberRange {
        key: ParsedTemplate,
        label: Component,
        width: u32,
        label_format: Option<BoxStr>,
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
pub struct SingleOptionEntry {
    pub id: BoxStr,
    pub display: Component,
    pub initial: bool,
}

#[derive(Clone, Copy)]
pub enum AfterAction {
    Close,
    None,
    WaitForResponse,
}

#[derive(Clone)]
pub struct ActionButton {
    pub botton: Botton,
    pub action: Option<Action>,
}

#[derive(Clone)]
pub struct Botton {
    pub label: Box<Component>,
    pub tooltip: Option<Box<Component>>,
    pub width: u32,
}

#[derive(Clone)]
pub enum Action {
    CommandTemplate {
        template: ParsedTemplate,
    },
    CustomAll {
        id: Identifier,
        additions: Option<Compound>,
    },
    Static {
        value: ClickEvent,
    },
}

#[derive(Clone)]
pub struct ParsedTemplate {
    pub raw: BoxStr,
}
