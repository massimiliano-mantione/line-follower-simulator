# Speaker notes — Lesson 07: Host-side device futures

**Branch** `ws/simulator/07-async-host` · **Optional pool — deep end** · ~45 min

The cleverest code in the repository. Offer it to anyone who finished a slot early
and looked bored, and pair it with Lesson 08 if someone wants the full picture.

**Goal.** Let the robot have many device operations in flight at once, and let the
host recognise "this guest has finished a polling round and nothing is ready" so it
can fast-forward simulated time instead of letting the robot burn fuel spinning.

---

## Removed on this branch

`sim/executor/src/wasm_host.rs`

- `device_operation_async`, `device_poll`, `poll_loop`, `forget_handle`, `update_futures`
- `WakeupPoint::{set_time, disable, clear}`, `FutureReadyCondition::wakeup_point`

Kept: `FutureValueRequest`, `FutureValueReadyTime` with its hand-written `Ord`, the
three indices, the `WakeupPoint` enum, `FutureOperation::compute_value`.

## Framing (8 min)

Start from the robot's problem. It wants to read the left and right sensor banks
concurrently (`async_api::get_line_sensors` does exactly this with `zip!`). With
blocking calls, reading both costs two waits. With futures, one.

So the guest gets four host functions: start an operation and get a handle, poll a
handle, forget a handle, and — the odd one out — `poll_loop(start: bool)`.

Ask what `poll_loop` could possibly be for. Nobody guesses it, and that is the point.

Here is the problem it solves. The guest's executor is a plain `loop { poll all
tasks }` with no waker (Lesson 08). So when nothing is ready, it spins. Each spin
burns instructions, which burn fuel, which *advances simulated time* — so the robot
would eventually reach its deadline, but it would spend its entire 60-second race
budget polling instead of racing. A robot that awaits a 20 ms sleep would pay 20 ms
of *CPU* for it, when a real MCU would idle or sleep.

The fix is a handshake. The guest brackets each polling round:

```rust
loop {
    poll_loop(true);                    // "I am starting a round"
    if root_task.poll(cx).is_ready() { break; }
    poll_loop(false);                   // "round over, nothing was ready"
}
```

- `poll_loop(true)` → the host makes ready any futures whose time has come
  (`update_futures`) and clears the wakeup point.
- during the round, every `device_poll` that returns `Pending` records *when* that
  future will be ready, keeping the **earliest** such time.
- `poll_loop(false)` → if a wakeup point survived the round, nothing was ready, so
  the host jumps simulated time straight to it and steps the physics there.

And the crucial detail: any `device_poll` that returns `Ready` calls
`first_wakeup_point.disable()`. If *anything* was ready this round, the guest made
progress, so there must be no fast-forward — the robot has real work to do and must
be allowed to do it. `WakeupPoint::Disabled` is sticky; `set_time` on a disabled
point does nothing.

That three-state enum (`AtTime` / `Missing` / `Disabled`) is the whole algorithm, and
it is worth putting on the board as a state machine.

## The pieces

**`device_operation_async`** — allocate an id, compute the ready condition (the same
`ready_condition` from Lesson 03 — reuse, do not reimplement), insert into
`futures_by_id` plus *one* of the two secondary indices, and return
`FutureHandle { id, ready_at }`.

Two indices because there are two kinds of waiting: `futures_by_ready_time` is a
`BTreeSet` ordered by `(ready_at, id)` so the host can `take_while(ready_at <= now)`
and touch only the futures that have matured; `futures_by_activity` holds the ones
waiting on the enable signal, which has no deadline. Ask why the hand-written `Ord`
includes the id: because two futures can mature on the same tick, and a `BTreeSet`
would silently collapse them into one.

**`device_poll`** — three states. `Pending` records the wakeup point; `Ready` returns
the value, marks it `Consumed` and disables the wakeup point; `Consumed` is an error,
because polling a future after it resolved is a bug in the guest's executor.

**`update_futures`** — two passes, one per index. Note that it computes each value at
its **`ready_time`, not at the current time**:

```rust
let value = f.operation.compute_value(&self.stepper, &self.stepped_data, ready_time);
```

The sample is taken when the device would have sampled it, not when the robot got
around to collecting it. A robot that polls late gets stale data, exactly as it would
on hardware. This is the detail that makes the async path honest rather than a
shortcut, and it is easy to get wrong in a way no test would catch.

**`forget_handle`** — removes from all three indices. Called from `FutureValue::drop`
on the guest side, so dropping a future in Rust cancels the operation in the host.
`or!(sleep_for(MAX_TIME), race_task())` relies on this: when the race task wins, the
sleep future is dropped and the host stops tracking it.

## Verification

```bash
cd sim && cargo test -p executor
cargo run --release -p sim -- run --cli -i bots/bot.wasm
```

The interesting test is the fuel assertion: an async robot that awaits must finish
with a large fuel balance. Without the wakeup-point logic it still *works* — same
trajectory, same lap time — but it burns its entire budget spinning and traps early.
The assertion is the lesson: this optimisation is invisible in behaviour and decisive
in cost.

## Notes to self

- This lesson needs Lesson 08's half of the handshake to make sense. If someone does
  only one, steer them here first; the host side is where the reasoning lives.
- If asked why not use real wakers: because a waker would have to be able to *wake*
  the guest, and the guest is a synchronous WASM call stack the host cannot re-enter.
  The `poll_loop` bracket is a waker turned inside out — instead of the future waking
  the executor, the executor tells the host it is about to wait.
