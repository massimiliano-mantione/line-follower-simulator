# Speaker notes — Lesson 06: Record and replay

**Branch** `ws/simulator/06-replay` · **Slot 5**, 5:05–5:50 (45 min) · ~30 min hands-on

This lesson explains the shape of the whole application, and it is a good one to end
on, because what you build is the thing you have been looking at all day.

**Goal.** You record a race into a flat array, then render it as a pure function of
playback time — and stack several robots on the same track, ranked.

---

## Removed on this branch

`sim/execution-data/src/lib.rs`

- `BodyExecutionData::at_time_secs`, `WheelExecutionData::at_time_secs`
- `ActivityData::{status_at_time, final_status}`
- `Ord` for `BotFinalStatus` (`PartialOrd` stays: it just delegates to `Ord::cmp`,
  so there is one ranking to write)

`sim/sim/src/data.rs` — body of `store_data`

`sim/sim/src/visualizer.rs` — `sync_bot_body`, `sync_bot_wheel`, `sync_bot_layers`,
`BotVisualization::build_transform`, body of `spawn_bot_visualization`

I kept every type and field, and all ~350 lines of mesh assembly in `bot/vis.rs`.
That is craft, not concept, and re-deriving a robot out of cylinders and spheres would
eat the whole slot.

## Framing (7 min)

The simulation is a batch job that runs as fast as the CPU allows, and the
visualizer is a 60 fps interactive app with a scrub bar. **How do you connect them?**

*(Ask the room, and let the tempting answer come up first.)*

The tempting answer is *run the simulation live and draw it*. It is worth a minute to
see why it does not work:

- the simulation is not real-time — it runs a 60-second race in a couple of seconds,
  and at a 100 µs period it runs *slower* than real time;
- you cannot scrub backwards through a physics engine;
- you cannot show five robots that raced at different moments, side by side;
- rendering would have to be inside the robot's own clock, so **drawing a frame would
  cost the robot simulated time**.

So: **record, then replay.** The simulator writes `ExecutionData` — for each tick, one
body `Transform` and two wheel angles — and the visualizer never touches physics
again.

Then the architectural punchline, and this is the thing I would like you to take
away:

> **Same robot, two entity layouts: each one shaped by what drives it.**

During simulation the robot is **flat**: `setup_bot_entities` spawns the body and the
two wheels as three separate top-level entities, and `setup_bot_model` gives them
`Collider`, `RigidBody`, `Velocity`. What holds them together is an `ImpulseJoint`
per wheel, not parent/child: each is an independent Rapier rigid body and the solver
owns all three transforms. The only children are the 16 `LineSensor`s riding on the
body. There is no mesh anywhere.

During replay the robot is a **tree**: `spawn_bot_visualization` builds a
`BotVisualization` root with its own track copy, the body under it carrying
`BodyExecutionData`, and the wheels as *children of the body* carrying
`WheelExecutionData`. The hierarchy is shaped by the recording. We store the body's
full `Transform` but only one *angle* per wheel, so each wheel sits fixed at its axle
position relative to the body and `sync_bot_wheel` only turns it. Why is that
enough? *(Ask the room.)* The joint already forced the wheel to stay on the axle, so
its position was never information worth recording.

The bridge between the two is a contract, not shared entities: one `Transform` plus
two `f32` per tick. The visual spawn functions (`spawn_bot_body`, `spawn_bot_wheel`)
are shared, but with `test` mode, not with the physics. In `test` mode,
`EntityFeatures::PhysicsAndVisualization` builds the flat physics robot *and* hangs
the same mesh subtrees under its entities, with no `ExecutionData`, so the meshes
just follow the solver. That is why you can drive a physically simulated robot with
the arrow keys. Look at `bot/mod.rs::BotPlugin::build` and
`app_builder.rs::AppType::entity_features()`: one `EntityFeatures` enum (`Physics`,
`Visualization`, `PhysicsAndVisualization`) decides which systems and components
exist.

(`spawn_bot_wheel` shows the two cases side by side: it places the wheel at the axle
only when it is given `WheelExecutionData`, because under a physics wheel the parent
is already in the right place.)

## The pieces

**Recording** — `store_data`, three lines, runs in the custom `BotUpdate` schedule
after `compute_imu_data`:

```rust
exec_data.body_data.steps.push(body_transform);
exec_data.left_wheel_data.steps.push(motor_angles.left);
exec_data.right_wheel_data.steps.push(motor_angles.right);
```

Notice what is *not* recorded: no sensor values, no PWM, no forces. The
recording is the minimum needed to redraw. Everything else a robot author wants to
inspect goes through telemetry, which the robot writes itself (Lesson 09) — because
the *robot* knows what is worth recording, and it pays for it in simulated time.

**Replay** — `at_time_secs`, and the only trick is that there is no trick:

```rust
let index = ((time_secs * 1_000_000.0 / self.period as f32).floor().max(0.0) as usize)
    .min(self.steps.len() - 1);
self.steps[index]
```

Divide by the period, clamp, index. *That* is why scrubbing is free, why stepping one
500 µs tick is free, and why playing backwards works. You are about to write the
cheapest function of the day, and it is the one that makes the UI feel good. What
would a naive "advance the simulation to time t" have cost? *(Ask the room.)*

Then `sync_bot_body` and `sync_bot_wheel` — the systems that apply it, once per frame,
for every entity holding the data component. Wheels convert the angle to a rotation
about `data.axis_rotation()`.

**Activity and ranking.** `ActivityData` holds three optional timestamps:
`start_time_us`, `out_time_us`, `end_time_us`, written by `AppWrapper::step` as the
race unfolds. From them:

- `status_at_time(t)` → `Waiting` / `Racing` / `EndedAt` / `OutAt`, driving the
  per-robot timer and colour in the side panel. Note that it reports time *relative to the
  start signal*, so the 1-second countdown is not charged to anyone's lap time.
- `final_status()` → the ranking key.

And `Ord for BotFinalStatus`, which is a genuinely nice little piece of design:

```rust
fn kind_rank(&self) -> usize {
    EndedAt => 0, OutAt => 1, NotEnded => 2, NotStarted => 3
}
```

Sort by *category* first, then by time within the category. Finishers beat
crashers, crashers beat robots that never finished, and within the finishers the
fastest wins. Try writing it yourself before looking at the solution: "sort robots
by how well they did" sounds trivial and is not, because the orderings are not
comparable until you say what the categories are.

**Layering** — the trick that makes multi-robot comparison work. Each robot gets its
own *private copy of the track*, one `VIS_LAYER_Z_STEP = 0.7` above the next:

```rust
Transform::from_xyz(0.0, 0.0, layer as f32 * VIS_LAYER_Z_STEP)
```

`spawn_bot_visualization` spawns a root carrying `BotVisualization`, calls
`setup_track(..., is_bottom = false)` under it — which is why the stacked copies get
a *translucent* floor (alpha 0.1) while the bottom one is opaque — and then the bot
body and wheels beneath it. `sync_bot_layers` re-sorts every frame by
`bot_final_status` and reassigns the Z offsets, so **the leader floats to the top of
the stack as the race unfolds.** That is a UI idea, not a physics one, and it is the
sort of thing that only works because replay is decoupled from simulation.

## Verification

```bash
cd sim && cargo run --release -p sim -- run -i bots/bot.wasm
```

Press space: the bot moves and the wheels turn at a rate matching its speed. `Home`,
`End`, `,` / `.` to scrub; `ctrl+shift+,` to step a single tick. Then use the `+`
button to load a second robot and watch the two track copies stack and re-sort.

If the body moves but the wheels do not turn, check that you use `axis_rotation()`;
if the body slides without rotating, check that the whole recorded `Transform` is
assigned.

## Walkthrough (12 min)

*(Load three robots, let it play, and point at the stack re-ordering. Then scrub
backwards through a crash at single-tick resolution.)*

This is the debugging tool. On real hardware you get a shaky phone video.

*(Back at the whiteboard diagram.)* Let's close the loop on the whole day: fuel became a
clock (02), the clock drove the physics (03), the physics moved a body (04), the body
carried sensors (05), and the run became a recording you can inspect (06).

---

## If something goes wrong

- **The wheels teleport to the origin.** The body wants the whole `Transform`
  (`*transform = data.at_time_secs(t)`), the wheel wants only its rotation: check
  that these are not swapped.
- **It panics on a robot that traps during `setup`.** That robot produces an
  **empty** `steps` vector, and `.min(len - 1)` underflows on it. Guard with
  `is_empty()` first, as both `at_time_secs` implementations do.
- **Every lap time is 1 second too long.** The per-robot timer should use time since
  that robot's start, not the absolute play time.
- **The winner sinks to the bottom of the stack**, hidden under four translucent
  floors. Check for the `.reverse()` after sorting in `sync_bot_layers`.
- **Playback lags slightly behind.** If you record in `Update` rather than in the
  `BotUpdate` schedule, samples land before the sensors and transforms for that tick
  have been computed, and the entire recording is one tick off.
