use std::cell::UnsafeCell;
use std::ptr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::sync::Arc;

#[inline(always)]
pub fn create_single_use_channel<T: Send>() -> SingleUseChannel<T> {
    SingleUseChannel {
        ready: AtomicBool::new(false),
        val: UnsafeCell::new(None),
    }
}

pub struct SingleUseChannel<T: Send> {
    ready: AtomicBool,
    val: UnsafeCell<Option<T>>,
}
impl<T: Send> SingleUseChannel<T> {
    #[inline(always)]
    pub fn split_channel(self) -> (SingleUseSender<T>, SingleUseReceiver<T>) {
        let channel = Arc::new(self);
        (
            SingleUseSender { channel: channel.clone() },
            SingleUseReceiver { channel },
        )
    }
}



pub struct SingleUseSender<T: Send> {
    channel: Arc<SingleUseChannel<T>>,
}
impl<T: Send> SingleUseSender<T> {
    
    #[inline(always)]
    pub fn send(self, val: T) {
        unsafe {
            ptr::write(self.channel.val.get(), Some(val));
        }
        self.channel.ready.store(true, Release);
    }
}

#[derive(Clone)]
pub struct SingleUseReceiver<T: Send> {
    channel: Arc<SingleUseChannel<T>>,
}
impl<T: Send> SingleUseReceiver<T> {
    #[inline(always)]
    pub fn wait_recv(self) -> T {
        loop {
            if self.channel.ready.load(Acquire) {
                unsafe {
                    break ptr::read(self.channel.val.get()).unwrap_unchecked()
                }
            } else {
                std::thread::yield_now();
            }
        }
    }
    #[inline(always)]
    pub fn try_recv(self) -> TryRecvVal<T> {
        if self.channel.ready.load(Acquire) {
            unsafe { TryRecvVal::Ok(ptr::read(self.channel.val.get()).unwrap_unchecked()) }
        } else {
            TryRecvVal::Fail(self)
        }
    }
}
pub enum TryRecvVal<T: Send> {
    Fail(SingleUseReceiver<T>),
    Ok(T),
}