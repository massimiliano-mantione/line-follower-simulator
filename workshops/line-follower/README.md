# Programming a Line Follower Robot

A hands-on workshop, run at [RustLab 2025](https://rustlab.it/talks/programming-a-line-follower-robot) and [EuroRust 2026](https://eurorust.eu/workshops/programming-a-line-follower-robot/)
by Massimiliano Mantione and Michele Mantione.

You will program a line follower robot in Rust and race it. Fast robots are
expensive, so we use a simulator instead, and a *faithful* one: wheels can lose
grip, motor responses are not linear, sensors have noise and other quirks. Your
code is compiled to WASM and runs inside the simulator as if it were the robot's
firmware, and every instruction it executes costs simulated time — so a slow
control loop is slow on the track too.

We start from a robot that barely works and improve it step by step, up to a
classical PID controller with extensions. The day ends with a competition event,
with your robots racing each other.

## What you will learn

- `no_std` Rust programming, the way it is done on a microcontroller
- async code in an embedded context: tasks, combinators and channels without an OS
- debugging a robot through realistic simulation
- telemetry: seeing what your robot decided, sample by sample, after the race
- PID controllers, and how to actually tune them

## Prerequisites

Reasonable Rust proficiency. **No prior `no_std`, embedded or async experience is
needed.**

Bring a notebook with a Rust setup (rustup, git, an editor or IDE), a GPU and driver
that run Bevy, and about 20 GB of free disk for the build cache.

## Before the workshop

Do this **at home**: a cold build of the simulator takes a long time.

```bash
git clone https://github.com/massimiliano-mantione/line-follower-simulator
cd line-follower-simulator

rustup update stable
rustup target add wasm32-wasip2     # your robot is compiled to a WASM component

cd sim
cargo build --release -p sim        # the slow part
cargo run --release -p sim -- test  # drive a robot with the arrow keys or WASD
```

If you get a window with a robot on a track, you are ready. Press `/` or `F1` there
for the command help.

## How it works

Your robot lives in [`bot/`](../../bot):

- `setup()` in `bot/src/lib.rs` returns the robot's **configuration**: its name and
  colours, and its physical shape — axle width, wheel diameter, sensor spacing and
  height, gear ratio. Change them; they change how the robot behaves.
- `run()` is the robot's **firmware**. The examples in `bot/src/examples/` are the
  starting points, from a two-sensor toy to a PID and async multi-task robots.
- It talks to the hardware through a small device API: `get_time_us`,
  `get_line_sensors`, `set_motors_pwm`, `sleep_for`, `console_log` and a few more.

Build it and race it:

```bash
cd bot && ./build.sh                # builds the WASM and copies it into sim/
cd ../sim
cargo run --release -p sim -- run -i line_follower_robot.wasm
```

The simulator runs the whole race first, then lets you replay it like a video:
pause, scrub, slow down, and load other robots next to yours.

Do not edit `wit/world.wit`: it is the contract between your robot and the
simulator.

## The day

1. **A toy robot**: two sensors, "if you see the line on one side, turn that way".
2. **Measuring the error**: using all sixteen sensors to know *how far* off the
   line you are.
3. **PID**: proportional, integral and derivative control, and why each one matters.
4. **90° turns**, the bane of all line followers, and what to do when the line is
   gone.
5. **Timing**: a late answer is a wrong answer. How fast is fast enough?
6. **Telemetry**: recording every decision in a few bytes, and reading it after the
   race.
7. **Async tasks**: splitting the robot's logic into concurrent tasks on an embedded
   async runtime.
8. **Tuning**: making sense of the PID constants instead of guessing them.
9. **The race.**

The slides are in [`presentation/`](presentation), written for
[presenterm](https://github.com/mfontanini/presenterm).

---

This repository also holds a second, independent workshop about *building* the
simulator itself: see [`../simulator`](../simulator).
