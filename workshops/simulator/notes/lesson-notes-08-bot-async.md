# Speaker notes — Lesson 08: The robot's async runtime

**Branch** `ws/simulator/08-bot-async` · **Optional pool — deep end** · ~45 min

The signup page lists "no_std and embedded async patterns" as a prerequisite, so this
lands on prepared ground. The mirror image of Lesson 07: the guest half of the
handshake.

**Goal.** Write a single-threaded, allocation-light executor with **no waker at all**,
and a channel type that lets concurrent robot tasks share sensor readings.

Only lesson needing `rustup target add wasm32-wasip2`.

---

## Removed on this branch

`bot/src/async_framework.rs`

- body of `run_boxed`
- `FutureValue::poll` and its `Drop` impl
- `ValueWatcher::{get, update, next, stream}`, `NextValue::poll`, `ValueStream::next`

Kept: the no-op waker machinery (`no_op`, `RWVT`, `noop_raw_waker`, `noop_waker`),
`MappedValue` / `FilteredValue`, and `async_api.rs` untouched — it is the consumer,
and reading it tells you what you must provide.

The waker stays because it is arcane `unsafe` boilerplate that is worth *reading
aloud* and not worth typing. It is the payoff of the "nobody can wake anybody"
argument, so show it; do not assign it.

## Framing (8 min)

Ask first: **why does a line follower need async at all?** It is one control loop.
The honest answer has two halves, and both are on the RustLab slides from last year:

1. *Concurrent sensor reads.* Devices have mismatched rates — line sensors every
   500 µs, gyro every 1 ms, fused IMU every 5 ms — on different buses. A loop that
   reads them sequentially waits for the slowest.
2. *Concurrent logic.* Error filtering, wheel-speed estimation, driving, telemetry,
   timeout and remote-control handling are separate state machines advancing at
   different rates. Expressing them as one flat loop means hand-rolling five
   interleaved state machines; expressing them as tasks means `zip!` and `or!`.

Then the design question: **what is the cheapest possible executor?**

There is exactly one thread. There are no interrupts. Nothing can wake anything,
because the only thing that can make a future ready is the *host*, and the host only
runs when we call it. So:

- no `Waker` — it would have nobody to notify. Hence the no-op vtable whose four
  function pointers all do nothing. It is already in the file; show it and let the
  room enjoy it:
  ```rust
  static RWVT: RawWakerVTable = RawWakerVTable::new(no_op_clone, no_op, no_op, no_op);
  ```
  This is a legitimate, sound `Waker`. It is just one that has given up.
- no task queue, no spawning, no `Box<dyn Future>` per task. Composition is
  *structural*: `zip!` and `or!` from `futures-micro` build one nested future whose
  shape is known at compile time. One `Box::pin` for the root, and that is the entire
  heap footprint of the runtime.
- no polling storm, thanks to the `poll_loop` bracket.

That last point is the connection to Lesson 07 — draw the handshake both ways.

## The pieces

**`run_boxed`** — the whole executor, ten lines:

```rust
let waker = noop_waker();
let mut context = Context::from_waker(&waker);
loop {
    poll_loop(true);
    if root_task.as_mut().poll(&mut context) == Poll::Ready(()) { break; }
    poll_loop(false);
}
```

Make them notice the asymmetry: `poll_loop(true)` before the poll,
`poll_loop(false)` *only if we are going round again*. On the iteration that
completes, we break without closing the bracket — the run is over, there is nothing
to wait for.

**`FutureValue::poll`** — a direct translation of the host's answer:

```rust
match device_poll(self.handle) {
    PollOperationStatus::Ready(value) => Poll::Ready(value),
    PollOperationStatus::Pending      => Poll::Pending,
}
```

No waker registration, because there is no waker. Note it ignores `cx` entirely.

**`Drop for FutureValue`** → `forget_handle(self.handle)`. This is the piece that
makes cancellation work: `or!(sleep_for(MAX_TIME), race_task())` drops the losing
future, Rust runs `Drop`, and the host forgets the operation. Point out how much
machinery this replaces — no cancellation tokens, no `select!` bookkeeping. Rust's
ownership model *is* the cancellation protocol.

**`ValueWatcher<T>`** — a one-slot channel that keeps only the newest value:
a `Cell<T>` plus a `Cell<usize>` counter. `update` bumps the counter and overwrites;
`NextValue` is ready when `sender.counter >= self.counter`.

Ask why "latest value wins" rather than a queue. Because this is a control loop: a
sensor reading from three ticks ago is not *late data*, it is *wrong data*. Queueing
it would make the robot act on the past. Backpressure is the wrong model for
telemetry-style streams, and this is a recurring embedded-systems lesson — dropping
stale samples is a feature.

`Cell` and not `RefCell`/`Mutex` because `T: Copy` and there is one thread. No
locking, no borrow-checking at runtime, no atomics.

Then `stream()` / `ValueStream::next`, which hands out `NextValue`s with a
monotonically increasing counter so a consumer task sees each update once:

```rust
let counter = self.sender.counter.get().max(self.counter);
self.counter = counter + 1;
```

The `max` handles a consumer that fell behind — it resynchronises to the present
rather than trying to catch up through values that no longer exist.

## Verification

```bash
cd bot && ./build.sh
cd ../sim && cargo run --release -p sim -- run --cli -l -i line_follower_robot.wasm
```

Switch `bot/src/lib.rs::run()` between the blocking and async variants (the call is
already there, commented). `examples::toy::run()` versus
`async_framework::run(examples::nb::toy::run())` must produce the same trajectory.

Then the payoff, and it is measurable: `async_api::get_line_sensors` reads both banks
with `zip!` in parallel, while `blocking_api::get_line_sensors` reads them in
sequence. Compare the timestamps in `log.txt` — the async version's loop iterations
are cheaper in simulated time. That is a real robot going faster because of an
executor.

Also try `examples::nb::parallel_tasks` (a race task plus a timeout task under `or!`)
and `tasks_with_channels` (a sensor task feeding a race task through a
`ValueWatcher`). Those demonstrate why the runtime exists.

## Notes to self

- If time is short, demo this rather than assigning it: the beats are the no-op waker,
  `Drop`-as-cancellation, and latest-value-wins, all of which show well on a screen.
- Expect a question about `embassy`. Fair answer: embassy is what you would use on
  real hardware, and it is excellent; this is 300 lines with no HAL, no interrupts
  and no timer queue because the host *is* the timer queue. The structural
  composition style (`zip!`/`or!` instead of spawning) is the same idea embassy's
  `join`/`select` use.
