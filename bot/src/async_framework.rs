use std::{
    cell::Cell,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

// All of these are used by the EXERCISE 8.x bodies once implemented.
#[allow(unused_imports)]
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

// Used by EXERCISE 8.1 once it is implemented. Read it: all four vtable entries do
// nothing, which is exactly right when nobody can wake anybody.
#[allow(dead_code)]
#[inline]
fn noop_waker() -> core::task::Waker {
    unsafe { core::task::Waker::from_raw(noop_raw_waker()) }
}

/// Run a pinned and boxed future, polling until completion
#[allow(unused_variables, unused_mut)]
pub fn run_boxed(mut root_task: PinBoxed<impl Future<Output = ()>>) {
    // EXERCISE 8.1: the entire executor. About ten lines.
    //
    // There is one thread, no interrupts, and nothing can wake anything: the only
    // thing that can make a future ready is the host, and the host only runs when we
    // call it. So there is no task queue, no spawning, and no waker worth the name -
    // `noop_waker()` above is a legitimate, sound `Waker` that has simply given up.
    //
    // Poll the root task until it is ready. Bracket each round with
    // `poll_loop(true)` before and `poll_loop(false)` after, which is what lets the
    // host fast-forward simulated time instead of letting us spin (see lesson 07).
    //
    // Mind the asymmetry: on the round that *completes*, break without closing the
    // bracket. The run is over; there is nothing left to wait for.
    todo!("poll the root task to completion")
}

/// Run a pinned and boxed future, polling until completion
pub fn run(root_task: impl Future<Output = ()>) {
    run_boxed(pin_boxed(root_task))
}

/// A future value (wraps a `FutureHandle` from WASM bindings)
#[allow(dead_code)]
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
        // EXERCISE 8.2: ask the host whether the device value has arrived.
        //
        // A direct translation of the host's answer. Note there is nothing to
        // register with `_cx`: there is no waker, so nobody to notify.
        //
        // See: device_poll, PollOperationStatus
        todo!("translate the host's poll result")
    }
}

impl Drop for FutureValue {
    fn drop(&mut self) {
        // EXERCISE 8.3: cancel the operation in the host.
        //
        // This one line is what makes cancellation work. `or!(sleep_for(MAX_TIME),
        // race_task())` drops the losing future, Rust runs this, and the host
        // forgets the operation. No cancellation tokens, no select! bookkeeping -
        // Rust's ownership model *is* the cancellation protocol.
        //
        // See: forget_handle
    }
}

/// This acts like a single-value channel that only retains the most recent value
#[allow(dead_code)]
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
        // EXERCISE 8.4: read the latest value, without waiting.
        todo!("return the current value")
    }

    /// Update the current value
    #[allow(unused_variables)]
    pub fn update(&self, value: T) {
        // EXERCISE 8.5: publish a new value.
        //
        // This is a one-slot channel that keeps only the newest value - there is no
        // queue. That is deliberate: this is a control loop, and a sensor reading
        // from three ticks ago is not *late* data, it is *wrong* data. Queueing it
        // would make the robot act on the past. Dropping stale samples is a feature.
        //
        // Consumers need to be able to tell that something new arrived, so there is
        // a counter as well as a value. Both are `Cell`, not `RefCell` or `Mutex`:
        // `T: Copy` and there is one thread, so no locking and no atomics.
        todo!("store the value and mark it as new")
    }

    /// Wait for a new value (set using `update`)
    pub fn next<'a>(&'a self) -> NextValue<'a, T> {
        // EXERCISE 8.6: a future that completes on the *next* update.
        //
        // Build a `NextValue` borrowing self, carrying the counter value it should
        // wait for. "Next" means strictly newer than what is here now.
        todo!("a future for the next update")
    }

    /// Get a stream of new values
    pub fn stream<'a>(&'a self) -> ValueStream<'a, T> {
        // EXERCISE 8.7: a stream over successive updates.
        todo!("a stream starting from the present")
    }
}

/// A future value that resolves when its channel is updated
#[allow(dead_code)]
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
        // EXERCISE 8.8: has the value we are waiting for arrived yet?
        //
        // Compare the channel's counter against the one this future was created
        // with. Careful with the comparison: a consumer that was not polled for
        // several updates must still see that it missed them.
        todo!("ready when the channel has moved on")
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
#[allow(dead_code)]
pub struct ValueStream<'a, T: Copy + Default> {
    sender: &'a ValueWatcher<T>,
    counter: usize,
}

impl<T: Copy + Default> ValueStream<'_, T> {
    /// Get the next value from the stream
    pub fn next<'a>(&'a mut self) -> NextValue<'a, T> {
        // EXERCISE 8.9: hand out the next future in the sequence.
        //
        // The stream tracks its own position so a consumer sees each update once,
        // and advances that position each time.
        //
        // One subtlety worth thinking about: what should happen when the consumer
        // has fallen *behind* the channel? It cannot catch up through values that no
        // longer exist, so it should resynchronise to the present rather than
        // replaying history. `max` is your friend.
        todo!("advance the stream position and return a future")
    }
}
