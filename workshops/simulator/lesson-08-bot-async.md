# Lesson 08 — The robot's async runtime

> **Optional / deep end.** The mirror image of lesson 07: this is the *guest's* half
> of the handshake. Doing both gives you the whole design.

> **Goal.** Write a single-threaded, allocation-light executor with **no waker at
> all**, plus a channel that lets concurrent robot tasks share sensor readings.

This is the only lesson that needs the WASM toolchain:
`rustup target add wasm32-wasip2`.

## The setup

**Why does a line follower need async?** It is one control loop. Two reasons:

1. *Concurrent sensor reads.* Devices have mismatched rates — line sensors every
   500 µs, gyro every 1 ms, fused IMU every 5 ms — on different buses. A loop that
   reads them in sequence waits for the slowest.
2. *Concurrent logic.* Error filtering, wheel-speed estimation, driving, telemetry,
   timeout and remote-control handling are separate state machines advancing at
   different rates. As one flat loop that means hand-rolling five interleaved state
   machines; as tasks it means `zip!` and `or!`.

**What is the cheapest possible executor?** There is one thread. There are no
interrupts. Nothing can wake anything, because the only thing that can make a future
ready is the host, and the host only runs when we call it. So:

- **No waker.** It would have nobody to notify. Read `noop_waker` at the top of the
  file: a `RawWakerVTable` whose four entries all do nothing. It is a perfectly sound
  `Waker`. It has just given up.
- **No task queue, no spawning, no `Box<dyn Future>` per task.** Composition is
  *structural*: `zip!` and `or!` from `futures-micro` build one nested future whose
  shape is known at compile time. One `Box::pin` for the root is the entire heap
  footprint of the runtime.
- **No polling storm**, thanks to the `poll_loop` bracket — see lesson 07.

## Your task

All in `bot/src/async_framework.rs`:

| | What |
|---|---|
| 8.1 | `run_boxed` — the whole executor, about ten lines |
| 8.2 | `FutureValue::poll` — translate the host's answer |
| 8.3 | `Drop for FutureValue` — cancellation, one line |
| 8.4 | `ValueWatcher::get` |
| 8.5 | `ValueWatcher::update` |
| 8.6 | `ValueWatcher::next` |
| 8.7 | `ValueWatcher::stream` |
| 8.8 | `NextValue::poll` |
| 8.9 | `ValueStream::next` |

`MappedValue` and `FilteredValue` are left alone (combinator craft), and
`async_api.rs` is untouched — it is the consumer, so reading it tells you what you
must provide.

Suggested order: 8.2, 8.3, 8.1 (now the blocking-style async examples run), then the
channel, 8.4–8.9.

## How to verify

```bash
cd bot && ./build.sh
cd ../sim && cargo run --release -p sim -- run --cli -l -i line_follower_robot.wasm
```

`bot/src/lib.rs::run()` already has the variants, commented. Switch between them:

```rust
examples::toy::run();                                   // blocking
async_framework::run(examples::nb::toy::run());         // async, same behaviour
async_framework::run(examples::nb::parallel_tasks::run(4.0));   // or! - needs 8.3
async_framework::run(examples::nb::tasks_with_channels::run(4.0)); // needs 8.4-8.9
```

The blocking and async versions of `toy` must produce the same trajectory.

**Then the payoff, and it is measurable.** `async_api::get_line_sensors` reads both
sensor banks with `zip!` in parallel; `blocking_api::get_line_sensors` reads them in
sequence. Compare the timestamps in the log — the async version's loop iterations are
cheaper *in simulated time*. That is a real robot going faster because of an
executor.

## Hints

<details>
<summary>Hint 1 — 8.1, where exactly do the poll_loop calls go?</summary>

`poll_loop(true)` at the top of each iteration, before polling. `poll_loop(false)`
at the bottom — but only on iterations that are going round again. Think about what
closing the bracket would mean on the iteration where the task completed.
</details>

<details>
<summary>Hint 2 — 8.2, what do I do with the `Context`?</summary>

Nothing. That is the point. There is no waker to register, so the argument is unused
(hence the underscore in its name).
</details>

<details>
<summary>Hint 3 — `or!` leaks operations / the host complains about handles</summary>

8.3. When a combinator drops the losing future, the host still has it registered.
</details>

<details>
<summary>Hint 4 — 8.6 vs 8.7: what is the difference?</summary>

`next()` waits for something strictly newer than what is here now. `stream()` starts
*at* the present and then advances one position per value, so a consumer sees each
update exactly once.
</details>

<details>
<summary>Hint 5 — 8.8, a consumer task misses updates</summary>

Your comparison is probably `==`. If the channel advanced twice while this task was
not polled, the counter has already gone past the value being waited for — and it
must still be considered ready.
</details>

## Solution

```bash
git diff ws/simulator/08-bot-async..main -- bot/src/async_framework.rs
```
