# Lesson 07 — Host-side device futures

> **Optional / deep end.** The cleverest code in the repository. Pairs with lesson
> 08, which is the guest's half of the same handshake.

> **Goal.** Let the robot have many device operations in flight at once, and let the
> host recognise "this guest finished a polling round and nothing was ready", so it
> can fast-forward simulated time instead of letting the robot burn fuel spinning.

## The setup

The robot wants to read the left and right sensor banks concurrently —
`async_api::get_line_sensors` does exactly that with `zip!`. With blocking calls that
costs two waits; with futures, one.

So the guest gets four host functions: start an operation and get a handle, poll a
handle, forget a handle, and — the odd one out — `poll_loop(start: bool)`.

**What is `poll_loop` for?** The guest's executor is a plain `loop { poll }` with no
waker (that is lesson 08). When nothing is ready, it spins. Each spin burns
instructions, which burn fuel, which *advances simulated time*. So a robot awaiting a
20 ms sleep would eventually get there — having spent 20 ms of CPU on it, when a real
microcontroller would have idled.

The fix is a handshake. The guest brackets each polling round:

```rust
loop {
    poll_loop(true);                    // starting a round
    if root_task.poll(cx).is_ready() { break; }
    poll_loop(false);                   // round over, nothing was ready
}
```

- `poll_loop(true)` matures anything due, and starts a fresh round.
- each `device_poll` that returns `Pending` records *when* that future will be ready,
  keeping the earliest.
- `poll_loop(false)` jumps simulated time straight to that point.

And the crucial rule: any poll returning `Ready` **disables** the fast-forward. If
anything was ready, the guest made progress and has real work to do.

## Your task

All in `sim/executor/src/wasm_host.rs`:

| | What |
|---|---|
| 7.1 | `FutureReadyCondition::wakeup_point` — when is it worth polling again? |
| 7.2 | `WakeupPoint::{set_time, disable, clear}` — the three-state machine |
| 7.3 | `device_operation_async` — register a future, return a handle |
| 7.4 | `device_poll` — pending / ready / bug |
| 7.5 | `poll_loop` — open and close a round |
| 7.6 | `forget_handle` — cancellation |
| 7.7 | `update_futures` — mature what is due |

Everything else is left in place: `FutureValueRequest`, `FutureValueReadyTime` with
its hand-written `Ord`, the three indices, the `WakeupPoint` enum, and
`FutureOperation::compute_value`.

Suggested order: 7.2, 7.1, 7.3, 7.7, 7.4, 7.5, 7.6.

## How to verify

```bash
cd sim
cargo test -p executor
```

Five tests are red. The interesting one is `fast_forward_costs_the_guest_no_fuel`:

```rust
assert_eq!(h.current_time().unwrap(), 20_000);
assert_eq!(h.current_fuel, fuel);
```

20 ms of simulated time passed while the guest executed **no instructions at all**.
That assertion *is* the lesson: without the wakeup logic everything still works —
same trajectory, same lap time — but the robot burns its entire budget spinning and
traps early. This optimisation is invisible in behaviour and decisive in cost.

## Hints

<details>
<summary>Hint 1 — why does `set_time` keep the earliest rather than the latest?</summary>

Because the round may have polled several pending futures, and time may only advance
to the point where the *first* of them could make progress. Jumping past it would
skip work the robot was waiting to do.
</details>

<details>
<summary>Hint 2 — why two secondary indices instead of one?</summary>

Deadlines are sortable and can be walked in order; conditions on the enable signal
are not — they have no time at all, and must be re-checked against the stepper. Also
note why the ready-time key is `(ready_at, id)` and not just `ready_at`.
</details>

<details>
<summary>Hint 3 — 7.7, my values look slightly stale or slightly fresh</summary>

Look at which time you pass to `compute_value`. A future that matured at 1500 µs but
was collected at 1900 µs must report the 1500 µs sample.
</details>

<details>
<summary>Hint 4 — a borrow-checker fight in `update_futures`</summary>

You cannot remove entries from the index while iterating it. Collect what needs
removing into a `Vec` first, then drain it.
</details>

<details>
<summary>Hint 5 — the async robot works but still runs out of fuel</summary>

The fast-forward is not firing. Either `poll_loop(false)` is not acting on the
recorded wakeup point, or something is calling `disable` when it should not — check
that only a *ready* poll disables it.
</details>

## Solution

```bash
git diff ws/simulator/07-async-host..main -- sim/executor/src/wasm_host.rs
```
