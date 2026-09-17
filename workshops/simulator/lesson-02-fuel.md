# Lesson 02 — Fuel is the clock

> **Goal.** Turn "instructions the robot has executed" into "microseconds of
> simulated time" — exactly, reproducibly, and in a way that also lets the host
> charge time for work the robot did not do itself.

This is the smallest lesson of the day and the most important one. About 80 lines.

## The setup

A real robot has a CPU, and its control loop takes time to run. That latency *is part
of the robot's behaviour*: a slow PID is a different robot from a fast PID, even with
identical constants. So a simulator that pretends computation is free has deleted the
most interesting failure mode in robotics.

Measuring wall-clock time would make the simulation depend on your laptop, on whether
you have a debugger attached, on what else is running. Instead we ask Wasmtime to
charge the robot one unit of **fuel** per instruction, and declare a 20 MHz CPU:

```rust
const FUEL_UNIT_NS: u64 = 50;   // one instruction, 50 ns
```

Fuel consumed *is* elapsed robot time. Deterministic, machine-independent, and it
gets more expensive exactly when the robot does more work.

The store is granted `fuel_for_time_us(total_simulation_time)` units up front, and
Wasmtime reports the **remaining** balance via `store.get_fuel()`. Your job is to turn
that balance into a clock.

## Your task

Everything is in `sim/executor/src/wasm_host.rs`, marked `// EXERCISE 2.x`.

| | What |
|---|---|
| 2.1 / 2.2 | `fuel_for_time_us` and `time_us_for_fuel` — unit conversion, and mind the µs/ns mismatch |
| 2.3 | `check_fuel` — the out-of-fuel test |
| 2.4 | `setup_current_time` — resample the clock on entry to a host call |
| 2.5 | **`current_time`** — the heart of it |
| 2.6 / 2.7 | `skip_fuel` / `skip_time` — charge time for host-side work |
| 2.8 | `set_current_time` — the inverse of 2.5 |

`BotHost` has three fields that matter:

```rust
total_simulation_time: TimeUs,   // the whole race
current_fuel: u64,               // what Wasmtime says is left
skipped_fuel: u64,               // what the host has spent on the robot's behalf
```

Why `skipped_fuel` exists: a 40-character log line costs 4 ms of serial transmission
that consumed *no WASM instructions at all*. The robot must still be charged for it.

## How to verify

```bash
cd sim
cargo test -p executor
```

Sixteen tests are red. They check that the clock starts at zero, is derived from
consumed fuel, never runs backwards, and that `write_line` of N characters advances
it by exactly N × 100 µs.

Then watch it work on a real robot:

```bash
cargo run --release -p sim -- run --cli -l -i bots/bot.wasm
```

Those microseconds in the log were *computed from instruction counts*. Nothing was
measured.

## Hints

<details>
<summary>Hint 1 — I am trying to accumulate elapsed time and it keeps going wrong</summary>

Good. That is the productive mistake.

You cannot accumulate: nobody tells you how much time passed, only how much budget is
*left*. Go the other way round — the clock is the total race length minus whatever the
unspent budget is worth. The clock effectively runs backwards out of a budget.
</details>

<details>
<summary>Hint 2 — what exactly counts as "unspent"?</summary>

Not just `current_fuel`. The host has already committed some of it on the robot's
behalf, and that portion is gone even though Wasmtime still thinks it is available.
</details>

<details>
<summary>Hint 3 — `set_current_time` (2.8)</summary>

Write down the relation that `current_time` implements:

```
remaining_fuel == current_fuel - skipped_fuel
```

You know what `remaining_fuel` must be for the clock to read `time`, and you know
`current_fuel`. Solve for the third one.
</details>

## Solution

```bash
git diff ws/simulator/02-fuel..main -- sim/executor/src/wasm_host.rs
```

## Where this goes next

The clock now knows what time it is — but nothing has moved. The physics has not
ticked once. Who decides when it ticks? That is lesson 03.
