pub trait Read: Sized {
    fn read(buf: &mut &[u8]) -> Option<Self>;
}
