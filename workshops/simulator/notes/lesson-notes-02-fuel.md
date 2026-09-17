# Speaker notes — Lesson 02: Fuel is the clock

**Branch** `ws/simulator/02-fuel` · **Slot 1**, 0:50–1:35 (45 min) · ~30 min hands-on

The most important lesson of the day, and the smallest: about 80 lines. If they get
only one thing from the workshop, this is it.

**Goal.** Turn "instructions the robot has executed" into "microseconds of simulated
time", in a way that is exact, monotonic, and lets the host *also* charge time for
work the robot did not do itself.

---

## Removed on this branch

`sim/executor/src/wasm_host.rs`

- `FUEL_UNIT_NS`, `fuel_for_time_us`, `time_us_for_fuel`
- `BotHost::{check_fuel, setup_current_time, current_time, skip_fuel, skip_time, set_current_time}`

Kept: every field, every device-operation body, the `bindgen!` invocation *including*
its `store` flag, and the `HostWithStore` forwarding impls with their `get_fuel()`
calls.

Why the `store` flag stays: removing it changes the shape of the generated host
traits (`Host` with `&mut self` instead of `HostWithStore` with `Access`), which would
turn this into a 60-line refactor of the forwarding impls rather than a clock
exercise. So they get the plumbing for free, and you explain it.

## Setting it up (4 min)

Do not re-explain fuel. Instead, put the question on the board and let them find it:

> `BotHost` has three fields: `total_simulation_time`, `current_fuel`,
> `skipped_fuel`. Write `current_time()`.

Then give them the three facts they need and nothing more:

1. The store starts with `fuel_for_time_us(total_simulation_time)` units.
2. Wasmtime tells us the **remaining** balance, via `store.get_fuel()`.
3. Some simulated time is spent by the *host* on the robot's behalf — a 40-character
   log line costs 4 ms of serial transmission that consumed no WASM instructions at
   all. That is what `skipped_fuel` is for.

## The three ideas (what to make sure they hit)

**1. The clock runs backwards.**

```rust
let remaining_fuel = self.current_fuel - self.skipped_fuel;
Ok(self.total_simulation_time - time_us_for_fuel(remaining_fuel))
```

We do not accumulate elapsed time; we *subtract the unspent budget from the total*.
People reliably try to track elapsed time in a counter first, and it reliably goes
wrong the first time the host needs to charge time. Let them discover that.

**2. `skipped_fuel` is a debit account.**

`skip_time(t)` converts microseconds to fuel and adds it to `skipped_fuel`. The
robot's balance is untouched — Wasmtime still thinks it has the fuel — but *our*
clock has moved forward. This is how `write_line` charges 100 µs per character and
`write_file` 10 µs per byte.

The point to land, and it is the one that makes people sit up: **this is what makes
logging honest.** On a real robot a debug `println!` over a serial line costs
milliseconds and wrecks your control loop. Most simulators let you log for free and
therefore lie to you. We charge for it. Lesson 09 is built on this.

**3. Setting the clock forward is solving for `skipped_fuel`.**

`set_current_time(t)` is the inverse: compute the fuel that *should* remain to make
the clock read `t`, then set `skipped_fuel` so it does. The comment on `00-base`
shows the two-line derivation; make sure they read it rather than guess.

```rust
// remaining_fuel == self.current_fuel - self.skipped_fuel
self.skipped_fuel = self.current_fuel - remaining_fuel;
```

This is the function Lesson 03 uses after a blocking sleep: the physics advanced,
so the clock must be *told* where it now is.

**And the plumbing:** `check_fuel` is the out-of-fuel test, and it is the *only*
failure mode a runaway robot has. `current_fuel <= skipped_fuel` means the budget is
spent, and the host returns an error, which becomes a WASM trap, which ends the run.
An infinite loop in the robot is not a hang — it is a 60-second race that ends.

## The `store` flag (3 min, do not let them lose time here)

This part is already written for them. Just explain it - it is a `bindgen!` detail,
not an insight:

```rust
imports: { default: store | trappable },
```

`store` makes every generated host function receive an
`Access<T, Self>` to the store instead of just the host data — which is the only way
to call `get_fuel()` from inside a host call. `trappable` lets host functions return
`Result`, which is how out-of-fuel becomes a trap. The `HostWithStore` impls at the
bottom of the file are then pure forwarding:

```rust
let current_fuel = host.as_context_mut().get_fuel()?;
host.get().device_operation_immediate(current_fuel, operation)
```

**Read the fuel at the top of every host call.** That is the discipline: the clock is
resampled on entry to each call, because between two calls the robot burned an
unknown number of instructions, and that is exactly the time we want to account for.

## Verification

```bash
cd sim && cargo test -p executor
```

Two tests, both over `MockStepper`, both instant:

- the clock is monotonic across a sequence of device operations;
- `write_line` of N characters advances the clock by exactly N × 100 µs.

## Walkthrough (12 min)

Diff against the solution and dwell on two things only:

- the backwards subtraction in `current_time()`;
- `set_current_time` as the inverse of `current_time`.

Then run a real robot and show the log timestamps:

```bash
cargo run --release -p sim -- run --cli -l -i bots/bot.wasm
```

Point at the timestamps in `log.txt`: those microseconds were *computed from
instruction counts*. Nobody measured anything.

Close with the hook for Slot 2: "the clock now knows what time it is. But nothing has
moved — the physics has not ticked once. Who decides when it ticks?"

---

## Common wrong turns

- **Accumulating elapsed time** instead of subtracting the remaining budget. Works
  until the host needs to charge time, then falls apart. This is the productive
  mistake; let it happen.
- **Forgetting `skipped_fuel` in `current_time`.** Then log lines cost nothing and
  the monotonicity test passes but the `write_line` test fails.
- **Reading fuel once and caching it** in `BotHost::new`. The whole point is
  resampling per call. (`setup_current_time` is already called for them at the top of
  every operation, so this shows up as ignoring its argument.)
- **Off-by-1000** in `fuel_for_time_us`: it is `time_us * 1000 / FUEL_UNIT_NS`
  (µs → ns → fuel units). Someone will divide instead of multiply and get a robot
  with a 50-microsecond race.
- Subtraction underflow if `set_current_time` is asked to move *backwards*. On
  `00-base` both guards return an error. Ask why an error and not a clamp: because a
  backwards clock is a bug in the host, not a condition to tolerate.
