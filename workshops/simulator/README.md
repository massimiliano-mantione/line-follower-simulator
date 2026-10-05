# Accurate Robot Simulations with Bevy and WASM

A hands-on workshop. You already have the finished simulator in front of you: `main`
is the complete working application. Each **lesson** is a branch holding that same
application with exactly one feature removed, for you to put back.

Every lesson's brief lives here in `workshops/simulator/`, so you can read any of
them from any branch without switching.

---

## Before the workshop

Please do this **at home**. A cold build of Bevy takes a long time, and doing it in
the room costs you a lesson.

```bash
# 1. Get the repo
git clone https://github.com/massimiliano-mantione/line-follower-simulator
cd line-follower-simulator

# 2. Toolchain
rustup update stable
rustup target add wasm32-wasip2          # needed for the robot-side lessons

# 3. Warm the build cache. This is the slow part - do not leave it for the day.
cd sim && cargo build --release -p sim

# 4. Check it runs
cargo run --release -p sim -- test
```

You should get a window with a robot on a track. Drive it with the arrow keys or
`WASD`; the sliders on the right set the motor power. Press `/` or `F1` for the
command help.

**Requirements:** Rust stable, a GPU and driver that run Bevy 0.19, and about
**20 GB of free disk** for the build cache. All lessons share one cache, so switching
lesson does not rebuild Bevy.

---

## How the lessons work

One checkout, one branch per lesson. Switch with the helper:

```bash
./workshops/simulator/lesson.sh              # the lesson list, and where you are now
./workshops/simulator/lesson.sh 02           # open a lesson (or: 2, 02-fuel, fuel)
./workshops/simulator/lesson.sh --reset      # start the current lesson over
./workshops/simulator/lesson.sh --solution   # back to main, the full application
```

Keep your IDE open on **this one directory** the whole day; the files change under it
as you switch.

Switching recompiles only the three workshop crates — Bevy, Rapier and Wasmtime are
never rebuilt, so it takes seconds rather than the ~20 minutes of step 3.

**Opening a lesson** puts you on your own branch, `my/NN-slug`, with the lesson's
`// EXERCISE` hint comments applied as **uncommitted changes**. Your IDE's change
markers (and `git diff --stat`) therefore point at every place you need to write
code, and nothing else.

> **Commit whenever you like.** `git commit -am "wip"` keeps your work on
> `my/NN-slug`; opening the lesson again later brings you straight back to it. The
> helper refuses to switch lesson while you have uncommitted work, rather than
> dragging it into another lesson. (Untouched hints are fine: they are simply
> re-applied next time.)
>
> **Starting over:** `lesson.sh --reset` saves whatever you had on
> `my/NN-slug-before-reset-<time>` and gives you the lesson fresh. If the lesson is
> updated during the day, `git fetch` and then `--reset` to pick up the new version.

In each lesson:

- **`workshops/simulator/lesson-NN-slug.md`** is your brief: the goal, the files to
  touch, how to verify, and progressive hints.
- Every place you need to write code is marked with a comment beginning
  `// EXERCISE`, and those comments are your uncommitted changes. Nothing else has
  been changed.
- Types, struct fields, components and plugin wiring are all **left in place**. They
  are the scaffolding — read them, they tell you what the code must do.
- **The solution is `main`.** Each brief ends with the exact `git diff` command. Use
  it when you are stuck; you learn more from reading the answer at minute 25 than
  from staring at minute 29.

Lessons are **independent**. If one defeats you, the next still works.

---

## The lessons

| | Lesson | What you implement |
|---|---|---|
| demo | `01-wasmtime` | Embedding Wasmtime: engine, store, linker, sandboxing |
| **slot 1** | `02-fuel` | **Fuel as the simulated clock** |
| **slot 2** | `03-stepping` | **On-demand physics ticks** |
| **slot 3** | `04-physics` | **Rapier bodies, joints and the motor model** |
| **slot 4** | `05-sensors` | **Ray-cast light sensors and the light model** |
| **slot 5** | `06-replay` | **Recording a run and replaying it** |
| optional | `07-async-host` | Host-side device futures and the poll-loop fast-forward |
| optional | `08-bot-async` | The robot's async runtime — an executor with no waker |
| optional | `09-telemetry` | Collecting and exploring telemetry data |
| optional | `10-track` | Declarative track segments to colliders and meshes |
| optional | `11-ui` | The record-player UI |
| capstone | `12-new-device` | Add a device end to end, across all five layers |

Finished a slot early? Take one from the optional pool.

```bash
./workshops/simulator/lesson.sh 05     # short names work: 5, 05, 05-sensors, sensors
```

---

## Running things

All commands from the `sim/` directory.

```bash
# Drive the robot yourself, with live sensor readings. No WASM needed.
cargo run --release -p sim -- test

# Simulate a robot, then watch the recording
cargo run --release -p sim -- run -i bots/bot.wasm

# Headless, with the robot's log on stdout
cargo run --release -p sim -- run --cli -l -i bots/bot.wasm

# Pick a track (line, angle, turn, simple, race) and a step period in us
cargo run --release -p sim -- -t race -p 200 run -i bots/bot.wasm

# Accept robots over HTTP, competition style
cargo run --release -p sim -- serve

# Tests
cargo test
```

`sim/bots/bot.wasm` is a prebuilt robot, so the simulator-side lessons never need the
WASM toolchain.

Every lesson ships tests, and they are the fastest way to know where you stand:

```bash
cd sim && cargo test
```

To build the robot yourself (lessons `08`, `09`, `12`):

```bash
cd bot && ./build.sh      # writes line_follower_robot.wasm into sim/
```

---

## The architecture, in one picture

```
          ┌─────────────────────────────────────────┐
  bot/    │  robot logic (PID, async tasks)         │   compiled to
          │  blocking_api  /  async_api             │   wasm32-wasip2
          │  async_framework (tiny no_std executor) │
          └──────────────────┬──────────────────────┘
                             │
  wit/    ═══════════  world.wit  ═══════════════════   the contract
                             │     devices / diagnostics / robot
          ┌──────────────────┴──────────────────────┐
 executor │  wasm_host.rs    BotHost<S>             │   fuel -> time
          │  wasm_executor.rs  engine, store, linker│   sandbox
          └──────────────────┬──────────────────────┘
                             │
 execution│  ═══════  trait SimulationStepper  ═════   the seam
   -data  │            + MockStepper                   (no Bevy here)
          └──────────────────┬──────────────────────┘
                             │
          ┌──────────────────┴──────────────────────┐
  sim/    │  RunnerStepper -> AppWrapper -> Bevy App│
          │  Rapier physics · sensors · track       │
          │  recording -> ExecutionData -> replay UI│
          └─────────────────────────────────────────┘
```

The one idea the whole day hangs off:

> **The robot's WASM fuel consumption is the clock, and the physics simulation only
> ticks when the robot asks it to.**

---

## Speaker notes

`workshops/simulator/notes/` holds the speaker's script for every lesson — the
framing, the beats, the things to break on purpose, the common wrong turns.

They are not hidden from you, and they are not meant to be read ahead of a slot: the
lessons work because you hit the problems yourself. But if you miss ten minutes, or
want the reasoning behind a design after the fact, they are right there.

---

*RustLab — Massimiliano Mantione and Michele Mantione.
Maintainer notes: [MAINTAINING.md](MAINTAINING.md).*
