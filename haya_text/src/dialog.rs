use crate::chat::TextComponent;
use crate::click_event::ClickEvent;
use alloc::boxed::Box;
use alloc::vec::Vec;
use haya_ident::Identifier;
use haya_nbt::{StringTag, Tag};

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
        dialogs: Tag,
        exit_action: Option<ActionButton>,
        columns: u32,
        button_width: u32,
    },
}

#[derive(Clone)]
pub struct CommonDialog {
    pub title: TextComponent,
    pub external_title: Option<TextComponent>,
    pub body: Vec<DialogBody>,
    pub inputs: Vec<Input>,
    pub can_close_with_escape: bool,
    pub pause: bool,
    pub after_action: AfterAction,
}

#[derive(Clone)]
pub enum DialogBody {
    PlainMessage {
        contents: TextComponent,
        width: u32,
    },
    Item {
        id: Identifier,
        count: Option<i32>,
        components: Option<Tag>,
        description: Description,
        show_decoration: bool,
        show_tooltip: bool,
        width: u32,
        height: u32,
    },
}

#[derive(Clone)]
pub struct Description {
    pub contents: Option<TextComponent>,
    pub width: Option<u32>,
}

#[derive(Clone)]
pub enum Input {
    Text {
        key: ParsedTemplate,
        label: Box<TextComponent>,
        width: u32,
        label_visible: bool,
        initial: Option<StringTag>,
        max_length: u32,
        multiline: Option<Multiline>,
    },
    Boolean {
        key: ParsedTemplate,
        label: TextComponent,
        initial: bool,
        on_true: Option<StringTag>,
        on_false: Option<StringTag>,
    },
    SingleOption {
        key: ParsedTemplate,
        label: TextComponent,
        width: u32,
        label_visible: bool,
        options: Vec<SingleOptionEntry>,
    },
    NumberRange {
        key: ParsedTemplate,
        label: TextComponent,
        width: u32,
        label_format: Option<StringTag>,
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
    pub id: StringTag,
    pub display: TextComponent,
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
    pub button: Button,
    pub action: Option<Action>,
}

#[derive(Clone)]
pub struct Button {
    pub label: Box<TextComponent>,
    pub tooltip: Option<Box<TextComponent>>,
    pub width: u32,
}

#[derive(Clone)]
pub enum Action {
    CommandTemplate {
        template: ParsedTemplate,
    },
    CustomAll {
        id: Identifier,
        additions: Option<Tag>,
    },
    Static {
        value: ClickEvent,
    },
}

#[derive(Clone)]
pub struct ParsedTemplate {
    pub raw: StringTag,
}
