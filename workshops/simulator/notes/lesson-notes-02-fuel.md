# Speaker notes — Lesson 02: Fuel is the clock

**Branch** `ws/simulator/02-fuel` · **Slot 1**, 0:50–1:35 (45 min) · ~30 min hands-on

This is the most important lesson of the day, and the smallest: about 80 lines. If
you take only one thing home from this workshop, I would like it to be this one.

**Goal.** You turn "instructions the robot has executed" into "microseconds of
simulated time", in a way that is exact, monotonic, and lets the host *also* charge
time for work the robot did not do itself.

---

## Removed on this branch

`sim/executor/src/wasm_host.rs`

- `FUEL_UNIT_NS`, `fuel_for_time_us`, `time_us_for_fuel`
- `BotHost::{check_fuel, setup_current_time, current_time, skip_fuel, skip_time, set_current_time}`

I kept every field, every device-operation body, the `bindgen!` invocation
*including* its `store` flag, and the `HostWithStore` forwarding impls with their
`get_fuel()` calls.

Why the `store` flag stays: removing it changes the shape of the generated host
traits (`Host` with `&mut self` instead of `HostWithStore` with `Access`), which would
turn this into a 60-line refactor of the forwarding impls rather than a clock
exercise. So you get the plumbing for free, and I explain it.

## Setting it up (4 min)

I am not going to re-explain fuel. Instead, here is the question:

> `BotHost` has three fields: `total_simulation_time`, `current_fuel`,
> `skipped_fuel`. Write `current_time()`.

*(Put the question on the board.)*

These are the three facts you need, and nothing more:

1. The store starts with `fuel_for_time_us(total_simulation_time)` units.
2. Wasmtime tells us the **remaining** balance, via `store.get_fuel()`.
3. Some simulated time is spent by the *host* on the robot's behalf — a 40-character
   log line costs 4 ms of serial transmission that consumed no WASM instructions at
   all. That is what `skipped_fuel` is for.

## The three ideas

**1. The clock runs backwards.**

```rust
let remaining_fuel = self.current_fuel - self.skipped_fuel;
Ok(self.total_simulation_time - time_us_for_fuel(remaining_fuel))
```

We do not accumulate elapsed time; we *subtract the unspent budget from the total*.
Tracking elapsed time in a counter is a very natural first idea, and it breaks the
first time the host needs to charge time. If you went that way, it is worth seeing
exactly where it breaks.

**2. `skipped_fuel` is a debit account.**

`skip_time(t)` converts microseconds to fuel and adds it to `skipped_fuel`. The
robot's balance is untouched — Wasmtime still thinks it has the fuel — but *our*
clock has moved forward. This is how `write_line` charges 100 µs per character and
`write_file` 10 µs per byte.

And this is the point I care most about: **this is what makes logging honest.** On a
real robot a debug `println!` over a serial line costs milliseconds and wrecks your
control loop. Most simulators let you log for free and therefore lie to you. We
charge for it. Lesson 09 is built on this.

**3. Setting the clock forward is solving for `skipped_fuel`.**

`set_current_time(t)` is the inverse: compute the fuel that *should* remain to make
the clock read `t`, then set `skipped_fuel` so it does. The comment on `main`
shows the two-line derivation; it is worth reading it rather than guessing.

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

## The `store` flag (3 min)

*(Keep this short: it is not where the hands-on time should go.)*

This part is already written for you. It is a `bindgen!` detail, not an insight:

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

Let's diff against the solution and look at two things only:

- the backwards subtraction in `current_time()`;
- `set_current_time` as the inverse of `current_time`.

Then let's run a real robot and look at the log timestamps:

```bash
cargo run --release -p sim -- run --cli -l -i bots/bot.wasm
```

Look at the timestamps in `log.txt`: those microseconds were *computed from
instruction counts*. Nobody measured anything.

Next: "the clock now knows what time it is. But nothing has moved — the physics has
not ticked once. Who decides when it ticks?"

---

## If something goes wrong

- **It works until the host charges time, then the clock is wrong.** You are probably
  accumulating elapsed time instead of subtracting the remaining budget from the
  total. This is a useful thing to run into: it shows why the clock runs backwards.
- **The monotonicity test passes but the `write_line` test fails.** Check that
  `current_time` subtracts `skipped_fuel`; without it, log lines cost nothing.
- **The clock does not move between calls.** The fuel has to be resampled on every
  call, not read once and cached in `BotHost::new`. `setup_current_time` is already
  called for you at the top of every operation, so look at whether its argument is
  actually used.
- **The race lasts 50 microseconds** (or the times are off by a factor of a
  million). Look at `fuel_for_time_us`: it is `time_us * 1000 / FUEL_UNIT_NS`
  (µs → ns → fuel units), a multiplication, not a division.
- **A subtraction underflows in `set_current_time`.** That happens if it is asked to
  move *backwards*. On `main` both guards return an error. Why an error and not a
  clamp? *(Ask the room.)* Because a backwards clock is a bug in the host, not a
  condition to tolerate.
