# Codebase Exploration Report

## Project Overview

This is a **line-follower robot simulator** written in Rust. It simulates differential-drive robots that follow a black line on a white surface. The architecture separates the robot logic (compiled to WebAssembly) from the physics simulation (Bevy game engine with Rapier 3D physics). The robot code runs inside a sandboxed WASM runtime (Wasmtime), communicating with the simulator through a WIT (WebAssembly Interface Types) contract. The project appears designed for educational/competitive contexts, supporting multiple robots on configurable tracks, PID-based control algorithms, and a rich 3D visualization with VCR-style playback controls.

## Repository Structure

```
line-follower-simulator/
  bot/                          -- Robot code crate (compiled to WASM component)
    Cargo.toml                  -- cdylib crate "line-follower-robot"
    src/
      lib.rs                    -- Entry point: implements WIT Guest trait (setup + run)
      wasm_bindings.rs          -- Auto-generated WIT bindings (large, machine-generated)
      wasm_bindings_ext.rs      -- DeviceValue accessor extensions (get/set typed fields)
      blocking_api.rs           -- Blocking (synchronous) robot API
      async_api.rs              -- Async robot API (device ops as futures)
      async_framework.rs        -- Minimal async executor + ValueWatcher channel
      massi.rs                  -- User's custom robot code (not committed yet)
      examples/
        mod.rs
        toy.rs                  -- Simplest line follower (bang-bang control)
        basic_pid.rs            -- PID-based line follower (blocking)
        pid_with_memory.rs      -- PID with out-of-line recovery memory
        telemetry_test.rs       -- CSV telemetry demonstration
        nb/
          mod.rs
          toy.rs                -- Async version of toy example
          parallel_tasks.rs     -- Async PID with parallel sensor reading + race task
          tasks_with_channels.rs -- Async PID using ValueWatcher channels
          pid_with_memory.rs    -- Async PID with memory

  sim/                          -- Simulator workspace
    Cargo.toml                  -- Workspace root (4 members)
    execution-data/             -- Shared data types crate
      src/lib.rs                -- ExecutionData, SensorsData, SimulationStepper trait, etc.
    executor/                   -- WASM execution engine crate
      src/
        lib.rs
        wasm_bindings.rs        -- Host-side auto-generated WIT bindings
        wasm_executor.rs        -- Functions to load/run WASM robot components
        wasm_host.rs            -- BotHost: implements WIT host interface for simulation
        mock_stepper.rs         -- MockStepper for config-only WASM invocation
    sim/                        -- Main simulator binary crate
      src/
        main.rs                 -- CLI entry point (Run, Test, Serve subcommands)
        app_builder.rs          -- Bevy App construction (physics, visualization modes)
        bot/
          mod.rs                -- BotPlugin (Bevy plugin)
          model.rs              -- Physics model: rigid bodies, colliders, joints, sensors
          motors.rs             -- DC motor torque model, PWM-to-force conversion
          vis.rs                -- Bot 3D mesh visualization
          sensors/
            mod.rs              -- SensorsModelPlugin
            line_sensors.rs     -- Ray-cast based line sensor simulation
            bot_position.rs     -- Track position detection (on-track, out, end)
            imu.rs              -- Gyroscope and IMU fused data computation
            motor_angles.rs     -- Motor angle tracking from wheel transforms
        track.rs                -- Track geometry: segments, colliders, meshes
        track_selection.rs      -- Predefined track definitions (Line, Angle, Turn, Simple, Race)
        runner.rs               -- RunnerStepper: bridges Bevy physics to SimulationStepper
        server.rs               -- HTTP server for accepting WASM robot code uploads
        data.rs                 -- StoreExecDataPlugin: records execution data per step
        visualizer.rs           -- Bot visualization playback (body/wheel sync to time)
        ui.rs                   -- GUI setup: egui, camera controls, help dialog
        ui_runner.rs            -- Runner GUI: VCR controls, bot management, file dialog
        ui_test.rs              -- Test mode GUI (manual PWM control)
        utils.rs                -- Side enum, EntityFeatures, NormalRandom, geometry helpers
    bevy_panorbit_camera/       -- Vendored/forked orbit camera plugin for Bevy

  wit/
    world.wit                   -- WIT interface definition (the contract between bot and sim)

  presentation/                 -- Slide deck materials (Markdown + images + scripts)
  ITEMS.md                      -- Curriculum/presentation topic checklist
  TODO.md                       -- Feature checklist
  LICENSE                       -- MIT License
  README.md                     -- Project readme
```

## Languages & Build Systems

- **Language**: Rust (edition 2024)
- **Build system**: Cargo workspaces
  - `bot/` is a standalone workspace producing a `cdylib` WASM component
  - `sim/` is a workspace with 4 member crates: `execution-data`, `executor`, `sim`, `bevy_panorbit_camera`
- **WASM toolchain**: The bot crate uses `wit-bindgen 0.46.0` to generate bindings from `wit/world.wit`. It compiles to a WASM component (cdylib target).
- **Key dependencies**:
  - **Bevy 0.16.1** -- game engine for 3D visualization and ECS
  - **bevy_rapier3d** -- Rapier 3D physics integration for Bevy
  - **Wasmtime 36.0.2** -- WebAssembly component model runtime
  - **bevy_egui / egui** -- immediate-mode GUI
  - **tiny_http** -- lightweight HTTP server
  - **futures-micro** -- minimal async combinators (bot side)
  - **pin-project-lite** -- pin projection for async types
  - **rand / rand_distr** -- deterministic sensor noise

## Architecture

The system follows a **host-guest** architecture with a strict boundary at the WIT interface:

```
+------------------+        WIT Interface         +-------------------+
|    Bot (WASM)    | <=========================>  |   Simulator Host  |
|  (line-follower- |   devices (sensors, motors)  |    (executor +    |
|   robot crate)   |   diagnostics (logging, CSV) |     sim crate)    |
+------------------+                              +-------------------+
                                                         |
                                              +----------+----------+
                                              |                     |
                                       Bevy + Rapier          Visualization
                                       (Physics ECS)          (3D + egui UI)
```

**Execution flow**:

1. The simulator loads a `.wasm` component file (the compiled bot).
2. It calls `robot.setup()` to get the robot's physical configuration (dimensions, colors, gear ratio, sensor spacing).
3. It spawns the Bevy app with physics entities matching the configuration.
4. It calls `robot.run()`, which runs the robot's main loop.
5. Inside `run()`, the robot code calls imported device functions (read sensors, set motors, sleep). Each call crosses the WASM boundary into the host.
6. The host (`BotHost` / `wasm_host.rs`) services these calls by stepping the physics simulation forward as needed and returning sensor values.
7. After execution completes, recorded `ExecutionData` is used for playback visualization.

**Three operational modes**:
- **Run**: Load a single WASM file, simulate, then visualize the recorded replay.
- **Test**: Launch with a bot configuration (no WASM execution) for manual testing of physics/tracks.
- **Serve**: Start an HTTP server; accept WASM uploads via POST, simulate each, and add results to the live visualizer.

## Components

### WIT Interface (`wit/world.wit`)
- **Purpose**: Defines the contract between robot code and simulator host.
- **Key types**: `device-value` (8-byte record), `device-operation` (variant of all sensor/actuator ops), `future-handle` (async handle), `configuration` (robot physical parameters), `color`.
- **Imports (host provides)**: `devices` interface (sensor reads, motor writes, time, sleep, async polling), `diagnostics` interface (logging, file/CSV output).
- **Exports (robot provides)**: `robot` interface with `setup() -> configuration` and `run()`.

### Bot Crate (`bot/`)
- **Purpose**: Robot logic compiled to a WASM component.
- **Public API**: Two parallel APIs for robot authors:
  - `blocking_api`: `get_line_sensors()`, `set_motors_pwm()`, `sleep_for()`, `get_time_us()`, `console_log()`, etc.
  - `async_api`: Same operations as async futures, enabling concurrent sensor reads via `futures_micro::zip!`.
- **Async Framework**: A minimal single-threaded executor using a no-op waker. `FutureValue` wraps the WIT `FutureHandle` and implements `Future<Output = DeviceValue>`. `ValueWatcher<T>` provides a single-value channel with `next()` future and `stream()` for async inter-task communication.
- **Examples**: Progressive complexity from bang-bang control (`toy`) through PID control (`basic_pid`, `pid_with_memory`) to async parallel architectures (`parallel_tasks`, `tasks_with_channels`).
- **External dependencies**: `wit-bindgen`, `futures-micro`, `pin-project-lite`.

### Execution Data Crate (`sim/execution-data/`)
- **Purpose**: Shared data types between executor and simulator.
- **Key types**:
  - `SimulationStepper` trait -- abstraction for physics stepping, sensor access, motor control.
  - `ExecutionData` -- complete recording: `BodyExecutionData` (transforms per step), `WheelExecutionData` (angles per step), `ActivityData` (start/end/out times).
  - `SensorsData` -- aggregated snapshot: line sensors (16 f32), motor angles, gyro, IMU, bot position.
  - `MotorDriversDutyCycles`, `MotorAngles`, `GyroData`, `ImuFusedData`, `BotPosition`, `BotFinalStatus`.
- **Dependencies**: `bevy` (for `Vec3`, `Transform`, `Component`, `Resource`), `anyhow`.

### Executor Crate (`sim/executor/`)
- **Purpose**: Loads and runs WASM robot components using Wasmtime.
- **Key design**: `BotHost` holds a `Box<dyn SimulationStepper>` and manages async operation handles (a `BTreeMap` of pending futures with readiness times). It services device calls by advancing the physics simulation to the required time. Uses Wasmtime fuel metering to enforce execution time limits.
- **Dependencies**: `wasmtime`, `execution-data`, `bevy`.

### Simulator Binary Crate (`sim/sim/`)
- **Purpose**: Main application -- physics simulation, 3D visualization, and GUI.
- **Submodules**:
  - `bot/` -- Bevy plugin for bot entities (body, wheels, sensors)
  - `track.rs` -- Track geometry with segment types (Straight, NinetyDegTurn, CyrcleTurn, Start, End)
  - `runner.rs` -- `RunnerStepper` bridges Bevy ECS physics to the `SimulationStepper` trait
  - `server.rs` -- HTTP server for accepting WASM uploads
  - `visualizer.rs` -- Playback of recorded execution data
  - `ui.rs`, `ui_runner.rs`, `ui_test.rs` -- egui-based GUI

### Bot Physics Model (`sim/sim/src/bot/`)
- **Purpose**: Spawns and simulates the robot as Bevy/Rapier entities.
- `model.rs` -- Creates rigid body, colliders (compound), revolute joints for wheels, child sensor entities.
- `motors.rs` -- DC motor torque-speed curve model. Converts PWM duty cycle to wheel torque via `pwm_to_torque()`.
- `sensors/line_sensors.rs` -- Ray-casts from each sensor position downward, computes reflection based on distance to track centerline with height-based attenuation and Gaussian noise.

### Track System (`sim/sim/src/track.rs`, `track_selection.rs`)
- **Purpose**: Defines track geometry as a sequence of segments.
- **Segment types**: `Start`, `End`, `Straight(length)`, `NinetyDegTurn(line_half_length, side)`, `CyrcleTurn(radius, angle, side)`.
- Each segment generates both a physics collider and a visualization mesh. Segments are chained via `compute_next_origin()` which computes the transform for the next segment.
- **Predefined tracks**: Line (straight), Angle (90-degree turns), Turn (smooth curves), Simple (mixed), Race (complex course).

### Visualization & UI
- **Purpose**: 3D replay of simulation results with camera controls and bot management.
- `visualizer.rs` -- Spawns bot visualization, syncs body/wheel transforms to playback time.
- `ui.rs` -- Camera setup (PanOrbitCamera), keyboard controls, help dialog.
- `ui_runner.rs` -- VCR playback controls, bot loading, ranking display.
- **Features**: Play/pause/rewind/fast-forward, step-by-step, numpad camera presets, file dialog for loading WASM bots, error modal, help modal with markdown rendering, bot ranking sorted by finish time.

### Bevy Panorbit Camera (`sim/bevy_panorbit_camera/`)
- **Purpose**: Vendored/forked orbit camera plugin with egui integration.
- **Notable**: Customized axis mapping (`[Vec3::X, -Vec3::Z, -Vec3::Y]`) to match the simulator's coordinate system (Z-up).

## Patterns & Conventions

- **ECS Architecture**: The simulator is built entirely on Bevy's Entity-Component-System pattern. Systems run in well-defined schedules (`Startup`, `FixedMain`, `BotUpdate`, `CustomTransformPropagation`).
- **Plugin Composition**: Each subsystem is a Bevy `Plugin` (`BotPlugin`, `TrackPlugin`, `MotorsModelPlugin`, `SensorsModelPlugin`, `StoreExecDataPlugin`, `GuiSetupPlugin`, `CameraSetupPlugin`).
- **Mode Separation**: The `EntityFeatures` enum (`Physics`, `Visualization`, `PhysicsAndVisualization`) cleanly separates what gets spawned in each mode. Headless simulation uses `MinimalPlugins`; visualization adds `DefaultPlugins`.
- **WASM Sandboxing**: Robot code is fully sandboxed via Wasmtime. Fuel metering enforces time limits. The `define_unknown_imports_as_traps` call prevents undefined behavior from unexpected imports.
- **Async-over-Sync Pattern**: The bot's async framework is a cooperative single-threaded executor that polls futures in a loop. The `poll_loop(start/end)` calls signal the host to advance simulation time between poll rounds, enabling time-aware async device reads.
- **Deterministic Simulation**: Seeded RNG (`SmallRng::seed_from_u64(42)`) ensures reproducible sensor noise. Fixed timestep physics via Bevy's `Time<Fixed>`.
- **Data Recording**: Every physics step, `store_data()` appends the body transform and wheel angles to `ExecutionData`. This enables frame-accurate playback after simulation completes.
- **No Tests**: No test files or test modules were found in the source code.
- **Minimal Error Handling**: Most errors use `unwrap()` on query results. The WASM executor uses `wasmtime::Result` / `anyhow` for error propagation.

## Configuration & Usage

**Building the bot** (requires WASM component toolchain):
```bash
cd bot/
cargo component build --release
# produces target/wasm32-wasip2/release/line_follower_robot.wasm
```

**Running the simulator**:
```bash
cd sim/
# Run a single bot:
cargo run --release -p sim -- run -i /path/to/robot.wasm -o output_dir

# Run headless (no GUI):
cargo run --release -p sim -- run -i /path/to/robot.wasm --cli

# Test mode (manual control, no WASM):
cargo run --release -p sim -- test

# Server mode (accept WASM via HTTP POST):
cargo run --release -p sim -- serve -p 9999
```

**CLI options** (from `main.rs`):
- `-p/--period`: Simulation step period in microseconds (100-1000, default 500)
- `-t/--track`: Track selection (line, angle, turn, simple, race; default simple)
- `--time-limit`: Simulation time limit in seconds (default 60)
- `-s/--start-time`: Racing start delay in microseconds (default 1,000,000)

**Robot configuration** is defined in `setup()` returning a `Configuration` struct with physical parameters: axle width, body dimensions, wheel diameter, gear ratio, sensor spacing, sensor height, and colors.

## Notable Details

- **WIT-based contract**: The project uses the WebAssembly Component Model (not just raw WASM) with a formal interface definition. This enables language-agnostic robot code in principle -- any language that can produce WASM components with WIT support could be used to write robot logic.
- **DC Motor Model**: The motor simulation (`motors.rs`) approximates a brushed DC motor with a linear torque-speed curve. The constants (NO_LOAD_RPM: 40000, STALL_TORQUE: 0.001 N.m) are tuned for small toy motors.
- **Line Sensor Model**: Sensors simulate optical reflection using ray-casting. The reflection model accounts for sensor height (Z-distance attenuation), line width (20mm), and uses smoothstep interpolation at line edges. Gaussian noise (sigma=1.0) is added for realism.
- **Gravity axis**: The physics uses Z-up convention (`gravity = Vec3::NEG_Z * 9.81`), with the track on the XY plane.
- **Units**: Physical dimensions in the configuration are specified in millimeters but converted to meters internally (division by 1000).
- **Presentation materials**: The `presentation/` directory contains a slide deck (Markdown + images), suggesting this project is used for teaching or workshop purposes. The `ITEMS.md` file outlines a progressive curriculum from basic line following through PID tuning to async architectures.
- **Edition 2024**: Both the bot and sim workspaces use Rust edition 2024.
- **Deleted experiment**: The git status shows a deleted `bevy-experiment/` directory, which appears to have been an earlier prototype.
- **Active development**: The `massi.rs` file is untracked (user's personal robot code), and `parallel_tasks.rs` has been modified, indicating active iteration on the async bot examples.
