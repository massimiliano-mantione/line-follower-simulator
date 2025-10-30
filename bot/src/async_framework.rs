use std::{
    cell::Cell,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::wasm_bindings::devices::{
    DeviceValue, FutureHandle, PollOperationStatus, device_poll, forget_handle, poll_loop,
};

/// A pinned and boxed value
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

/// Run a pinned and boxed future, polling until completion
pub fn run_boxed(mut root_task: PinBoxed<impl Future<Output = ()>>) {
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

/// Run a pinned and boxed future, polling until completion
pub fn run(root_task: impl Future<Output = ()>) {
    run_boxed(pin_boxed(root_task))
}

/// A future value (wraps a `FutureHandle` from WASM bindings)
pub struct FutureValue {
    handle: FutureHandle,
}

impl From<FutureHandle> for FutureValue {
    fn from(handle: FutureHandle) -> Self {
        Self { handle }
    }
}

/// Convenience extension trait for `FutureHandle`
pub trait FutureHandleExt {
    /// Convert a `FutureHandle` into a `FutureValue`
    fn into_future(self) -> FutureValue;
}

impl FutureHandleExt for FutureHandle {
    fn into_future(self) -> FutureValue {
        FutureValue::from(self)
    }
}

impl Future for FutureValue {
    type Output = DeviceValue;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match device_poll(self.handle) {
            PollOperationStatus::Ready(value) => Poll::Ready(value),
            PollOperationStatus::Pending => Poll::Pending,
        }
    }
}

impl Drop for FutureValue {
    fn drop(&mut self) {
        forget_handle(self.handle);
    }
}

/// This acts like a single-value channel that only retains the most recent value
pub struct ValueWatcher<T> {
    counter: Cell<usize>,
    value: Cell<T>,
}

impl<T: Copy + Default> ValueWatcher<T> {
    /// Create the channel
    pub fn new() -> Self {
        ValueWatcher {
            counter: Cell::new(0),
            value: Cell::new(T::default()),
        }
    }

    /// Get the current value (returns immediately)
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Update the current value
    pub fn update(&self, value: T) {
        self.counter.set(self.counter.get() + 1);
        self.value.set(value);
    }

    /// Wait for a new value (set using `update`)
    pub fn next<'a>(&'a self) -> NextValue<'a, T> {
        NextValue {
            sender: self,
            counter: self.counter.get() + 1,
        }
    }

    /// Get a stream of new values
    pub fn stream<'a>(&'a self) -> ValueStream<'a, T> {
        ValueStream {
            sender: self,
            counter: self.counter.get(),
        }
    }
}

/// A future value that resolves when its channel is updated
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct NextValue<'a, T> {
    sender: &'a ValueWatcher<T>,
    counter: usize,
}

impl<'a, TO> NextValue<'a, TO> {
    /// Map the value to a new type
    pub fn map<TM>(
        self,
        mapper: &'a impl Fn(TO) -> TM,
    ) -> MappedValue<'a, Self, impl Fn(TO) -> TM> {
        MappedValue {
            original: self,
            mapper,
        }
    }

    pub fn filter(
        self,
        filter: &'a impl Fn(&TO) -> bool,
    ) -> FilteredValue<'a, Self, impl Fn(&TO) -> bool> {
        FilteredValue {
            original: self,
            filter,
        }
    }
}

impl<T: Copy + Default> Future for NextValue<'_, T> {
    type Output = T;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context,
    ) -> core::task::Poll<Self::Output> {
        if self.sender.counter.get() >= self.counter {
            core::task::Poll::Ready(self.sender.get())
        } else {
            core::task::Poll::Pending
        }
    }
}

pin_project! {
    /// A mapped future value
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct MappedValue<'a, FUTURE, MAPPER> {
        #[pin]
        original: FUTURE,
        mapper: &'a MAPPER,
    }
}

impl<'a, FUTURE: Future<Output = TO> + 'a, TO, TM, MAPPER: Fn(TO) -> TM>
    MappedValue<'a, FUTURE, MAPPER>
{
    pub fn new(original: FUTURE, mapper: &'a MAPPER) -> Self {
        MappedValue { original, mapper }
    }

    /// Filter the value with a predicate
    pub fn filter<FILTER>(
        self,
        filter: &'a impl Fn(TO) -> bool,
    ) -> FilteredValue<'a, Self, impl Fn(TO) -> bool> {
        FilteredValue {
            original: self,
            filter,
        }
    }
}

impl<'a, FUTURE: Future<Output = TO> + 'a, TO, TM, MAPPER: Fn(TO) -> TM> Future
    for MappedValue<'a, FUTURE, MAPPER>
{
    type Output = TM;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();
        match this.original.poll(cx) {
            core::task::Poll::Ready(value) => core::task::Poll::Ready((this.mapper)(value)),
            core::task::Poll::Pending => core::task::Poll::Pending,
        }
    }
}

pin_project! {
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct FilteredValue<'a, FUTURE, FILTER> {
        #[pin]
        original: FUTURE,
        filter: &'a FILTER,
    }
}

impl<'a, FUTURE: Future<Output = TO> + 'a, TO, FILTER: Fn(&TO) -> bool>
    FilteredValue<'a, FUTURE, FILTER>
{
    pub fn new(original: FUTURE, filter: &'a FILTER) -> Self {
        FilteredValue { original, filter }
    }

    /// Map the filtered value to a new type
    pub fn map<TM>(
        self,
        mapper: &'a impl Fn(TO) -> TM,
    ) -> MappedValue<'a, Self, impl Fn(TO) -> TM> {
        MappedValue {
            original: self,
            mapper,
        }
    }
}

impl<'a, FUTURE: Future<Output = TO> + 'a, TO, FILTER: Fn(&TO) -> bool> Future
    for FilteredValue<'a, FUTURE, FILTER>
{
    type Output = FUTURE::Output;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();
        match this.original.poll(cx) {
            core::task::Poll::Ready(value) => {
                if (this.filter)(&value) {
                    core::task::Poll::Ready(value)
                } else {
                    core::task::Poll::Pending
                }
            }
            core::task::Poll::Pending => core::task::Poll::Pending,
        }
    }
}

/// An asynchronous stream of values
pub struct ValueStream<'a, T: Copy + Default> {
    sender: &'a ValueWatcher<T>,
    counter: usize,
}

impl<T: Copy + Default> ValueStream<'_, T> {
    /// Get the next value from the stream
    pub fn next<'a>(&'a mut self) -> NextValue<'a, T> {
        let counter = self.sender.counter.get().max(self.counter);
        self.counter = counter + 1;
        NextValue {
            sender: self.sender,
            counter,
        }
    }
}
