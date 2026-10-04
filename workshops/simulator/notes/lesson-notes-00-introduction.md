# Speaker notes — Lesson 00: Introduction

**Branch** `main` · **Slot** 0:00–0:30 (30 min) · **Nobody codes yet**

Goal of this block: give everyone the map, so that for the rest of the day every
exercise has a place to sit. Do *not* explain any code in detail here. The single
sentence you want in their heads by minute 30 is:

> **The robot's WASM fuel consumption is the clock, and the physics simulation
> only ticks when the robot asks it to.**

Everything else in this workshop hangs off that sentence.

---

## 0. Frame (3 min)

Last year's edition of this workshop was "program a line follower". This year is the
mirror image: **we build the thing that ran your robot last year.**

Say plainly what a line follower is, in one slide, and why it is a good target: it is
the simplest robot that is still genuinely *hard* — a control loop where being 2 ms
late means being 4 mm wrong, and 4 mm is a fifth of the line width.

Then the honest reason a simulator exists at all: a competitive line follower costs
real money, and you cannot debug one by looking at it. Everything we build today is
in service of **seeing what the robot decided, and why.**

## 1. The problem with simulating a robot (5 min)

Pose it as a puzzle before showing the answer. Ask the room.

A real robot has a CPU. Its control loop takes some number of microseconds to run,
and *that latency is part of the robot's behaviour*. A slow PID is a different robot
from a fast PID, even with identical constants.

So: how do you simulate that honestly?

Let the wrong answers surface, because each one teaches something:

- **"Run the robot code and measure wall-clock time."** Then your simulation is not
  reproducible, results depend on whether Slack is open, and a debugger breakpoint
  becomes a 30-second control-loop stall.
- **"Assume the control loop is instantaneous."** Then you have deleted the single
  most interesting failure mode in robotics.
- **"Let the user declare the loop period."** Better, but it is a lie you have to
  maintain by hand, and it cannot tell you that adding a `format!` to a log line just
  blew your deadline.

What we actually want: a clock that is **deterministic**, **independent of the host
machine**, and **charged in proportion to the work the robot actually did**.

Land the answer: *WASM fuel*. Wasmtime can be told to charge one unit of "fuel" per
instruction executed. We declare a 20 MHz CPU, so one fuel unit = one instruction =
50 ns. Fuel consumed **is** elapsed robot time. Reproducible to the instruction,
identical on every machine, and it gets more expensive exactly when the robot does
more work.

That is the idea the whole day is built on, and it is Lesson 02, your first exercise.

## 2. Architecture walkthrough (12 min)

Draw this. Leave it on the whiteboard all day and point at it before every slot.

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
   -data  │            + MockStepper                   (no Bevy here!)
          └──────────────────┬──────────────────────┘
                             │
          ┌──────────────────┴──────────────────────┐
  sim/    │  RunnerStepper -> AppWrapper -> Bevy App│
          │  Rapier physics · sensors · track       │
          │  recording -> ExecutionData -> replay UI│
          └─────────────────────────────────────────┘
```

Four things to say about it, no more:

1. **The contract is a WASM component.** The robot is a sandboxed guest. It cannot
   touch the filesystem or the network; it can only call the functions we hand it. We
   even trap unknown imports. The robot is not "a plugin we trust", it is an
   untrusted competitor's binary — which is literally true when you run the server
   mode at a competition.

2. **`SimulationStepper` is the seam** (`execution-data/src/lib.rs`). It is the only
   thing the WASM host knows about the world: step it, ask it for sensor values, push
   motor duty cycles at it. Note what is *not* in that trait: Bevy, Rapier, rendering.
   That is why `MockStepper` exists — 96 lines that let us test the entire clock and
   futures machinery with no physics at all, in milliseconds. Point out that every
   test you will run today goes through it.

3. **The simulator has no game loop.** This surprises people. Bevy apps normally run
   a frame loop; ours does not. `run_bot_from_code` installs a custom runner, and
   `AppWrapper::step` hand-runs the `FixedMain` and `Main` schedules, one tick at a
   time, *when the robot asks*. `Time<Virtual>` is paused. The simulation is a batch
   job that the robot drives.

4. **Simulation and visualization are two different worlds, bridged only by data.**
   During simulation the robot is three Rapier rigid bodies joined by joints. The
   run produces a flat recording (`ExecutionData`: one body `Transform` per step,
   plus wheel angles). The visualizer then spawns its *own* entities for the robot,
   a tree with `BodyExecutionData` / `WheelExecutionData` components, shaped by
   that recording, and the UI is a pure function of `play_time_sec`. That is why scrubbing is free and why you can stack five bots
   racing side by side.

## 3. Determinism, and why we are strict about it (4 min)

Advertised outcome, so give it real airtime.

Every source of nondeterminism is either removed or seeded:

- time comes from fuel, not the wall clock;
- the physics step is fixed (100–1000 µs, i.e. **1 kHz to 10 kHz**; default 500 µs);
- sensor noise comes from a `SmallRng` seeded with a constant (`utils.rs`, seed 42);
- the recording is the whole truth — replay reads an array, it does not re-simulate.

The payoff to state out loud: **a bug is reproducible.** You can re-run the identical
race after adding telemetry and get the identical crash. Anyone who has chased a
Heisenbug on real hardware will feel this one.

## 4. Demo (4 min)

Run it, do not describe it.

```bash
cargo run --release -p sim -- test          # drive with WASD, watch the 16 sensors
cargo run --release -p sim -- -t race run -i bots/bot.wasm
```

In `test` mode point at the live sensor readout in the bottom panel — that panel is
how they will debug Lesson 05. In `run` mode use the transport controls: play, scrub,
step by a single 500 µs tick. Then say: "that is a recording, not a live simulation,
and by the end of the day you will have written the part that makes it one."

## 5. How the day works (2 min)

- One lesson per git worktree under `lessons/`, already created by
  `create-worktrees.sh`. Open your IDE with `lessons/02-fuel` as the project root.
- Each lesson is **the complete working simulator with exactly one feature removed.**
  Lessons are independent: if one defeats you, the next still works.
- Every removal site is marked `// EXERCISE n.m:` and every lesson has an
  `workshops/simulator/lesson-00-introduction.md` with hints and the verification command.
- **The solution to every lesson is the `00-base` branch**, and `workshops/simulator/lesson-00-introduction.md` gives
  you the exact `git diff` command. Use it if you are stuck — you learn more from
  reading the answer at minute 25 than from staring at minute 29.
- Five slots. Finish early and there is an optional pool: host-side futures, the
  robot's async runtime, telemetry, track geometry, the UI.

Order of the day, and tell them *why* this order — it walks up the stack:

| | | |
|---|---|---|
| Slot 1 | `02-fuel` | fuel becomes the clock |
| Slot 2 | `03-stepping` | the clock drives the physics |
| Slot 3 | `04-physics` | the physics moves a robot |
| Slot 4 | `05-sensors` | the robot perceives the track |
| Slot 5 | `06-replay` | the run becomes something you can watch |

Last 10 minutes are the ROS contrast and the physical robot.

## 6. Before they start

Check the room:

- `cargo build --release -p sim` completed (homework — if anyone skipped it, pair
  them up now rather than at minute 45);
- `./create-worktrees.sh --list` shows all lessons present;
- `rustup target add wasm32-wasip2` for the bot-side lessons.

---

## Notes to self

- Resist explaining the fuel arithmetic here. It is Lesson 02 and it lands far better
  when they have the `todo!()` in front of them.
- If the room is quieter/more junior than expected, the two lessons to protect are
  `02-fuel` and `05-sensors`; `03-stepping` is the one to shorten by walking the
  solution earlier.
- The whiteboard diagram earns its keep all day. Draw it big.

---
---

# Speaker notes — Closing block

**Slot** 5:50–6:00 (10 min) · These notes live on `main` because that is
the branch everyone has open, and because the intro and the wrap-up are one argument.

## 1. Recap in one pass (2 min)

Walk the whiteboard diagram bottom to top, one sentence each — they built this:

fuel became a clock → the clock drove the physics → the physics moved a body → the
body carried sensors → the run became a recording you can scrub.

Then the sentence from minute 30, again, because now it means something:

> The robot's WASM fuel consumption is the clock, and the physics only ticks when the
> robot asks it to.

## 2. Contrast with ROS (6 min)

Promised on the signup page, so it is not optional. Be genuinely even-handed — the
room will contain people who use ROS professionally, and overclaiming here costs you
all the credibility you built during the day.

Set the comparison up properly first. The standard stack is **ROS 2 + Gazebo**:
nodes as separate processes communicating over DDS pub/sub, with Gazebo simulating
physics and publishing sensor topics while subscribing to command topics.

### Where this approach wins

**Computation cost is part of the model.** This is the real difference, and it is the
one to lead with. ROS 2 *can* virtualise the clock — publish `/clock`, set
`use_sim_time`, and every node's timers follow simulated time. What it cannot do is
charge a node for the instructions it executed. Your control node runs at whatever
speed your laptop provides; its callback latency is wall-clock latency. So ROS can
answer "what if the sensor published at 200 Hz?" but not "what if my PID takes 300 µs
on a 20 MHz microcontroller?" — and for a line follower that second question *is* the
engineering problem.

**Determinism is by construction.** DDS is asynchronous by design; message ordering
and arrival timing vary run to run, and reproducing a failure across the full stack is
a well-known pain point. Ours is deterministic to the instruction: same binary, same
seed, same trajectory, on any machine. Note that Gazebo in lockstep mode gets much
closer than the general case — be fair about that.

**The robot is sandboxed.** A ROS node is a process with your user's privileges. Here
the robot is a WASM component whose only capabilities are the functions in
`world.wit`, with unknown imports trapped and a hard fuel ceiling. That is why
`serve` mode can accept a `.wasm` over HTTP from a stranger at a competition and run
it safely. You would not do that with a ROS node.

**Deployment path to bare metal.** The robot here is `no_std`-shaped code with a thin
device API. The same logic compiles to a Cortex-M with a real HAL behind the same
API. ROS on a microcontroller means micro-ROS, which needs an RTOS and an agent
process, and is a much larger commitment for a robot whose whole job is one control
loop. For 8-bit and small 32-bit targets it is simply not on the table.

**Weight.** One binary and a 100 KB `.wasm`, versus an install, a middleware layer,
and a process tree.

### Where ROS wins, plainly

Do not soften this part; it is most of the robotics world for good reasons.

- **Ecosystem.** Nav2, MoveIt, SLAM implementations, `tf2`, rviz, rosbag, thousands of
  hardware drivers. We wrote a line sensor. They have a lidar model with realistic
  noise, cameras with lens distortion, depth sensors, IMUs, and URDF/SDF as a
  standard way to describe a robot.
- **Scale of robot.** Manipulators, kinematic chains, multi-sensor fusion, perception
  pipelines. Our approach assumes a small robot with a tight loop and a handful of
  devices. Point at `bot/model.rs`: we hand-built four colliders and two joints. Doing
  a seven-DoF arm that way is not a good use of anyone's life; that is what URDF is
  for.
- **Interoperability and hiring.** ROS is the lingua franca. A ROS-shaped robot can be
  worked on by people who have never seen your codebase, and its tooling composes
  with everyone else's.
- **Recording and introspection.** `rosbag` records every topic and replays it into
  the live stack. Our recording is deliberately minimal and our telemetry is
  hand-rolled per robot.
- **Maturity.** Gazebo's physics and sensor models have had years of tuning against
  real hardware. Ours is a workshop project with a friction table we picked by hand.

### The honest summary

Put it as a choice of question, not a verdict:

> ROS and Gazebo answer **"will my robot's algorithms work?"**
> This answers **"will my robot's code make its deadlines?"**

Both are real questions. The second one dominates when the robot is a control loop on
a microcontroller, when determinism is a requirement rather than a nicety, or when
the logic comes from someone you do not trust. That is a narrow slice of robotics —
and it happens to be exactly the slice a line-follower competition lives in.

And the transferable point, which is the one to actually leave them with: the
technique — *metering a sandboxed guest's execution and using that meter as the
simulation clock* — is not about robots. It applies to anything where you need to
simulate code whose execution cost is part of its behaviour. Trading strategies.
Game AI. Smart contracts. Anything you run as an untrusted plugin with a latency
budget.

## 3. The physical robot (2 min)

Demo the real line follower here. The honest framing: this is what the simulator is a
model *of*, and every simplification we made today — spherical wheels, a linear
attenuation curve where reality is quadratic, a friction table picked by hand, a
linear DC motor model — is a place where the model and this object disagree. Name a
couple out loud. A simulator whose limitations you can list is a tool; one whose
limitations you cannot is a liability.

Then the repo URL, the invitation to open a PR, and thanks.

## Notes to self

- If a ROS practitioner pushes back hard, concede immediately and specifically — say
  which claim you are withdrawing. The argument only works if it is fair, and the
  distinction you actually care about (metering computation, not just virtualising the
  clock) survives every fair objection.
- Do not let this block run long. If you are at 5:55, cut straight to the honest
  summary and the physical robot.
