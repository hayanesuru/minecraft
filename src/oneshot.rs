use core::future::Future;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::pin::Pin;
use core::ptr::NonNull;
use core::sync::atomic::{fence, AtomicU8, Ordering};
use core::task::{Context, Poll, Waker};

/// The initial channel state. Active while both endpoints are still alive, no message has been
/// sent, and the receiver is not receiving.
const EMPTY: u8 = 0b011;

/// A message has been sent to the channel, but the receiver has not yet read it.
const MESSAGE: u8 = 0b100;

/// No message has yet been sent on the channel, but the receiver is currently receiving.
const RECEIVING: u8 = 0b000;

const UNPARKING: u8 = 0b001;

/// The channel has been closed. This means that either the sender or receiver has been dropped,
/// or the message sent to the channel has already been received. Since this is a oneshot
/// channel, it is disconnected after the one message it is supposed to hold has been
/// transmitted.
const DISCONNECTED: u8 = 0b010;

pub struct Channel(AtomicU8, MaybeUninit<Waker>);
pub struct Sender(NonNull<Channel>);
pub struct Receiver(NonNull<Channel>);

unsafe impl Send for Sender {}
unsafe impl Send for Receiver {}

pub fn channel() -> (Sender, Receiver) {
    let channel_ptr = Box::into_raw(Box::new(Channel(
        AtomicU8::new(EMPTY),
        MaybeUninit::uninit(),
    )));
    let channel_ptr = unsafe { NonNull::new_unchecked(channel_ptr) };
    (Sender(channel_ptr), Receiver(channel_ptr))
}

impl Channel {
    #[inline(always)]
    unsafe fn write_waker(&mut self, waker: Waker) {
        self.1 = MaybeUninit::new(waker);
    }

    #[inline(always)]
    unsafe fn take_waker(&self) -> Waker {
        self.1.assume_init_read()
    }

    #[inline(always)]
    unsafe fn drop_waker(&mut self) {
        self.1.assume_init_drop()
    }

    /// # Safety
    ///
    /// * `Channel::waker` must not have a waker stored in it when calling this method.
    /// * Channel state must not be RECEIVING or UNPARKING when calling this method.
    unsafe fn write_async_waker(&mut self, waker: &Waker) -> Poll<bool> {
        // Write our thread instance to the channel.
        // SAFETY: we are not yet in the RECEIVING state, meaning that the sender will not
        // try to access the waker until it sees the state set to RECEIVING below
        self.write_waker(waker.clone());

        // ORDERING: we use release ordering on success so the sender can synchronize with
        // our write of the waker. We use relaxed ordering on failure since the sender does
        // not need to synchronize with our write and the individual match arms handle any
        // additional synchronization
        match self
            .0
            .compare_exchange(EMPTY, RECEIVING, Ordering::Release, Ordering::Relaxed)
        {
            // We stored our waker, now we return and let the sender wake us up
            Ok(_) => Poll::Pending,
            // The sender sent the message while we prepared to park.
            // We take the message and mark the channel disconnected.
            Err(MESSAGE) => {
                // ORDERING: Synchronize with the write of the message. This branch is
                // unlikely to be taken, so it's likely more efficient to use a fence here
                // instead of AcqRel ordering on the compare_exchange operation
                fence(Ordering::Acquire);

                // SAFETY: we started in the EMPTY state and the sender switched us to the
                // MESSAGE state. This means that it did not take the waker, so we're
                // responsible for dropping it.
                self.drop_waker();

                // ORDERING: sender does not exist, so this update only needs to be visible to us
                self.0.store(DISCONNECTED, Ordering::Relaxed);

                // SAFETY: The MESSAGE state tells us there is a correctly initialized message
                Poll::Ready(true)
            }
            // The sender was dropped before sending anything while we prepared to park.
            Err(_) => {
                // SAFETY: we started in the EMPTY state and the sender switched us to the
                // DISCONNECTED state. This means that it did not take the waker, so we're
                // responsible for dropping it.
                self.drop_waker();
                Poll::Ready(false)
            }
        }
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        // SAFETY: The receiver only ever frees the channel if we are in the MESSAGE or
        // DISCONNECTED states. If we are in the MESSAGE state, then we called
        // mem::forget(self), so we should not be in this function call. If we are in the
        // DISCONNECTED state, then the receiver either received a MESSAGE so this statement is
        // unreachable, or was dropped and observed that our side was still alive, and thus didn't
        // free the channel.
        let channel = unsafe { self.0.as_ref() };

        // Set the channel state to disconnected and read what state the receiver was in
        // ORDERING: we don't need release ordering here since there are no modifications we
        // need to make visible to other thread, and the Err(RECEIVING) branch handles
        // synchronization independent of this cmpxchg
        //
        // EMPTY ^ 001 = DISCONNECTED
        // RECEIVING ^ 001 = UNPARKING
        // DISCONNECTED ^ 001 = EMPTY (invalid), but this state is never observed
        match channel.0.fetch_xor(0b001, Ordering::Relaxed) {
            // The receiver has not started waiting, nor is it dropped.
            EMPTY => (),
            // The receiver is waiting. Wake it up so it can detect that the channel disconnected.
            RECEIVING => {
                // See comments in Sender::send

                fence(Ordering::Acquire);

                let waker = unsafe { channel.take_waker() };

                // We still need release ordering here to make sure our read of the waker happens
                // before this, and acquire ordering to ensure the unparking of the receiver
                // happens after this.
                channel.0.swap(DISCONNECTED, Ordering::AcqRel);

                // The Acquire ordering above ensures that the write of the DISCONNECTED state
                // happens-before unparking the receiver.
                waker.wake();
            }
            // The receiver was already dropped. We are responsible for freeing the channel.
            _ => {
                // SAFETY: when the receiver switches the state to DISCONNECTED they have received
                // the message or will no longer be trying to receive the message, and have
                // observed that the sender is still alive, meaning that we're responsible for
                // freeing the channel allocation.
                unsafe { drop(Box::from_raw(self.0.as_ptr())) };
            }
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        // SAFETY: since the receiving side is still alive the sender would have observed that and
        // left deallocating the channel allocation to us.
        let channel = unsafe { self.0.as_mut() };

        // Set the channel state to disconnected and read what state the receiver was in
        match channel.0.swap(DISCONNECTED, Ordering::Acquire) {
            // The sender has not sent anything, nor is it dropped.
            EMPTY => (),
            // The sender already sent something. We must drop it, and free the channel.
            MESSAGE => unsafe {
                drop(Box::from_raw(self.0.as_ptr()));
            },
            // The receiver has been polled.
            RECEIVING => {
                // TODO: figure this out when async is fixed
                unsafe { channel.drop_waker() };
            }
            // The sender was already dropped. We are responsible for freeing the channel.
            _ => unsafe { drop(Box::from_raw(self.0.as_ptr())) },
        }
    }
}

impl Future for Receiver {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: the existence of the `self` parameter serves as a certificate that the receiver
        // is still alive, meaning that even if the sender was dropped then it would have observed
        // the fact that we're still alive and left the responsibility of deallocating the
        // channel to us, so `self.channel` is valid
        let channel = unsafe { self.get_mut().0.as_mut() };

        // ORDERING: we use acquire ordering to synchronize with the store of the message.
        match channel.0.load(Ordering::Acquire) {
            // The sender is alive but has not sent anything yet.
            // SAFETY: We can't be in the forbidden states, and no waker in the channel.
            EMPTY => unsafe { channel.write_async_waker(cx.waker()) },
            // We were polled again while waiting for the sender. Replace the waker with the new one.
            RECEIVING => {
                // ORDERING: We use relaxed ordering on both success and failure since we have not
                // written anything above that must be released, and the individual match arms
                // handle any additional synchronization.
                match channel.0.compare_exchange(
                    RECEIVING,
                    EMPTY,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    // We successfully changed the state back to EMPTY. Replace the waker.
                    // This is the most likely branch to be taken, which is why we don't use any
                    // memory barriers in the compare_exchange above.
                    Ok(_) => {
                        // SAFETY: We wrote the waker in a previous call to poll. We do not need
                        // a memory barrier since the previous write here was by ourselves.
                        unsafe { channel.drop_waker() };
                        // SAFETY: We can't be in the forbidden states, and no waker in the channel.
                        unsafe { channel.write_async_waker(cx.waker()) }
                    }
                    // The sender sent the message while we prepared to replace the waker.
                    // We take the message and mark the channel disconnected.
                    // The sender has already taken the waker.
                    Err(MESSAGE) => {
                        // ORDERING: Synchronize with the write of the message. This branch is
                        // unlikely to be taken.
                        channel.0.swap(DISCONNECTED, Ordering::Acquire);
                        // SAFETY: The state tells us the sender has initialized the message.
                        Poll::Ready(true)
                    }
                    // The sender was dropped before sending anything while we prepared to park.
                    // The sender has taken the waker already.
                    Err(DISCONNECTED) => Poll::Ready(false),
                    // The sender is currently waking us up.
                    Err(_) => {
                        // We can't trust that the old waker that the sender has access to
                        // is honored by the async runtime at this point. So we wake ourselves
                        // up to get polled instantly again.
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
            // The sender sent the message.
            MESSAGE => {
                // ORDERING: the sender has been dropped so this update only needs to be
                // visible to us
                channel.0.store(DISCONNECTED, Ordering::Relaxed);
                Poll::Ready(true)
            }
            // The sender was dropped before sending anything, or we already received the message.
            DISCONNECTED => Poll::Ready(false),
            // The sender has observed the RECEIVING state and is currently reading the waker from
            // a previous poll. We need to loop here until we observe the MESSAGE or DISCONNECTED
            // state. We busy loop here since we know the sender is done very soon.
            _ => loop {
                core::hint::spin_loop();
                // ORDERING: The load above has already synchronized with the write of the message.
                match channel.0.load(Ordering::Relaxed) {
                    MESSAGE => {
                        // ORDERING: the sender has been dropped, so this update only
                        // needs to be visible to us
                        channel.0.store(DISCONNECTED, Ordering::Relaxed);
                        // SAFETY: We observed the MESSAGE state
                        break Poll::Ready(true);
                    }
                    DISCONNECTED => break Poll::Ready(false),
                    UNPARKING => (),
                    _ => unreachable!(),
                }
            },
        }
    }
}

impl Sender {
    pub fn send(self) -> bool {
        // Don't run our Drop implementation if send was called, any cleanup now happens here
        let mut x = ManuallyDrop::new(self);

        // SAFETY: The channel exists on the heap for the entire duration of this method and we
        // only ever acquire shared references to it. Note that if the receiver disconnects it
        // does not free the channel.
        let channel = unsafe { x.0.as_mut() };

        // Set the state to signal there is a message on the channel.
        // ORDERING: we use release ordering to ensure the write of the message is visible to the
        // receiving thread. The EMPTY and DISCONNECTED branches do not observe any shared state,
        // and thus we do not need acquire orderng. The RECEIVING branch manages synchronization
        // independent of this operation.
        //
        // EMPTY + 1 = MESSAGE
        // RECEIVING + 1 = UNPARKING
        // DISCONNECTED + 1 = invalid, however this state is never observed
        match channel.0.fetch_add(1, Ordering::Release) {
            // The receiver is alive and has not started waiting. Send done.
            EMPTY => true,
            // The receiver is waiting. Wake it up so it can return the message.
            RECEIVING => {
                // ORDERING: Synchronizes with the write of the waker to memory, and prevents the
                // taking of the waker from being ordered before this operation.
                fence(Ordering::Acquire);

                // Take the waker, but critically do not unpark it. If we unparked now, then the
                // receiving thread could still observe the UNPARKING state and re-park, meaning
                // that after we change to the MESSAGE state, it would remain parked indefinitely
                // or until a spurious wakeup.
                // SAFETY: at this point we are in the UNPARKING state, and the receiving thread
                // does not access the waker while in this state, nor does it free the channel
                // allocation in this state.
                let waker = unsafe { channel.take_waker() };

                // ORDERING: this ordering serves two-fold: it synchronizes with the acquire load
                // in the receiving thread, ensuring that both our read of the waker and write of
                // the message happen-before the taking of the message and freeing of the channel.
                // Furthermore, we need acquire ordering to ensure the unparking of the receiver
                // happens after the channel state is updated.
                channel.0.swap(MESSAGE, Ordering::AcqRel);

                // Note: it is possible that between the store above and this statement that
                // the receiving thread is spuriously unparked, takes the message, and frees
                // the channel allocation. However, we took ownership of the channel out of
                // that allocation, and freeing the channel does not drop the waker since the
                // waker is wrapped in MaybeUninit. Therefore this data is valid regardless of
                // whether or not the receive has completed by this point.
                waker.wake();

                true
            }
            _ => unsafe {
                drop(Box::from_raw(x.0.as_ptr()));
                false
            },
        }
    }
}
