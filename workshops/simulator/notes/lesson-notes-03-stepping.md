# Speaker notes — Lesson 03: On-demand physics ticks

**Branch** `ws/simulator/03-stepping` · **Slot 2**, 1:50–2:40 (50 min) · ~30 min hands-on

The hardest lesson, and the one that makes the architecture click. It is also the one
to shorten if you are running late — walk the solution at minute 22 instead of 30.

**Goal.** The robot's calls are the *only* thing that advances the world. Implement
the stepping engine: a blocking device call figures out when its value will be ready,
runs the physics forward to that instant, and tells the clock where it landed.

---

## Removed on this branch

`sim/executor/src/wasm_host.rs`

- `BotHost::{step, step_until_time}`
- body of `device_operation_blocking`
- `DeviceOperationExt::{ready_condition, ready_steps}`
- body of `set_motors_power`

`sim/sim/src/app_builder.rs`

- the headless `TimestepMode::Fixed { dt, substeps: 1 }` and `Time::<Virtual>::pause()`

Kept: `SteppedData`, `READY_STEPS_GYRO` (2), `READY_STEPS_IMU_FUSED` (10), and the
`CustomTransformPropagation` wiring with its comment.

## Framing (5 min) — do this on the whiteboard, it saves ten minutes later

Draw a timeline with 500 µs tick marks. Put the robot's activity *above* it and the
physics ticks *below* it.

The robot calls `sleep_for(1000)` at t = 1300 µs. Ask the room: what has to happen?

Walk it together:

1. Read the fuel → the clock says 1300 µs. (Lesson 02 did this.)
2. Ask the *operation* when it will be ready → 1300 + 1000 = 2300 µs.
3. Run the physics forward until the next tick would overshoot 2300 — so ticks at
   1500, 2000, and stop (2500 > 2300).
4. Tell the clock we are now at 2300 µs, by charging the difference to
   `skipped_fuel`.
5. Return.

Then the important observation: **between step 1 and step 5, no wall-clock time
matters and no frame was rendered.** The Bevy app is a subroutine of the robot.

Now the second case, which is the one they will get wrong: the robot calls
`get_line_sensors()` at t = 1300 µs. A real sensor is sampled by an interrupt on a
fixed schedule — you cannot read it *between* samples. So the answer is not "now",
it is "at 1500 µs, the next sample boundary". The robot pays 200 µs for asking at an
awkward moment. That is `ready_condition`, and it is why a robot that polls faster
than the sample rate gains nothing.

## The pieces

**`step()`** — one tick, plus the sampling schedule:

```rust
self.stepper.step();
let steps = self.stepper.get_step_count() as u32;
if steps % READY_STEPS_GYRO == 0 { self.stepped_data.gyro_data = self.stepper.get_gyro(); }
if steps % READY_STEPS_IMU_FUSED == 0 { /* imu */ }
self.update_futures(self.stepper.get_time_us());
```

Why the modulo: not every sensor is available every tick. The gyro updates every
2 steps (1 kHz at the default period), the fused IMU every 10 (200 Hz). `SteppedData`
holds the *latched* values — reading the gyro gives you the last sample, not a fresh
one, exactly like a real device. Line sensors and motor angles, by contrast, are read
straight off the stepper because they are cheap and continuous.

(`update_futures` is a no-op for the blocking path — it belongs to Lesson 07. Tell
them to ignore it.)

**`step_until_time(target)`** — the one-liner with the subtle condition:

```rust
while self.stepper.get_time_us_at_next_step() <= target_time { self.step(); }
```

It is `<=` on the time *after* the next step, never overshooting the target. Someone
will write `while get_time_us() < target` and drift by one tick; the test catches it.

**`ready_condition`** — per-operation:

- sensor reads: round the clock up to the next step boundary (and out by
  `ready_steps()` for the slow devices);
- `SleepFor(d)`: `current + d`; `SleepUntil(t)`: `t.max(current)`;
- `GetTime` / `GetPeriod` / `GetEnabled`: ready *now*, no stepping — these are reads
  of host state, not devices;
- `WaitEnabled` / `WaitDisabled`: not a time at all, but a *condition*. Hence the
  `FutureReadyCondition` enum having three variants rather than being a `u32`.

That last one is worth a sentence: waiting for the start signal cannot be expressed
as a deadline, so the type system forces us to model it honestly.

**`device_operation_blocking`** — assemble the five steps from the whiteboard. The
one non-obvious line:

```rust
let end_time = self.stepper.get_time_us().max(start_time);
```

Stepping may leave the physics clock *behind* the requested instant (we stopped short
of overshooting), so the robot's clock takes the later of the two. Ask why we do not
instead step once more: because that would let a robot buy 499 µs of physics for free
by sleeping 1 µs.

**`set_motors_power`** — three lines, and the ordering is the lesson:

```rust
let current_time = self.setup_current_time(current_fuel)?;
self.step_until_time(current_time);              // catch the world up FIRST
self.stepper.set_motor_drivers_duty_cycles(...); // then apply the new command
```

Step *before* applying. The old duty cycle must act for the whole interval the robot
spent computing; the new one takes effect from now on. Reverse those two lines and
the robot's decisions apply retroactively — it becomes slightly clairvoyant, and its
lap times improve for no honest reason.

**The headless config** in `app_builder.rs`:

```rust
*tsm = TimestepMode::Fixed { dt: step_period_us as f32 / 1_000_000.0, substeps: 1 };
app.world_mut().resource_mut::<Time<Virtual>>().pause();
```

Rapier must take exactly one fixed step per tick, and Bevy's virtual clock must not
advance on its own. This is what turns a game loop into a subroutine. If they forget
the `pause()`, the simulation still runs but is no longer deterministic — a good
thing to demonstrate if there is time.

## Verification

```bash
cd sim && cargo test -p executor
cargo run --release -p sim -- run --cli -i bots/bot.wasm
```

Tests: a blocking sleep lands on or after the deadline and on a step boundary; a
sensor read at an unaligned time returns at the next boundary. Then the real signal —
`data has N frames` with N > 0, and the bot actually moves.

## Walkthrough (15 min)

Diff, then spend the time on the whiteboard timeline rather than the code. Finish by
running the same robot at two periods and showing that the recorded frame count
changes but the *trajectory* barely does:

```bash
cargo run --release -p sim -- -p 1000 run --cli -i bots/bot.wasm
cargo run --release -p sim -- -p 200  run --cli -i bots/bot.wasm
```

That is the payoff of virtualised time: the robot's behaviour is a property of the
robot, not of the simulator's resolution.

Hook for Slot 3: "something has to be *there* to step. Next we build the robot's
body."

---

## Common wrong turns

- `while get_time_us() < target` instead of checking the time *after* the next step.
  Off by one tick, every time.
- Forgetting `set_current_time(end_time)` at the end of the blocking call. The
  physics moves but the clock does not, and the robot gets physics for free — the
  trajectory looks fine and the lap times are absurd.
- Applying motor power before stepping. Subtle, fast, and flattering to the robot.
- Treating `WaitEnabled` as `ReadyAt(now)`. Then every robot starts instantly and the
  1-second start delay vanishes.
- Trying to make `ready_condition` a method on `BotHost`. It is on
  `DeviceOperationExt` deliberately: it is a pure function of the operation and the
  clock, which is why it is testable without a host.
