use crate::{UnsafeWriter, Write, V21};
use core::cell::{Cell, RefCell};
use core::future::Future;
use core::mem::{swap, take};
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use std::rc::Rc;

pub fn channel() -> (Sender, Receiver) {
    let shared = Rc::new(Shared {
        data: RefCell::new(Data {
            buf: Vec::new(),
            waker: None,
        }),
        status: Cell::new(Status { closed: false }),
    });

    let sender = Sender {
        shared: shared.clone(),
    };

    let receiver = Receiver { shared };

    (sender, receiver)
}

struct Shared {
    data: RefCell<Data>,
    status: Cell<Status>,
}

#[derive(Copy, Clone)]
struct Status {
    closed: bool,
}

#[derive(Clone)]
struct Data {
    buf: Vec<u8>,
    waker: Option<Waker>,
}

impl Shared {
    fn extend(&self, data: &[u8]) -> Option<()> {
        if self.status.get().closed {
            return None;
        }
        let mut x = self.data.borrow_mut();
        x.buf.extend(data);
        if let Some(w) = x.waker.take() {
            w.wake();
        }
        Some(())
    }

    fn send(&self, packet: impl Write) -> Option<()> {
        if self.status.get().closed {
            return None;
        }

        let mut x = self.data.borrow_mut();
        let len = packet.len();
        let frame = V21(len as u32);
        let len = len + frame.len();
        x.buf.reserve(len);

        unsafe {
            let mut writer = UnsafeWriter(x.buf.as_mut_ptr().add(x.buf.len()));
            frame.write(&mut writer);
            packet.write(&mut writer);
            let len = x.buf.len() + len;
            x.buf.set_len(len);
        }

        if let Some(w) = x.waker.take() {
            w.wake();
        }
        Some(())
    }

    fn recv(&self, waker: &Waker) -> Poll<Vec<u8>> {
        let mut x = self.data.borrow_mut();
        if self.status.get().closed || !x.buf.is_empty() {
            Poll::Ready(take(&mut x.buf))
        } else {
            x.waker = Some(waker.clone());
            Poll::Pending
        }
    }

    fn close(&self) {
        self.status.replace(Status { closed: true });
        let mut x = self.data.borrow_mut();
        if let Some(w) = x.waker.take() {
            w.wake();
        }
    }
}

#[derive(Clone)]
pub struct Sender {
    shared: Rc<Shared>,
}

impl Unpin for Sender {}

impl Sender {
    #[inline]
    pub fn extend(&self, data: &[u8]) -> Option<()> {
        self.shared.extend(data)
    }

    #[inline]
    pub fn send(&self, data: impl Write) -> Option<()> {
        self.shared.send(data)
    }

    #[inline]
    pub fn close(&self) {
        self.shared.close();
    }

    #[inline]
    pub fn closed(&self) -> bool {
        self.shared.status.get().closed
    }
}

#[derive(Clone)]
pub struct Receiver {
    shared: Rc<Shared>,
}

impl Future for Receiver {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.shared.status.get().closed {
            return Poll::Ready(());
        }
        let mut s = self.shared.data.borrow_mut();
        if s.buf.is_empty() {
            s.waker = Some(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub struct Recv {
    shared: Rc<Shared>,
}

impl Future for Recv {
    type Output = Vec<u8>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.shared.recv(cx.waker())
    }
}

impl Receiver {
    #[inline]
    pub fn recv(&self) -> Recv {
        Recv {
            shared: self.shared.clone(),
        }
    }

    #[inline]
    pub fn try_recv(&self) -> Vec<u8> {
        take(&mut self.shared.data.borrow_mut().buf)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.shared.data.borrow().buf.is_empty()
    }

    pub fn reuse(&self, buf: &mut Vec<u8>) {
        let mut data = self.shared.data.borrow_mut();
        if !data.buf.is_empty() {
            buf.append(&mut data.buf);
        }
        swap(&mut data.buf, buf);
    }

    #[inline]
    pub fn send(&self, data: impl Write) -> Option<()> {
        self.shared.send(data)
    }

    #[inline]
    pub fn extend(&self, data: &[u8]) -> Option<()> {
        self.shared.extend(data)
    }

    #[inline]
    pub fn close(&self) {
        self.shared.close();
    }

    #[inline]
    pub fn closed(&self) -> bool {
        self.shared.status.get().closed
    }
}
