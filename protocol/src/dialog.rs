use crate::chat::Component;
use alloc::alloc::{Allocator, Global};

#[derive(Clone)]
pub enum Dialog<A: Allocator = Global> {
    Notice {
        title: Component<A>,
        external_title: Option<Component<A>>,
    },
    Confirmation {},
    MultiAction {},
    ServerLinks {},
    DialogList {},
}
