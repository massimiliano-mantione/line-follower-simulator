use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::line_follower_robot::devices::{
    DeviceValue, FutureHandle, PollError, PollOperationStatus, device_poll, poll_loop,
};

pub type PinBoxed<T> = core::pin::Pin<Box<T>>;
pub fn pin_boxed<T>(t: T) -> PinBoxed<T> {
    Box::pin(t)
}

fn no_op(_: *const ()) {}
fn no_op_clone(_: *const ()) -> core::task::RawWaker {
    noop_raw_waker()
}
static RWVT: core::task::RawWakerVTable =
    core::task::RawWakerVTable::new(no_op_clone, no_op, no_op, no_op);

#[inline]
fn noop_raw_waker() -> core::task::RawWaker {
    core::task::RawWaker::new(core::ptr::null(), &RWVT)
}

#[inline]
fn noop_waker() -> core::task::Waker {
    unsafe { core::task::Waker::from_raw(noop_raw_waker()) }
}

pub fn run(mut root_task: PinBoxed<impl Future<Output = ()>>) {
    let waker = noop_waker();
    let mut context = Context::from_waker(&waker);

    loop {
        poll_loop(true);
        if root_task.as_mut().poll(&mut context) == Poll::Ready(()) {
            break;
        }
        poll_loop(false);
    }
}

pub struct FutureValue {
    handle: FutureHandle,
}

impl From<FutureHandle> for FutureValue {
    fn from(handle: FutureHandle) -> Self {
        Self { handle }
    }
}

pub trait FutureHandleExt {
    fn into_future(self) -> FutureValue;
}

impl FutureHandleExt for FutureHandle {
    fn into_future(self) -> FutureValue {
        FutureValue::from(self)
    }
}

impl Future for FutureValue {
    type Output = Result<DeviceValue, PollError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match device_poll(self.handle) {
            Ok(PollOperationStatus::Ready(value)) => Poll::Ready(Ok(value)),
            Ok(PollOperationStatus::Pending) => Poll::Pending,
            Err(error) => Poll::Ready(Err(error)),
        }
    }
}
