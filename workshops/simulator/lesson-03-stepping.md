# Lesson 03 — On-demand physics ticks

> **Goal.** Make the robot's own calls the only thing that advances the world. A
> blocking device call works out when its value will be ready, runs the physics
> forward to that instant, and tells the clock where it landed.

The hardest lesson of the day. If you get stuck, read the solution — this is one
where seeing it is worth more than grinding.

## The setup

Draw a timeline with 500 µs tick marks. The robot calls `sleep_for(1000)` at
t = 1300 µs. What has to happen?

```
robot:     ....calls sleep_for(1000) at 1300                     returns at 2300
           |                                                     |
time:  ----+---------+---------+---------+---------+---------+----+----
        1000      1500      2000      2500      3000      3500
physics:          tick      tick       (stop: 2500 would overshoot 2300)
```

1. Read the fuel — the clock says 1300 µs. (Lesson 02.)
2. Ask the *operation* when it will be ready: 1300 + 1000 = 2300 µs.
3. Run the physics forward until one more tick would overshoot: ticks at 1500 and
   2000, then stop.
4. Tell the clock we are now at 2300 µs.
5. Return.

Between steps 1 and 5, **no wall-clock time matters and no frame was rendered.** The
Bevy app is a subroutine of the robot.

Now the second case. The robot calls `get_line_sensors()` at t = 1300 µs. A real
sensor is sampled by an interrupt on a fixed schedule — you cannot read it *between*
samples. So the answer is not "now", it is "at 1500 µs". The robot pays 200 µs for
asking at an awkward moment, which is why a robot that polls faster than the sample
rate gains nothing.

## Your task

`sim/executor/src/wasm_host.rs`, marked `// EXERCISE 3.x`:

| | What |
|---|---|
| 3.1 | `ready_condition` — when is this operation's value available? |
| 3.2 | `ready_steps` — how many sample periods does this device need? |
| 3.3 | `device_operation_blocking` — assemble the five steps above |
| 3.4 | `set_motors_power` — three lines, and the ordering is the point |
| 3.5 | `step` — one tick, plus the slow-sensor latch |
| 3.6 | `step_until_time` — step up to, but not past, a target |

And in `sim/sim/src/app_builder.rs`:

| | What |
|---|---|
| 3.7 | fix Rapier's timestep and pause Bevy's virtual clock |

Suggested order: 3.6, 3.5, 3.2, 3.1, 3.3, 3.4, 3.7.

## How to verify

```bash
cd sim
cargo test -p executor
```

Eleven tests are red. The ones for this lesson check that a blocking sleep lands on
the deadline and steps the right number of ticks, that a sensor read at an unaligned
time returns at the next boundary, that host-state reads never step at all, and that
setting motor power catches the world up first.

The async tests go green too, since `device_poll` depends on your `step_until_time` —
don't be alarmed, that part of the file is already written.

Then run a real robot:

```bash
cargo run --release -p sim -- run --cli -i bots/bot.wasm
```

You want `data has N frames` with N > 0, and a robot that moves. Then try changing
the period and watch the frame count change while the trajectory barely does — that
is the payoff of virtualised time:

```bash
cargo run --release -p sim -- -p 1000 run --cli -i bots/bot.wasm
cargo run --release -p sim -- -p 200  run --cli -i bots/bot.wasm
```

## Hints

<details>
<summary>Hint 1 — 3.6, the stepping loop</summary>

One `while` loop over one condition. The trap is comparing the *current* stepper time
against the target; you want the time it *would* reach by stepping once more, so that
you never overshoot. `SimulationStepper` has a method for exactly that.
</details>

<details>
<summary>Hint 2 — 3.1, rounding to a boundary</summary>

You need the distance from the current clock to the previous boundary. Modulo the step
period gives you that. If it is zero you are already on a boundary and the value is
available now; otherwise move forward to the next one — and for a slow device, out by
`ready_steps()` periods rather than one.
</details>

<details>
<summary>Hint 3 — 3.3, the value comes out wrong / the clock is off by a tick</summary>

Two things to check. First, the clock at the end: stepping stops short of the target,
so the physics clock may be behind the instant the robot asked for — take the later of
the two. Second, `compute_value` takes a time argument, and for `GetTime` that is what
the robot observes.
</details>

<details>
<summary>Hint 4 — 3.5, what does "latch" mean?</summary>

`self.stepped_data` has a field per slow sensor. After stepping, check the step count
against `READY_STEPS_GYRO` and `READY_STEPS_IMU_FUSED` with `%`, and refresh the
corresponding field when it is due. Everything else reads live off the stepper.
</details>

## Solution

```bash
git diff ws/simulator/03-stepping..main -- sim/executor/src/wasm_host.rs sim/sim/src/app_builder.rs
```

## Where this goes next

Something has to be *there* to step. Lesson 04 builds the robot's body.
