# Lesson 01 — Embedding Wasmtime

> **This one is a live demo.** Sit back and watch; there is nothing you need to
> type. The code is here so you can read it afterwards, and so you can re-run the
> demo yourself later.

## The idea

The robot is not a plugin we trust. It is a **WASM component**: a sandboxed guest that
can only call the functions declared in `wit/world.wit`. No filesystem, no network,
no clock — and a hard ceiling on how many instructions it may execute.

That ceiling is the thing the whole workshop is built on. Wasmtime can charge one unit
of *fuel* per instruction, and we declare a 20 MHz CPU, so one fuel unit is 50 ns of
simulated robot time.

## What has been removed

`sim/executor/src/wasm_executor.rs` — the bodies of both functions:

| | Function | What it does |
|---|---|---|
| 1.1 | `get_robot_configuration` | instantiates the component and calls `setup()`, against a `MockStepper`, on a 1 ms budget |
| 1.2 | `run_robot_simulation` | wires in the host functions and calls `run()`, against the real Bevy simulator, on the whole race budget |

(Both sites are marked `// EXERCISE 1.x` in the code, like every other lesson — but
you are not expected to fill them in.)

## Things to watch for in the demo

1. **`Config::consume_fuel(true)`** — one line, and the reason this project works.

2. **`define_unknown_imports_as_traps`** — the sandbox. If the robot imports anything
   we did not offer, the import traps. Compare with the alternative: a robot compiled
   into the simulator as a trait object could hang it with `loop {}`, take it down
   with a panic, or delete your home directory.

3. **Instantiation consumes fuel.** Running a component's initializers executes WASM.
   A store with no fuel traps immediately, so the budget must be granted *before*
   `instantiate` — and then reset *afterwards*, or the robot's constructor quietly
   eats into its 60 seconds of race time.

4. **`setup()` runs against a mock, `run()` runs against Bevy.** Building the physics
   model needs the robot's dimensions, and the robot is what tells us them. The
   `SimulationStepper` seam is what makes that non-circular.

## Running it

```bash
cd sim
cargo run --release -p sim -- run --cli -l -i bots/bot.wasm
```

## Solution

```bash
git diff ws/simulator/01-wasmtime..main -- sim/executor/src/wasm_executor.rs
```
