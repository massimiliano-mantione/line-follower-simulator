---
title: Accurate Robot Simulations with Bevy and WASM
event: RustLab 2026
location: Bologna
date: November 1 2026
authors:
  -  Massimiliano Mantione
  -  Michele Mantione
  -  Claude Code
theme:
  path: style.yaml
options:
  end_slide_shorthand: true
---

Who are we?
---

**two passionate software engineers**

*one a bit more senior than the other*

*(questions about Claude later, but welcome)*

---
Massimiliano
---

*things I worked on*

telecom switching stations\
JIT compilers\
(Mono project, then V8 in Google)\
gaming engines (Unity 3D)\
interactive, collaborative VR

🦀 distributed systems and blockchains 🦀\
🦀 operational research (vehicle routing) 🦀

---

Michele
---

Too many interests to list them!

embedded systems\
sound engineering\
visualization\
robotics

last year I got my Software Engineering degree

next one will be the Master's degree

---

🤖 Robotic Competitions as a Hobby 🎉
---

![image:width:90%](img/itlug-team.jpg)

ItLUG Robotics Team

---

About this Workshop
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

<!-- pause -->
previous workshop: *program* a **line follower**

<!-- pause -->
today: **build** the *simulator*\
*that **runs** it*

<!-- pause -->
🔧 **simulate** 🔧\
a line follower: a *simple* robot\
with a **hard real time** *control loop*

<!-- pause -->
**2㎳** late ➡ **4㎜** wrong

<!-- pause -->
🎯 *see* what the robot **decided**, and **why** 🎯

---

Tools we will Use
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

<!-- pause -->
🎮 Bevy game engine 🎮

<!-- pause -->
💪 Rapier physics engine 💪

<!-- pause -->
⚙️ Wasmtime WASM runtime ⚙️

<!-- pause -->
💻 EGUI GUI framework 💻

---

What is Bevy?
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

<!-- pause -->
a *database (in RAM)* + a *scheduler*

<!-- pause -->
**entity** ➡ *row id*

<!-- pause -->
**component** ➡ *column*

<!-- pause -->
**resource** ➡ *one-row table (singleton)*

<!-- pause -->
**system** ➡ *function* over **columns**

<!-- pause -->
**schedule** ➡ *ordered* set of **systems**

---

What is WASM?
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

<!-- pause -->
an *abstract* **machine instruction set**

<!-- pause -->
easy to

<!-- pause -->
embed\
interpret\
sandbox

<!-- pause -->
efficient execution

<!-- pause -->
**Rust** can compile to *WASM*\
*(also to WASM components)*

<!-- pause -->
**Wasmtime** ➡ the *WASM* **engine** we use

---

Other Tools
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

<!-- pause -->
physics engine\
**Rapier** ➡ provides **Bevy** *components* and *systems*\
simulates colliders, forces, friction, gravity...

<!-- pause -->
GUI framework\
**EGUI** ➡ has a **Bevy** *integration*\
*immediate* mode: *simple* to use

---

🤔 Simulating a Robot: what about Time? 🤔
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

<!-- pause -->
a robot has a **CPU**, *running* code takes **time**

<!-- pause -->
the robot *event loop* **latency** *is* **behaviour**

<!-- pause -->
how do we *simulate* it **honestly**?

<!-- pause -->
⛔ *wall clock* ⛔\
**not** reproducible

<!-- pause -->
⛔ *instantaneous loop* ⛔\
**no** latency at all

<!-- pause -->
⛔ *declared period* ⛔\
a **lie** you maintain *by hand*

---

What We Want
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

a *clock* that is...

<!-- pause -->
**deterministic**

<!-- pause -->
*independent* of the **host**

<!-- pause -->
*charged* for the **work done**

<!-- pause -->
**accurate** *(high resolution)*

<!-- pause -->
a **deterministic** *clock* makes\
a **deterministic** *physics simulation*\
**possible**


---

💡 WASM Fuel 💡
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

<!-- pause -->
one **WASM instruction** ➡ one *fuel unit*

<!-- pause -->
a **20㎒** CPU ➡ *50ns* per instruction

<!-- pause -->
fuel *consumed* **is** *elapsed* **time**

<!-- pause -->
the robot's **fuel** *is* the **clock**

<!-- pause -->
clock *resolution* ➡ **50ns**

<!-- pause -->
*physics* only **ticks**\
when the **robot** *asks*

---

🎲 Determinism 🎲
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

<!-- pause -->
time comes from **fuel**

<!-- pause -->
*fixed* physics **step**\
*1㎑ to 10㎑, default 2㎑ (500µs)*

<!-- pause -->
**100µs** is **doable**\
*but then the simulation is **slow***

<!-- pause -->
*seeded* sensor **noise**

<!-- pause -->
replay reads a **recording**\
*it does not re-simulate*

<!-- pause -->
✅ everything *(including **bugs**)* is **reproducible** ✅

---

Four Ideas
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

<!-- pause -->
the robot logic is a *sandboxed*\
**WASM component**

<!-- pause -->
a *trait* (`SimulationStepper`) is the **seam**\
between the **executor** and the **simulator**\
*(no Bevy behind it in the tests)*

<!-- pause -->
**no** *game **loop***\
*(the robot drives the schedules)*

<!-- pause -->
*simulation* and *visualization*\
two distinct **worlds**, bridged by **data**

---

Architecture
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

```
           ┌─────────────────────────────────────────┐
   bot/    │  robot logic (PID, async tasks)         │   wasm32-wasip2
           └──────────────────┬──────────────────────┘
   wit/    ═══════════  world.wit  ═══════════════════   the contract
           ┌──────────────────┴──────────────────────┐
 executor  │  BotHost<S>   engine, store, linker     │   fuel -> time
           └──────────────────┬──────────────────────┘
   sim/    ═══════  trait SimulationStepper  ═════════   the seam
           ┌──────────────────┴──────────────────────┐
 simulator │  RunnerStepper -> AppWrapper -> Bevy App│   physics
           │  recording -> ExecutionData -> replay UI│   replay
           └─────────────────────────────────────────┘
```

---

Let's See
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

```bash
cd sim
cargo run --release -p sim -- test
cargo run --release -p sim -- -t race run -i bots/bot.wasm
```

<!-- pause -->
`test`: *drive* with **WASD**

<!-- pause -->
`run`: a **recording**\
*play*, *scrub*, *step one tick*

---

How the Day Works
---

<!-- alignment: right -->
<span class="time">[00] 09:30–10:00</span>

<!-- alignment: center -->

<!-- pause -->
one **branch** per *lesson*

<!-- pause -->
`./workshops/simulator/lesson.sh 02`

<!-- pause -->
the *complete* simulator\
with **one** feature *removed*

<!-- pause -->
look for `// EXERCISE n.m:`

<!-- pause -->
the **solution** is `main`\
*peek whenever you want*

---

The Day
---

| Time | Lesson |
|---|---|
| 09:30–10:00 | `00` *introduction* |
| 10:00–10:15 | `01` **demo**: embedding Wasmtime |
| 10:15–11:00 | `02` fuel is the clock |
| 11:00–11:30 | ☕ *break* |
| 11:30–12:20 | `03` on-demand physics ticks |
| 12:20–13:00 | `04` bodies, joints, motors |
| 13:00–14:30 | 🍝 *lunch* |
| 14:30–14:45 | `04` *continued* |
| 14:45–15:35 | `05` light sensors |
| 15:35–16:00 | `06` record and replay |
| 16:00–16:30 | ☕ *break* |
| 16:30–16:50 | `06` *continued* |
| 16:50–17:50 | *open lab* |
| 17:50–18:00 | *wrap-up* |

---

📜 The Contract 📜
---

<!-- alignment: right -->
<span class="time">[01] 10:00–10:15</span>

<!-- alignment: center -->

`wit/world.wit`

<!-- pause -->
`import devices`\
`import diagnostics`\
`export robot`

<!-- pause -->
`device-value`: **8** *raw* **bytes**\
*like a device register*

<!-- pause -->
**one** `device-operation`\
*one* **choke point** for time

<!-- pause -->
`setup()`: the robot *describes*\
its own **body**

---

🎬 Live Demo 🎬
---

<!-- alignment: right -->
<span class="time">[01] 10:00–10:15</span>

<!-- alignment: center -->

## 01-wasmtime

live coding **10:04–10:13**

```bash
./workshops/simulator/lesson.sh 01
```

`sim/executor/src/wasm_executor.rs`

*one* **hunk** *at a time*

---

🔒 The Sandbox 🔒
---

<!-- alignment: right -->
<span class="time">[01] 10:00–10:15</span>

<!-- alignment: center -->

<!-- pause -->
`consume_fuel(true)`\
*to allow using fuel as time*

<!-- pause -->
`Store` holds `BotHost<S>`\
*generic* over the **stepper**

<!-- pause -->
**no** WASI, *files*, *clock*, *network*

<!-- pause -->
*unknown* imports ➡ **traps**

<!-- pause -->
`setup()` runs on a **mock** stepper\
`run()` runs on **Bevy**


---

⚠️ The Trap ⚠️
---

<!-- alignment: right -->
<span class="time">[01] 10:00–10:15</span>

<!-- alignment: center -->

<!-- pause -->
we know that\
a *runaway* robot\
**runs out of fuel**


<!-- pause -->
*instantiation* **burns fuel**

<!-- pause -->
set the fuel **before** `instantiate`

<!-- pause -->
**reset** it *after*

---

⛽ Fuel is the Clock ⛽
---

<!-- alignment: right -->
<span class="time">[02] 10:15–11:00</span>

<!-- alignment: center -->

<!-- pause -->
what we measure:\
`total_simulation_time`\
`current_fuel` `skipped_fuel`

<!-- pause -->
the clock runs **backwards**\
*time = total − unspent*

<!-- pause -->
`skipped_fuel`: a **debit** *account*

<!-- pause -->
*host* work **costs** *time* (*e.g. logging is **not** free*)\
***WASM** does not see it: account as `skipped_fuel`*

<!-- pause -->
*setting* the clock ➡ **solving** for `skipped_fuel`

<!-- pause -->
read the fuel at the **top** of *every* **host call**\
*(so you know the current time)*

<!-- pause -->
*out of fuel* ➡ **trap** ➡ race **ends**

---

⌨️ Now Coding ⌨️
---

<!-- alignment: right -->
<span class="time">[02] 10:15–11:00</span>

<!-- alignment: center -->

## 02-fuel

coding **10:22–10:48**\
➡ walkthrough at **10:48**

```bash
./workshops/simulator/lesson.sh 02
cd sim && cargo test -p executor
```

`sim/executor/src/wasm_host.rs`

*the* **smallest** lesson\
*the* **most important** one

---

☕ Break ☕
---

<!-- alignment: right -->
<span class="time">11:00–11:30</span>

<!-- alignment: center -->

back at **11:30**

---

⏱️ On-Demand Physics ⏱️
---

<!-- alignment: right -->
<span class="time">[03] 11:30–12:20</span>

<!-- alignment: center -->

the robot's **calls**\
*advance* the **world**

<!-- pause -->

robot **execution** *(fuel)* drives the **time**

<!-- pause -->

robot **interactions** with the *world*\
drives the **physics** simulation\
*(using the **stepper**)*

---

⏱️ A Timeline Example ⏱️
---

<!-- alignment: right -->
<span class="time">[03] 11:30–12:20</span>

<!-- alignment: center -->

`sleep_for(1000)` at **1300µs**

<!-- pause -->
```
robot: sleep_for(2000) at 1300                returns (resumes) at 3300
                  |                                       |
time(µs):--+---------+---------+---------+---------+---------+----+----
        1000      1500      2000      2500      3000      3500
 physics:            |         |         |         |
                  tick      tick      tick      tick      |
                                      (stop: 3500 would overshoot 3300)
```

<!-- pause -->
**no** *frame* rendered

<!-- pause -->
Bevy is a **subroutine**\
*of the robot*

---

📡 Sampling Sensor Values 📡
---

<!-- alignment: right -->
<span class="time">[03] 11:30–12:20</span>

<!-- alignment: center -->

<!-- pause -->
sensors sample on a *fixed* **schedule**

<!-- pause -->
asking *between* samples\
**costs** time

<!-- pause -->
polling *faster* **gains nothing**

<!-- pause -->
*latched* values\
*gyro* every **2** steps, *IMU* every **10**

<!-- pause -->
`WaitEnabled`: a **condition**\
*not a deadline*

---

Apply Motor Power: Order Matters
---

<!-- alignment: right -->
<span class="time">[03] 11:30–12:20</span>

<!-- alignment: center -->

<!-- pause -->
step while the *next* tick\
**does not** *overshoot*

<!-- pause -->
the robot's clock takes the **later** time\
*no free physics*

<!-- pause -->
**step** *first*\
*then* apply the **motors**

<!-- pause -->
the other way round:\
a *clairvoyant* robot 🔮

<!-- pause -->
`Time<Virtual>` **paused**\
*a game loop becomes a* **subroutine**

---

⌨️ Now Coding ⌨️
---

<!-- alignment: right -->
<span class="time">[03] 11:30–12:20</span>

<!-- alignment: center -->

## 03-stepping

coding **11:35–12:05**\
➡ walkthrough at **12:05**

```bash
./workshops/simulator/lesson.sh 03
cd sim && cargo test -p executor
cargo run --release -p sim -- run --cli -i bots/bot.wasm
```

`sim/executor/src/wasm_host.rs`\
`sim/sim/src/app_builder.rs`

*the* **hardest** lesson

---

🗄️ How Bevy Holds the World 🗄️
---

<!-- alignment: right -->
<span class="time">[04] 12:20–13:00 · 14:30–14:45</span>

<!-- alignment: center -->

<!-- pause -->
**entity** ➡ *row id*\
*our robot: three of them*\
two **wheels**, one **robot body**

<!-- pause -->
**component** ➡ *column*\
*a plain struct*

<!-- pause -->
**archetype** ➡ *table*\
*same component* **set**, *same* **table**

<!-- pause -->
*one* **contiguous** *array* per component

<!-- pause -->
add a component ➡ the entity **moves**

---

Systems are Queries
---

<!-- alignment: right -->
<span class="time">[04] 12:20–13:00 · 14:30–14:45</span>

<!-- alignment: center -->

```rust
fn apply_motors_pwm(
    pwm: Res<MotorDriversDutyCycles>,
    data: Res<ExecutionData>,
    mut wheels_query: Query<
        (&Wheel, &Transform, &Velocity, &mut ExternalForce),
        Without<Motors>>,
    mut motors_query: Query<
        (&Motors, &mut ExternalForce), Without<Wheel>>,
)
```

<!-- pause -->
read it as **SQL**

<!-- pause -->
`Without<…>`: the queries are **disjoint**

<!-- pause -->
the *borrow checker*\
at **app startup**

<!-- pause -->
*declared* access ➡ **parallel** systems

---

🤔 Why ECS? 🤔
---

<!-- alignment: right -->
<span class="time">[04] 12:20–13:00 · 14:30–14:45</span>

<!-- alignment: center -->

<!-- pause -->
*contiguous* **arrays**\
➡ **cache** *friendly*

<!-- pause -->
*objects* on the heap\
➡ **cache misses**

<!-- pause -->
*for us*: **3** bodies, *16* sensors\
➡ **irrelevant**

<!-- pause -->
what *matters* here: **composition**

<!-- pause -->
**no** *class hierarchy*\
a robot = *columns* + **systems**

---

🧭 This World 🧭
---

<!-- alignment: right -->
<span class="time">[04] 12:20–13:00 · 14:30–14:45</span>

<!-- alignment: center -->

<!-- pause -->
**Z-up**\
`Vec3::NEG_Z * 9.81`

<!-- pause -->
`RapierConfiguration::new(0.001)`\
*tolerances* at **our** *scale*

<!-- pause -->
*wheels* are **spheres**\
*one contact point, always*

<!-- pause -->
a *revolute* **joint**\
*anchors in* **two** *frames*

<!-- pause -->
**collision groups**\
*no self-collision*

---

⚙️ Friction and Motors ⚙️
---

<!-- alignment: right -->
<span class="time">[04] 12:20–13:00 · 14:30–14:45</span>

<!-- alignment: center -->

<!-- pause -->
*wheels* **grip** · *body* **slides**\
*the line is* **not** *a speed bump*

<!-- pause -->
a *DC motor*: torque **falls** with *speed*

<!-- pause -->
**gearbox**: *less speed*, **more torque**

<!-- pause -->
*Newton's* **third** law\
*reaction torque on the chassis*

<!-- pause -->
*forces* **before** *integration*

---

⌨️ Now Coding ⌨️
---

<!-- alignment: right -->
<span class="time">[04] 12:20–13:00 · 14:30–14:45</span>

<!-- alignment: center -->

## 04-physics

coding **12:31–13:00**\
➡ 🍝 lunch ➡ walkthrough at **14:30**

```bash
./workshops/simulator/lesson.sh 04
cd sim && cargo run --release -p sim -- test
```

`sim/sim/src/bot/model.rs`\
`sim/sim/src/bot/motors.rs`\
`sim/sim/src/app_builder.rs`

*drive* with **WASD**\
the *debug renderer* shows **every** collider

---

🍝 Lunch 🍝
---

<!-- alignment: right -->
<span class="time">13:00–14:30</span>

<!-- alignment: center -->

back at **14:30**

---

🦋 Determinism is Fragile 🦋
---

<!-- alignment: right -->
<span class="time">[04] 12:20–13:00 · 14:30–14:45</span>

<!-- alignment: center -->

<!-- pause -->
`ChildOf` *at spawn*\
*vs*\
`add_child` *afterwards*

<!-- pause -->
a different **archetype** *history*

<!-- pause -->
different **collider** *handles*

<!-- pause -->
float addition is **not** *associative*

<!-- pause -->
**1** *bit* ➡ **129㎳**

<!-- pause -->
determinism belongs to\
the **whole program**

---

👀 Seeing the Line 👀
---

<!-- alignment: right -->
<span class="time">[05] 14:45–15:35</span>

<!-- alignment: center -->

*IR LED* + *phototransistor*

<!-- pause -->
a *finite* **spot**\
➡ a **smooth** *edge*

<!-- pause -->
*45° cone*\
*transition width* = sensor **height**

<!-- pause -->
a *lifted* sensor\
➡ **contrast** *collapses*

<!-- pause -->
at **20㎜**: *blind*

---

Physics or Geometry?
---

<!-- alignment: right -->
<span class="time">[05] 14:45–15:35</span>

<!-- alignment: center -->

<!-- pause -->
the *line* has **no collider**

<!-- pause -->
the *ray-cast* finds the **segment**

<!-- pause -->
*geometry* gives the **distance**

<!-- pause -->
*physics*: what am I **over**?\
*math*: where **exactly**?

<!-- pause -->
*seeded* **noise**\
*real hardware is noisy*

---

⌨️ Now Coding ⌨️
---

<!-- alignment: right -->
<span class="time">[05] 14:45–15:35</span>

<!-- alignment: center -->

## 05-sensors

coding **14:52–15:20**\
➡ walkthrough at **15:20**

```bash
./workshops/simulator/lesson.sh 05
cd sim && cargo test -p sim
cargo run --release -p sim -- test
```

`sim/sim/src/bot/sensors/line_sensors.rs`

*worked example*: `compute_bot_position`\
**ray-cast** ➡ **distance** ➡ **perception**

---

📼 Record, then Replay 📼
---

<!-- alignment: right -->
<span class="time">[06] 15:35–16:00 · 16:30–16:50</span>

<!-- alignment: center -->

<!-- pause -->
*simulation*: a **batch** job\
*visualizer*: **60** *fps*

<!-- pause -->
⛔ *drawing live* ➡ **costs** *the robot time* ⛔

<!-- pause -->
per tick: **one** `Transform`\
and **two** *wheel angles*

<!-- pause -->
replay: *divide*, *clamp*, **index**

<!-- pause -->
scrubbing is **free**

---

Same Robot, Two Layouts
---

<!-- alignment: right -->
<span class="time">[06] 15:35–16:00 · 16:30–16:50</span>

<!-- alignment: center -->

<!-- pause -->
*simulation*: **flat**\
*bodies held by* **joints**

<!-- pause -->
*replay*: a **tree**\
*wheels are* **children**

<!-- pause -->
bridged by a **contract**\
*not by entities*

<!-- pause -->
`EntityFeatures` *picks*\
**components** and **systems**

<!-- pause -->
ranking: **category** *first*, then **time**

<!-- pause -->
the **leader** *floats* to the **top**

---

⌨️ Now Coding ⌨️
---

<!-- alignment: right -->
<span class="time">[06] 15:35–16:00 · 16:30–16:50</span>

<!-- alignment: center -->

## 06-replay

coding **15:42–16:00**\
☕ break **16:00–16:30**\
coding **16:30–16:38**\
➡ walkthrough at **16:38**

```bash
./workshops/simulator/lesson.sh 06
cd sim && cargo run --release -p sim -- run -i bots/bot.wasm
```

`sim/execution-data/src/lib.rs`\
`sim/sim/src/data.rs`\
`sim/sim/src/visualizer.rs`

*the* **cheapest** *function of the day*

---

🧪 Open Lab 🧪
---

<!-- alignment: right -->
<span class="time">[07–12] 16:50–17:50</span>

<!-- alignment: center -->

*pick one*

<!-- pause -->
`07` host-side **futures**

<!-- pause -->
`08` an *executor* with **no waker**

<!-- pause -->
`09` **telemetry**

<!-- pause -->
`10` a *declarative* **track**

<!-- pause -->
`11` the *record-player* **UI**

<!-- pause -->
`12` *capstone*: a **new device**, *end to end*

---

What We Built
---

<!-- alignment: right -->
<span class="time">[00] 17:50–18:00</span>

<!-- alignment: center -->

<!-- pause -->
*fuel* ➡ a **clock**

<!-- pause -->
*the clock* ➡ the **physics**

<!-- pause -->
*the physics* ➡ a **body**

<!-- pause -->
*the body* ➡ **sensors**

<!-- pause -->
*the run* ➡ a **recording**

---

vs ROS 2 + Gazebo
---

<!-- alignment: right -->
<span class="time">[00] 17:50–18:00</span>

<!-- column_layout: [1, 1] -->

<!-- column: 0 -->

<!-- alignment: center -->

✅ **here** ✅

<!-- pause -->
**computation** *cost*\
*is part of the model*

<!-- pause -->
**determinism**\
*by construction*

<!-- pause -->
a **sandboxed** *robot*

<!-- pause -->
a path to **bare metal**

<!-- column: 1 -->

<!-- alignment: center -->

<!-- pause -->
✅ **ROS** ✅

<!-- pause -->
the **ecosystem**

<!-- pause -->
*bigger* **robots**

<!-- pause -->
**interoperability**

<!-- pause -->
**maturity**

---

The Honest Summary
---

<!-- alignment: right -->
<span class="time">[00] 17:50–18:00</span>

<!-- alignment: center -->

<!-- pause -->
**ROS**:\
*will my* **algorithms** *work?*

<!-- pause -->
**this**:\
*will my* **code** *make its* **deadlines**?

<!-- pause -->
*metering* a **sandboxed** *guest*\
as the **simulation clock**

<!-- pause -->
*trading* · *game AI* · *smart contracts*\
*plugins with a* **latency budget**

---

🤖 The Physical Robot 🤖
---

<!-- alignment: right -->
<span class="time">[00] 17:50–18:00</span>

<!-- alignment: center -->

<!-- pause -->
*spherical* wheels\
*linear* attenuation\
*hand-picked* friction\
*linear* motor model

<!-- pause -->
*known* limits ➡ a **tool**

<!-- pause -->
*unknown* limits ➡ a **liability**

---

🚧 IMPORTANT NOTICE 🚧
---

<!-- pause -->
no 🦀 crab 🦀 was *harmed*\
in the *production* of this workshop

<!-- pause -->
but **several** were put to *good* **use**

<!-- pause -->
📰 Zed 📰\
🎮 Bevy 🎮\
💪 Rapier 💪\
⚙️ Wasmtime ⚙️\
📊 presenterm 📊\
💻 **WEZ** terminal 💻\
🌠 **Cosmic** desktop 🌠

<!-- pause -->
🙏 **thanks to the community** 🙏

<!-- pause -->
🦀 *the **Rust** ecosystem is **amazing*** 🦀

---

Finally...
---

```

 ████████╗██╗  ██╗ █████╗ ███╗   ██╗██╗  ██╗███████╗██╗
 ╚══██╔══╝██║  ██║██╔══██╗████╗  ██║██║ ██╔╝██╔════╝██║
    ██║   ███████║███████║██╔██╗ ██║█████╔╝ ███████╗██║
    ██║   ██╔══██║██╔══██║██║╚██╗██║██╔═██╗ ╚════██║╚═╝
    ██║   ██║  ██║██║  ██║██║ ╚████║██║  ██╗███████║██╗
    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝╚═╝

```

![image:width:70%](img/repo-url-qr-code.png)

https://github.com/massimiliano-mantione/line-follower-simulator
