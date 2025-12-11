use alloc::alloc::{Allocator, Global};
use alloc::vec::Vec;

pub struct Scanner<A: Allocator = Global> {
    height: Stack<A>,
    height_bit_len: usize,
    state: State,
}

#[derive(Clone, Copy)]
enum State {}

#[derive(Clone)]
enum Stack<A: Allocator = Global> {
    Inline(u8, [u8; 38]),
    Heap(Vec<u8, A>),
}
