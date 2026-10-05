# Workshop maintainer notes

*Accurate Robot Simulations with Bevy and WASM — RustLab, 360 minutes, level: intermediate.*

The branch layout, what is removed where, and how to regenerate everything.
Participants read [README.md](README.md) instead.

**Where things live.** Everything the workshop owns is under `workshops/simulator/`:
the briefs (`lesson-NN-slug.md`), the speaker notes (`notes/`), the switcher
(`lesson.sh`) and this file. The simulator and the robot stand on their own in
`sim/` and `bot/` and know nothing about the workshop — the one exception is the
test suites, which live with the code they cover because they are worth having
regardless.

## 1. What the official description commits us to

From <https://rustlab.it/talks/accurate-robot-simulations-with-bevy-and-wasm-workshop>:

| Promise | Covered by |
|---|---|
| "Implement a robotic simulator using Bevy" | `04-physics`, `05-sensors`, `06-replay`, `10-track` |
| "Manage deterministic time virtualization" | `02-fuel`, `03-stepping` — the centrepiece |
| "Collect telemetry data sets from the simulation and explore them" | `09-telemetry` |
| "Sandbox robot logic using WASM components separately from the simulator" | `01-wasmtime` (demo), `12-new-device` |
| Prereq: "no_std and embedded async patterns" | `07-async-host`, `08-bot-async` |
| "Contrast this Bevy/WASM approach with ROS-based implementations" | wrap-up, 10 min |
| Deterministic, ~10 kHz loops | intro + `02` |

Telemetry (`09-telemetry`) exists because it is an *advertised learning outcome*, not
because it is architecturally deep. It is cheap to run: the whole pattern already
exists in `bot/src/examples/telemetry_test.rs`.

The prerequisite line ("familiarity with no_std and embedded async patterns") is why
`08-bot-async` is pitched as a real lesson rather than a demo: this audience is
expected to know embedded async already, so it lands on prepared ground instead of
teaching futures from zero.

## 2. Day schedule (360 min)

| Time | Block | Branch |
|---|---|---|
| 0:00–0:30 | Intro + architecture walkthrough | `00-base` |
| 0:30–0:50 | **Live demo**: embedding Wasmtime, sandboxing | `01-wasmtime` |
| 0:50–1:35 | **Slot 1** — fuel is the clock | `02-fuel` |
| 1:35–1:50 | *break* | |
| 1:50–2:40 | **Slot 2** — on-demand physics ticks | `03-stepping` |
| 2:40–3:30 | **Slot 3** — Rapier bodies, joints, motors | `04-physics` |
| 3:30–4:15 | *lunch / long break* | |
| 4:15–5:05 | **Slot 4** — ray-cast light sensors | `05-sensors` |
| 5:05–5:50 | **Slot 5** — record & replay | `06-replay` |
| 5:50–6:00 | ROS contrast + physical robot demo + wrap-up | |

Each slot = ~30 min hands-on, ~12 min solution walkthrough, ~5 min slack.

**Optional pool** for anyone who finishes early — `07-async-host`,
`08-bot-async`, `09-telemetry`, `10-track`, `11-ui`, `12-new-device`. These exist
as full lessons and are *built*, not promised. Decide on the day whether to run
`08-bot-async` and `11-ui` as lessons or demote them to demos if time slips.

## 3. Branch layout

```
main                           the full application, and the reference solution
ws/simulator/01-wasmtime demo  executor/src/wasm_executor.rs
ws/simulator/02-fuel   SLOT 1  executor/src/wasm_host.rs
ws/simulator/03-stepping SLOT 2  executor/src/wasm_host.rs, sim/src/app_builder.rs
ws/simulator/04-physics SLOT 3  sim/src/bot/{model,motors}.rs, sim/src/app_builder.rs
ws/simulator/05-sensors SLOT 4  sim/src/bot/sensors/line_sensors.rs
ws/simulator/06-replay  SLOT 5  execution-data/src/lib.rs, sim/src/{data,visualizer}.rs
ws/simulator/07-async-host opt  executor/src/wasm_host.rs
ws/simulator/08-bot-async  opt  bot/src/async_framework.rs
ws/simulator/09-telemetry  opt  bot/src/examples/telemetry_test.rs
ws/simulator/10-track      opt  sim/src/track.rs
ws/simulator/11-ui         opt  sim/src/{ui,ui_runner}.rs
ws/simulator/12-new-device capstone  wit/world.wit + all five layers
```

**Twelve branches, two commits each, and those commits contain only code.** The
first removes the feature (with its `todo!()`s, degraded return values and
`#[allow]`s); the second adds nothing but the `// EXERCISE n.m` hint comments. The
split exists for the participant's IDE: `lesson.sh` puts them on `my/NN` at the
first commit and applies the second as uncommitted changes, so every hint shows up
as a change marker (§6). The briefs
and the speaker notes are ordinary files on `main`, so editing them is a normal
commit with no rebase at all. Because every lesson branch sits on top of `main`, all
of them are present in the working tree on every branch — you can read lesson 9's
brief while sitting on lesson 2.

### Invariants

1. **Every lesson branch is `main` minus exactly one feature.** Never cumulative. A
   participant who stalls in slot 1 is not locked out of slot 2.
2. **The solution to every lesson is `main`.** Each brief ends with the exact diff
   command.
3. **`Cargo.toml` and `Cargo.lock` are byte-identical on every branch.** This is what
   keeps switching cheap: identical dependency fingerprints mean Bevy, Rapier and
   Wasmtime are reused from the target directory and only the three workshop crates
   recompile. Load-bearing — see §8.
4. **Removals touch function bodies, never types.** Structs, fields, enums,
   components, plugin wiring, `add_systems` ordering and constants all stay. The
   types are the scaffolding.
5. **A lesson branch contains no prose.** Briefs and notes live on `main`. The only
   words on a branch are the `// EXERCISE n.m:` comments at the removal sites.
6. **The hints commit adds comment lines only.** Every block in it starts with
   `// EXERCISE n.m`, and every exercise number on the branch appears in it. The
   comments that justify an `#[allow]` ("Used by EXERCISE 4.3 once implemented")
   are *not* hints: they belong with their attribute, in the removal commit. §9
   checks all of this.

## 4. What the workshop adds to the project

| Path | Content |
|---|---|
| `workshops/simulator/README.md` | participant-facing: setup, lesson index, run commands |
| `workshops/simulator/MAINTAINING.md` | this document |
| `workshops/simulator/lesson.sh` | lesson switching, `my/NN` working branches, `--reset` |
| `workshops/simulator/split-hints.py` | splits a one-commit lesson into removal + hints (§7) |
| `workshops/simulator/push-all.sh` | publishes `main` and the lesson branches, then drops `refs/backup/*` (§7) |
| `workshops/simulator/lesson-NN-slug.md` | the twelve briefs |
| `workshops/simulator/notes/lesson-notes-NN-slug.md` | the thirteen speaker scripts (00 is the intro and the ROS wrap-up) |
| `workshops/simulator/samples/telemetry-example.csv` | 14k rows of real telemetry, for lesson 09 |
| `sim/bots/bot.wasm` | prebuilt robot — see below |
| `sim/executor/src/wasm_host_tests.rs` | 19 tests: the clock, the stepping engine, the futures registry (over `MockStepper`) |
| `sim/sim/src/bot/sensors/line_sensors_tests.rs` | 10 tests: the light model and per-segment distance |
| `sim/execution-data/tests/replay.rs` | 9 tests: replay indexing, activity status, ranking |

There are twelve briefs but thirteen notes: lesson 00 is the opening talk and the
closing ROS contrast, which have no exercise. The participant-facing equivalent of
note 00 is `README.md`.

**Prebuilt robot.** The five `.wasm` files that were in `sim/` were all *untracked*
(`sim/.gitignore` ignores `*.wasm`), so every simulator-side lesson silently depended
on a wasm build step. `sim/bots/bot.wasm` is now committed — currently a copy of
`line_follower_robot-toy.wasm`, named generically so the workshop robot can be
swapped without touching a single path. It lives with the simulator rather than under
`workshops/` because the simulator needs *a* robot to be runnable at all. To swap it:
replace that one file on `main`, commit, and rebase the lesson branches (§7).

**Tests.** The repo had zero tests before this. Without a red-to-green signal you
spend each slot answering "is this right?". All 38 pass on `main`; each lesson branch
turns a known subset red:

| Lesson | Red | Lesson | Red |
|---|---|---|---|
| `02-fuel` | 16 of 19 executor | `06-replay` | 8 of 9 execution-data |
| `03-stepping` | 11 of 19 executor | `07-async-host` | 5 of 19 executor |
| `05-sensors` | 10 of 10 sim | `12-new-device` | none — see §5a |

`01-wasmtime`, `04-physics`, `08`–`11` have no red tests; they are verified by
running the app. The tests live *next to* the code they cover (via `#[path]` child
modules) so a participant reads them as the spec, and they all run against
`MockStepper` or pure functions, so they finish in milliseconds.

## 5. Per-branch removal specification

Notation: `-` removed, `=` explicitly kept. Every removal site gets an
`// EXERCISE n.m:` comment stating the goal, the invariant, and an API pointer —
never the answer.

### `ws/simulator/01-wasmtime` — demo only

`sim/executor/src/wasm_executor.rs`

- bodies of `get_robot_configuration` and `run_robot_simulation`
- = the `bindgen!` invocation in `wasm_host.rs` (everything depends on its types)

Live-coded by you. The beats: `Config::consume_fuel(true)` → `Engine` → `Store`
carrying `BotHost` → `Component::new` → `Linker` →
`define_unknown_imports_as_traps` (the sandbox story) →
`add_to_linker::<_, HasSelf<_>>` → the subtle bit: **instantiation itself consumes
fuel**, so the budget must be set *before* `instantiate` and reset right after, or
instantiation eats into simulated time.

### `ws/simulator/02-fuel` — SLOT 1

`sim/executor/src/wasm_host.rs`

- `FUEL_UNIT_NS`, `fuel_for_time_us`, `time_us_for_fuel`
- `BotHost::{check_fuel, setup_current_time, current_time, skip_fuel, skip_time, set_current_time}`
- = all fields (`current_fuel`, `skipped_fuel`, `total_simulation_time`)
- = every device-operation body, and the doc comments explaining why `store` access exists
- = the `bindgen!` invocation **including its `store` flag**, and the `HostWithStore`
  forwarding impls with their `get_fuel()` calls

Removing the `store` flag was in the original plan and is wrong: it changes the shape
of the generated host traits (`Host` with `&mut self` instead of `HostWithStore` with
`Access`), turning the lesson into a 60-line refactor of the forwarding impls rather
than a clock exercise.

Highest insight-per-line in the repo. Three ideas: remaining fuel *is* the clock;
time flows backwards out of a budget; `skipped_fuel` is how the host charges
simulated time for I/O it performs itself.

Verify: `cargo test -p executor` — monotonic clock, and `write_line` of N chars
advances the clock by exactly N × 100 µs.

### `ws/simulator/03-stepping` — SLOT 2

`sim/executor/src/wasm_host.rs`

- `BotHost::{step, step_until_time}`
- body of `device_operation_blocking`
- `DeviceOperationExt::{ready_condition, ready_steps}`
- body of `set_motors_power`
- = `SteppedData`, `READY_STEPS_GYRO`/`_IMU_FUSED`

`sim/sim/src/app_builder.rs`

- the headless `TimestepMode::Fixed { dt, substeps: 1 }` + `Time::<Virtual>::pause()`
- = the `CustomTransformPropagation` schedule wiring and its comment

Verify: `cargo test -p executor` (a blocking `sleep_for(d)` lands on a step boundary
≥ d; sensor reads snap to boundaries) then
`cargo run --release -p sim -- run --cli -i sim/bots/bot.wasm` — the bot moves.

### `ws/simulator/04-physics` — SLOT 3

`sim/sim/src/bot/model.rs`

- in `setup_bot_model`: the compound `Collider`, `RigidBody::Dynamic`, `Friction`,
  `ColliderMassProperties`, `CollisionGroups`, and the revolute `ImpulseJoint` for
  each wheel
- = every geometry constant and derived dimension (`body_world`, `bodypart_body`,
  bumper positions, …) — deriving those from config is fiddly and teaches nothing
- = **the `LineSensor` child-spawn loop** (it belongs to `05`)
- = `Velocity`, `ExternalForce`, `Transform`, `Motors`, `BotPositionDetector`
- = the `bsn!` blocks themselves: the exercise comments sit *inside* them, with a
  short hint on `template_value(...)` / `template(...)`, so the participant fills in
  physics rather than learning the macro (a plain `.insert((...))` also works)

Those last ones are kept against the original plan: removing `Velocity` makes
`compute_imu_data` panic on an empty query, sending the participant hunting through
`sensors/imu.rs` for a bug that is really in `model.rs`. Declaring a component is not
the physics lesson.

`sim/sim/src/bot/motors.rs`

- `pwm_to_torque`, body of `apply_motors_pwm`
- = `Wheel`, `Motors`, `MotorsModelPlugin` and its `RunFixedMainLoopSystems::BeforeFixedMainLoop` ordering

`sim/sim/src/app_builder.rs`

- body of `RapierPhysicsSetupPlugin::build` (gravity on −Z, `RapierConfiguration::new(0.001)`)

Ship this branch with `RapierDebugRenderPlugin` **enabled** (already present,
commented out in `CameraSetupPlugin` in `ui.rs`) so colliders are visible while debugging.

Verify: `cargo run --release -p sim -- test` and drive with WASD.
Bonus for fast finishers: the DC torque-speed curve in `pwm_to_torque`.

### `ws/simulator/05-sensors` — SLOT 4

`sim/sim/src/bot/sensors/line_sensors.rs`

- `line_reflection_attenuation`, `line_reflection`
- `TrackSimulateLine::intersection_to_sensor_value`
- body of `compute_sensor_readings` (both the 16-sensor loop and the body ray-cast
  driving out-of-track / track-end detection)
- = `LineSensor`, the `point_to_new_origin` helper, all constants as documented values
- = `sensors/bot_position.rs::compute_bot_position` **intact**, two files away, as a
  worked `cast_ray_and_get_normal` reference

Three sub-problems of increasing difficulty: ray-cast to find the segment →
per-segment analytic distance-to-line (straight / 90° / arc) → the two perceptual
models (smoothstep edge whose width *equals sensor height*; lift attenuation
collapsing toward ambient 50.0).

Verify: `cargo test -p sim` (full contrast at z = 2 mm; all ≈ 50 at z = 20 mm;
smoothstep across the edge) and `sim test` — the bottom panel already prints all 16
live values.

### `ws/simulator/06-replay` — SLOT 5

`sim/execution-data/src/lib.rs`

- `BodyExecutionData::at_time_secs`, `WheelExecutionData::at_time_secs`
- `ActivityData::{status_at_time, final_status}`, `Ord` for `BotFinalStatus` (= `PartialOrd`, which delegates to it)
- = every struct, field and enum

`sim/sim/src/data.rs` — body of `store_data`

`sim/sim/src/visualizer.rs`

- `sync_bot_body`, `sync_bot_wheel`, `sync_bot_layers`
- `BotVisualization::build_transform`, body of `spawn_bot_visualization`
- = everything in `bot/vis.rs` (~350 lines of mesh assembly: craft, not concept)

The idea: **two entity layouts for one robot, bridged only by the recording.** The
simulator's robot is flat (three rigid bodies joined by `ImpulseJoint`s); the
visualizer's is a tree (wheels as children of the body) carrying `*ExecutionData`,
shaped by what is recorded: one body `Transform` and one angle per wheel per tick.
Plus a private track copy per bot, stacked on Z layers and sorted by ranking.

Verify: `sim run -i sim/bots/bot.wasm`, press space → bot animates and wheels turn;
add a second bot → layers stack and sort.

### `ws/simulator/07-async-host` — optional, deep end

`sim/executor/src/wasm_host.rs`

- `device_operation_async`, `device_poll`, `poll_loop`, `forget_handle`, `update_futures`
- `WakeupPoint::{set_time, disable, clear}`, `FutureReadyCondition::wakeup_point`
- = `FutureValueRequest`, `FutureValueReadyTime` + its hand-written `Ord`, the three
  indices (`futures_by_id` / `_ready_time` / `_activity`), the `WakeupPoint` enum,
  `FutureOperation::compute_value`

The cleverest thing in the codebase: `poll_loop(false)` tells the host "the guest
finished a polling round and nothing was ready", so the host may fast-forward
simulated time straight to the earliest wakeup point instead of burning fuel
spinning.

Verify: the async bot finishes with more than X fuel remaining — the assertion *is*
the lesson.

### `ws/simulator/08-bot-async` — optional, deep end

`bot/src/async_framework.rs`

- body of `run_boxed`
- = the no-op waker machinery: arcane `unsafe` boilerplate, worth reading aloud and
  not worth typing
- `FutureValue::poll` and its `Drop`
- `ValueWatcher::{get, update, next, stream}`, `NextValue::poll`, `ValueStream::next`
- = `MappedValue` / `FilteredValue` (combinator craft), `async_api.rs` untouched

A single-threaded executor with **no waker at all**, because the *host* decides when
time advances — the mirror image of `07`. Only branch needing
`rustup target add wasm32-wasip2`.

### `ws/simulator/09-telemetry` — optional (advertised outcome)

`bot/src/examples/telemetry_test.rs`

- `struct TelBlock`, `TelBlock::{new, csv_spec}`
- the buffer allocation, the per-iteration `push`, the final `write_csv_file`
- = `csv::transmute_buf` (provided, `unsafe`), the whole `Pid`, the host-side CSV writer

Design a packed `#[repr(C)]` sample, declare its column spec, buffer during the race,
flush at the end. Then *explore* the CSV — this is the promised "collect telemetry
data sets and explore them".

### `ws/simulator/10-track` — optional

`sim/sim/src/track.rs`

- `TrackSegment::{transform, compute_next_origin, collider}`
- `SegmentTransform::{translate_in_direction, rotate}`
- = all mesh generation (`arc_mesh`, `ninety_deg_mesh`, `quad_mesh`, `arc_collider`)

One declarative segment list → three derived artifacts: a physics collider, a render
mesh, and the analytic distance field `05` consumes. Wrong chaining produces
spectacularly broken tracks, which is excellent feedback.

### `ws/simulator/11-ui` — optional

- `sim/src/ui.rs`: the body of `setup_egui` (the order-1 `Camera2d` +
  `RenderLayers::none()` + `PrimaryEguiContext` overlay trick)
- `sim/src/ui_runner.rs`: the `play_time_sec` advance and clamp, the fine-seek key
  handling, `RunnerGuiState::handle_new_bots`
- = `viewport_ui`, every button and panel, camera, modals, file dialog, styling

`viewport_ui` is kept because without it nothing renders in any mode, so the
participant cannot see a baseline or verify anything. The transport *buttons* are
kept and only the state mutations removed, so the exercise is "make the clock move"
rather than "rebuild an egui layout".

Weakest of the set — kept because two bits are genuinely instructive: the egui
overlay camera, and the fact that the runner UI is a **pure function of
`play_time_sec`**, which is *why* scrubbing and per-tick stepping are free.

### `ws/simulator/12-new-device` — capstone

Remove `read-gyro` and `read-imu-fused-data` from `wit/world.wit` and every layer
that handles them: regenerated `bot/src/wasm_bindings.rs`, `FutureOperation`,
`DeviceOperationExt`, `SteppedData`, `SimulationStepper`, `MockStepper`,
`RunnerStepper`, `sim/src/bot/sensors/imu.rs`, `blocking_api`/`async_api`.

Nothing in `bot/src/examples/` calls either, which is what makes this surgically
possible. A true vertical slice through all five layers — the best way to teach the
component contract.

## 5a. Conventions every removal commit follows

Learned while building these; follow them when editing.

**`todo!()` versus graceful degradation.** A `todo!()` is right where the function is
off the hot path and a panic usefully names the next thing to write. It is *wrong*
anywhere a Bevy system, a `Startup` system or an every-frame UI closure would hit it,
because the app must stay runnable — a participant who cannot launch the simulator
cannot see the baseline or verify a partial answer. There, the removal returns a
degenerate-but-valid value chosen so the missing feature is *visible*:

| Lesson | Degradation | Symptom |
|---|---|---|
| 03 | Rapier timestep left variable | runs, but non-deterministic |
| 04 | `pwm_to_torque` → `0.0`, gravity → Rapier's `-Y` | falls sideways, does not move |
| 05 | all sensors → `100.0` | robot is blind |
| 06 | `Ord` → `Equal`, `store_data` → nothing | ranking ties, "data has 0 frames" |
| 07 | `update_futures` → nothing | async robots exhaust their fuel |
| 10 | transforms → identity | whole track stacked at the origin |
| 11 | `handle_new_bots` → nothing | loading a robot does nothing |

**Formatting must be clean on every exercise branch,** for the same reason as the
warnings below: a participant who runs `cargo fmt` should only ever see their own
code move. `bot/src/wasm_bindings.rs` is exempt — it is generated by `wit-bindgen`
in its own style and carries `#[rustfmt::skip]` on its `mod` line in `bot/src/lib.rs`.

**Warnings must be clean on every exercise branch.** Removing a body orphans the
imports, constants and fields it used — and those are exactly the scaffolding the
participant needs. Suppress with a targeted `#[allow(unused_imports)]` /
`#[allow(dead_code)]` / `#[allow(unused_variables)]` **plus a comment naming the
EXERCISE that will use it**. The attribute disappears in the solution diff. A noisy
`cargo check` buries the participant's own errors.

**Lesson 12 is the deliberate exception to invariant 4** (never remove types). Its
whole point is re-adding types across eight layers, so it removes `GyroData`,
`ImuFusedData`, trait methods and struct fields. It is also the only lesson that
starts with every test green, because the assertions covering the removed devices
referenced constants that no longer exist. `EXERCISE.md` says so and marks where to
put them back.

## 5b. A latent bug found while writing the tests

`BotHost::set_current_time` rejects a target equal to the current time:

```rust
if remaining_fuel >= self.current_fuel {
    return Err(wasmtime::Error::msg("Not enough fuel to advance time"));
}
```

With `remaining_fuel == current_fuel` — "move the clock to where it already is" — this
errors, which becomes a trap that ends the run. Reachable two ways: `sleep_until(t)`
with `t` already in the past, and a blocking device read at a clock value that happens
to land exactly on a step boundary.

In practice it has not bitten: `blocking_api::get_line_sensors` routes through
`device_operation_immediate`, so device reads never take the blocking path, and no
example calls `sleep_until` with a stale deadline.

**Not fixed** — changing simulator semantics is out of scope for workshop prep, and it
is your call. `>` instead of `>=` is the whole change, plus deciding whether
`set_current_time(now)` should be a no-op or stay an error. The test
`ready_condition_rounds_up_to_a_step_boundary` covers the aligned case through the
pure function instead, so the suite does not depend on either choice.

## 6. Lesson switching, and why not worktrees

```bash
./workshops/simulator/lesson.sh                 # list, and where you are
./workshops/simulator/lesson.sh 02-fuel         # open (accepts 2, 02, 02-fuel, fuel)
./workshops/simulator/lesson.sh --reset         # start the current lesson over
./workshops/simulator/lesson.sh --solution      # back to main
```

One checkout, one target directory. Switching lesson touches only the three workshop
crates, so cargo reuses every third-party artifact (this is what invariant 3 buys)
and a switch costs about seven seconds in release instead of a Bevy rebuild.

**Participants never work on `ws/simulator/NN`.** Opening a lesson creates
`my/NN` at `ws/simulator/NN^` (the removal commit) and restores the hinted files
from the lesson tip into the working tree, unstaged. The starting commit is
recorded in `branch.my/NN.lessonBase`:

- `my/NN` still at its base means "untouched": opening the lesson rebuilds it from
  the current published version. Otherwise it holds the participant's commits and
  opening just switches to it, warning if the lesson has moved on since.
- Leaving a lesson whose working tree is exactly the hints (nothing staged, tree
  identical to `ws/simulator/NN`) discards them silently; anything else must be
  committed first (`git commit -am wip`).
- `--reset` commits any uncommitted work, keeps the old branch as
  `my/NN-before-reset-<time>`, and starts over.

**A fresh clone has no local `ws/simulator/*` branches,** only
`origin/ws/simulator/*`. `lesson.sh` creates the local branch from the remote on
first use and marks it with `branch.ws/simulator/NN.lessonMirror=true`; marked
branches are reset to the remote whenever it differs, so a participant picks up a
lesson fixed during the day with `git fetch` plus `--reset`. Your own local
branches carry no mark and are never touched. (This also makes the briefs'
`git diff ws/simulator/NN..main` work in a fresh clone.)

### The worktree design that had to be abandoned

The original plan was one git worktree per lesson under `lessons/`, all sharing one
`CARGO_TARGET_DIR` so that Bevy was built once. **This is silently wrong, and it was
caught by running the test suites across all twelve worktrees:** `01-wasmtime`
reported `02-fuel`'s 16 failures, and `05-sensors` reported passes it could not
possibly have had. Twelve worktrees had produced two `executor` test binaries between
them, and cargo reported `Finished` without compiling.

The cause: for **workspace members** cargo deliberately omits the metadata hash from
artifact names, so that output files are stable (`libsim.rlib`, not
`libsim-<hash>.rlib`). Every worktree therefore collides on one fingerprint, and
whichever built most recently wins; the others see an artifact newer than their
sources and consider themselves fresh. Cargo's own documentation warns that a shared
target directory is not safe across differing source trees.

Neither escape is viable:

- **A target dir per worktree** is correct but means twelve full Bevy builds — on the
  order of 15 GB and tens of minutes *each*.
- **Touching sources before each build** forces the rebuild, but then every switch
  invalidates the previous worktree's artifacts. That is single-checkout switching
  with twelve copies of the source, and it fails silently the first time someone
  forgets.

`sccache` would genuinely fix it, at the cost of another tool to install and a cold
cache on the day. Not worth the risk for a six-hour workshop.

If you ever do want the worktrees back, the non-negotiable rule is **one target
directory per worktree**, and budget the disk and the build time accordingly.

## 7. Maintenance workflow

**Editing a brief or a speaker note** is an ordinary commit on `main`. No rebase,
nothing else to touch. This is the whole point of keeping the prose off the branches.

**Changing the code on `main`** — including swapping the prebuilt robot — means
replaying the twelve two-commit branches:

```bash
git switch main && <edit> && git commit

for b in 01-wasmtime 02-fuel 03-stepping 04-physics 05-sensors 06-replay \
         07-async-host 08-bot-async 09-telemetry 10-track 11-ui 12-new-device; do
  git rebase main "ws/simulator/$b" || break
done
git switch main
```

Each rebase replays the lesson's two commits. Conflicts only where the edit touches the
same lines a removal commit deleted — and then the fix is usually to redo that
removal by hand, since the surrounding code has changed anyway.

**Publishing.** Before a rebase, keep the old tips where you can get them back:

```bash
for b in $(git branch --format='%(refname:short)' | grep '^ws/simulator/'); do
  git update-ref "refs/backup/$b" "$b"
done
```

Once §9 is clean, `./workshops/simulator/push-all.sh` pushes `main`, force-pushes
the lesson branches with `--force-with-lease`, and only then deletes `refs/backup/*`.
It stops at the first failure, so a rejected push leaves the backups in place. Do
not `git fetch` the lesson branches just before it: the lease compares GitHub with
your remote-tracking refs, and a fetch would make them agree with whatever is there.

**If `main`'s history was rewritten** (squashed, amended, filtered), `git rebase main`
no longer knows where a lesson commit starts and tries to replay the whole old history
under it. Name the old base explicitly; every branch's parent is the same commit:

```bash
OLD=$(git rev-parse ws/simulator/01-wasmtime^)   # check it is the same for all twelve
for b in ...; do git rebase --onto main "$OLD" "ws/simulator/$b" || break; done
```

**Editing one lesson's hint comments** (the top commit):

```bash
git switch ws/simulator/05-sensors && <edit> && git commit --amend --no-edit
```

**Editing one lesson's removal commit** is easiest by squashing the two and
splitting again. `split-hints.py` rebuilds both commits with plumbing (no checkout),
keeping the removal commit's message and author and the tip's tree byte-identical:

```bash
git switch ws/simulator/05-sensors && <edit> && git commit -a --amend --no-edit
git reset --soft main && git commit -C ORIG_HEAD~1      # back to one commit
git switch main
python3 workshops/simulator/split-hints.py main ws/simulator/05-sensors
```

A hint block is a run of consecutive comment lines *added* by the branch, starting
a comment run with `// EXERCISE n.m`; anything else stays in the removal commit.
Use `--dry-run` to see what it would move. Nothing is stacked on top of a lesson
branch, so there is nothing downstream to repair.

> **Never `git add -A` / `git add .` in this repo.** `bot/src/massi.rs` is a
> deliberately untracked experiment, and a bare `git add -A` sweeps it into whatever
> commit you are making. It then vanishes from your working tree the next time you
> switch to a branch that does not have it, which looks exactly like losing the file.
> Always name the paths you mean:
> `git commit -a -- sim/executor/src/wasm_host.rs`.

After any code change, re-run the validation in §9.

## 8. Logistics risks

1. **Build time is the one thing that can sink the day.** `sim/target` here is 15 GB
   release / 37 GB debug. Participants need ~20 GB free and a **pre-warmed release
   build as mandatory homework**: clone, then `cargo build --release -p sim` in
   `sim/`. A cold Bevy build in the room is a dead hour.
2. Prerequisites to state on the signup page: Rust stable, `rustup target add
   wasm32-wasip2`, a GPU/driver that runs Bevy 0.19, ~20 GB disk.
3. `bevy`'s `dynamic_linking` feature is on — keep it, it is what makes incremental
   links tolerable. Binaries then need the dylib at runtime, which `cargo run`
   handles.
4. Invariant 3 (identical `Cargo.toml`/`Cargo.lock` everywhere) is load-bearing. If a
   lesson ever needs a new dependency, add it to `main` and rebase *all* branches.
5. The speaker notes in `notes/` are readable from any branch, so you never need to
   switch to prepare. They are not hidden from participants either — that is
   deliberate; see the note at the end of `README.md`. Just do not *project* the
   directory.

## 9. Validating the whole set

Run this after any code change. It switches through every lesson and checks four
things: that the build is warning-clean, that the code is rustfmt-clean, that exactly
the intended tests are red, and that the briefs still describe the markers actually
present in the code.

```bash
SLUGS="01-wasmtime 02-fuel 03-stepping 04-physics 05-sensors 06-replay \
       07-async-host 08-bot-async 09-telemetry 10-track 11-ui 12-new-device"

for L in main $SLUGS; do
  BR=$L; [ "$L" = main ] || BR="ws/simulator/$L"
  git switch -q "$BR"
  W=$(( $(cd sim && cargo check -p sim -p executor -p execution-data 2>&1 \
          | grep -cE "^warning") - 1 ))   # -1 for the harmless "profiles" warning
  B=$(cd bot && cargo check 2>&1 | grep -cE "^warning")
  R=$( { (cd sim && cargo fmt --all --check 2>&1); (cd bot && cargo fmt --all --check 2>&1); } \
       | grep -c "^Diff in")
  F=$(cd sim && cargo test 2>&1 | grep -E "^test result" | awk '{f+=$6} END {print f+0}')
  printf "%-14s warnings sim:%s bot:%s   rustfmt:%s   red tests:%s\n" "$L" "$W" "$B" "$R" "$F"
done
git switch -q main
```

Expected: **zero warnings and zero rustfmt diffs everywhere**, and red-test counts
matching the table in §4 (02→16, 03→11, 05→10, 06→8, 07→5, everything else 0).

A rustfmt diff on a lesson branch but not on `main` usually means a removal changed
the shape of the surrounding code (dropping two names from an import can let it fit
on one line). The fix belongs in the removal commit: squash, `cargo fmt --all`,
amend, and re-split, as in §7.

A non-zero warning count almost always means a removal orphaned an import, constant
or field and the `#[allow(...)]` plus its explanatory comment is missing — see §5a.

Beware of grepping too narrowly when checking this by hand: patterns like
`^warning: unused` miss `constant X is never used`, `fields X are never read` and
`methods X are never used`, which are precisely the ones removals cause.

### Brief-versus-code drift

Because the briefs live on `main` and the `// EXERCISE n.m:` markers live on the
branches, the two can now drift apart: rename a task in a brief and nothing tells you
the marker in the code still says the old thing. This catches it:

```bash
for L in $SLUGS; do
  N=${L%%-*}                                     # "05" from "05-sensors"
  DOC="workshops/simulator/lesson-$L.md"
  MISSING=""
  # awk, not a ${M#...} strip: "EXERCISE 8.5" word-splits into two tokens.
  MARKERS=$(git diff main.."ws/simulator/$L" \
            | grep -oE "EXERCISE [0-9]+\.[0-9]+" | awk '{print $2}' | sort -u)
  for M in $MARKERS; do
    grep -qF "$M" "$DOC" || MISSING="$MISSING $M"
  done
  [ -n "$MISSING" ] && printf "%-14s brief does not mention:%s\n" "$L" "$MISSING"
done
echo "(no output = briefs and markers agree)"
```

It only checks that every marker number appears somewhere in its brief, which is
enough to catch a renumbering or a forgotten task. It cannot tell you the *prose*
still makes sense — that is on you.

Because the match is literal, **write task numbers out in full in the briefs**: a
range like `8.4–8.7` hides 8.5 and 8.6 from the check. Both briefs that used ranges
have been expanded, and it reads better in a table anyway.

### The hints commit (invariant 6)

The second commit on each branch must add only comment lines; each chunk must start
with an `// EXERCISE n.m` line and must not continue a comment the removal commit
kept; the removal commit must not keep any hint; and no exercise may be missing from
the hints:

```bash
for L in $SLUGS; do
  B="ws/simulator/$L"
  NONC=$(git diff -U0 "$B^" "$B" | grep -E '^[-+]' | grep -vE '^(\+\+\+|---) ' \
         | grep -vE '^\+\s*//' | grep -c .)
  START=$(git diff -U0 "$B^" "$B" | awk '/^@@/ { getline;
          if ($0 !~ /^\+[ \t]*\/\/+[ \t]*EXERCISE [0-9]+\.[0-9]+/) print }' | grep -c .)
  CONT=$(git diff -U1 "$B^" "$B" | awk '/^@@/ { prev = ""; next }
          /^\+/ { if (prev ~ /^ [ \t]*\/\//) print; prev = "+"; next } { prev = $0 }' | grep -c .)
  LEFT=$(git diff -U0 main "$B^" | grep -cE '^\+\s*//+\s*EXERCISE [0-9]+\.[0-9]+\b.*:')
  MISS=$(comm -23 <(git diff main.."$B" | grep -oE 'EXERCISE [0-9]+\.[0-9]+' | sort -u) \
                  <(git diff "$B^" "$B" | grep -oE 'EXERCISE [0-9]+\.[0-9]+' | sort -u) | wc -l)
  [ $(( NONC + START + CONT + LEFT + MISS )) -ne 0 ] \
    && printf "%-14s code:%s bad-start:%s continues-comment:%s hints-left:%s missing:%s\n" \
              "$L" $NONC $START $CONT $LEFT $MISS
done
echo "(no output = every hints commit is clean)"
```

A non-zero `continues-comment` means the tail of a wrapped justification comment
("... is used by" / "// EXERCISE 4.1 and 4.2 once they are implemented.") was taken
for a hint, because that second line happens to start with `EXERCISE n.m`.
`split-hints.py` only starts a hint at the beginning of a comment run for exactly
this reason; if it fires, check that rule, then re-split the branch (§7).
