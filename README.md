# line-follower-simulator

A simulator for differential-drive line-follower robots written in Rust. Robot logic is compiled to a WebAssembly component and runs inside a sandboxed Wasmtime runtime. The simulator uses the Bevy game engine with Rapier 3D physics to model the robot's mechanics, and records execution data for frame-accurate 3D replay visualization.

The project is designed for educational and competitive contexts: multiple algorithm implementations are included as examples, the WIT interface contract means robot code can be written in any language that targets the WebAssembly Component Model, and a server mode allows contestants to upload `.wasm` files over HTTP for evaluation.

---

## Architecture

The system follows a host-guest architecture with a WIT (WebAssembly Interface Types) boundary separating robot logic from simulation infrastructure.

```
┌─────────────────────────────────────────────────────────┐
│  bot/ (WASM component – guest)                          │
│                                                         │
│  setup() ──► returns Configuration                      │
│  run()   ──► main control loop                          │
│               │                                         │
│         calls │ device imports (sensors, motors, time)  │
└───────────────┼─────────────────────────────────────────┘
                │  WIT boundary  (wit/world.wit)
┌───────────────┼─────────────────────────────────────────┐
│  sim/ (host)  │                                         │
│               ▼                                         │
│  executor/  ◄─── Wasmtime: loads .wasm, services calls  │
│      │                                                  │
│      │  SimulationStepper trait                         │
│      ▼                                                  │
│  sim/sim/   ◄─── Bevy ECS + Rapier 3D physics           │
│      │            BotPlugin, MotorsModelPlugin,         │
│      │            SensorsModelPlugin, TrackPlugin        │
│      │                                                  │
│      ├── run     – simulate then replay in 3D viewer    │
│      ├── test    – manual physics inspection            │
│      └── serve   – HTTP server, accepts .wasm uploads   │
└─────────────────────────────────────────────────────────┘
```

**Execution flow:**

1. Simulator loads a `.wasm` component from disk (or HTTP POST body).
2. Calls `robot.setup()` to obtain physical configuration (dimensions, colors, gear ratio, sensor geometry).
3. Spawns Bevy app with Rapier rigid bodies, revolute-joint wheels, and ray-cast line sensors.
4. Calls `robot.run()` — the robot's main loop runs on the WASM guest.
5. Each device call (read sensors, set motors, sleep) crosses the WASM boundary.
6. The host services the call by stepping physics forward and returning the current sensor state.
7. Every step appends body and wheel transforms to `ExecutionData` for frame-accurate replay.

---

## Project Structure

```
line-follower-simulator/
├── wit/
│   └── world.wit               # Host-guest contract: device ops, diagnostics, robot interface
│
├── bot/                        # Robot crate – compiled to a WASM component
│   ├── Cargo.toml              # cdylib; depends on wit-bindgen, futures-micro, pin-project-lite
│   └── src/
│       ├── lib.rs              # WIT Guest impl: setup() + run() entry points
│       ├── wasm_bindings.rs    # Auto-generated WIT bindings
│       ├── wasm_bindings_ext.rs# DeviceValue accessor helpers (get_u8, get_i16, get_u32, …)
│       ├── blocking_api.rs     # Synchronous robot API
│       ├── async_api.rs        # Async robot API (device ops as Futures)
│       ├── async_framework.rs  # Single-threaded executor, FutureValue, ValueWatcher
│       └── examples/
│           ├── toy.rs          # Bang-bang control (blocking)
│           ├── basic_pid.rs    # PID line follower (blocking)
│           ├── pid_with_memory.rs      # PID with out-of-line recovery state (blocking)
│           ├── telemetry_test.rs       # CSV telemetry demonstration
│           └── nb/             # Async variants
│               ├── toy.rs
│               ├── parallel_tasks.rs   # Async PID, both sensor banks read concurrently
│               ├── tasks_with_channels.rs  # Async PID using ValueWatcher channels
│               └── pid_with_memory.rs
│
└── sim/                        # Simulator workspace (4 crates)
    ├── Cargo.toml              # Workspace root
    ├── execution-data/         # Shared data types: ExecutionData, SimulationStepper trait
    ├── executor/               # Wasmtime host: loads WASM component, services device calls
    ├── bevy_panorbit_camera/   # Vendored orbit camera Bevy plugin
    └── sim/                    # Main simulator binary
        └── src/
            ├── main.rs         # CLI: Run / Test / Serve subcommands
            ├── bot/            # Bevy plugin: BotPlugin, MotorsModelPlugin, SensorsModelPlugin
            ├── track.rs        # Track geometry and segment types
            ├── track_selection.rs  # Predefined track definitions
            ├── app_builder.rs  # Bevy App construction (wires all plugins together)
            ├── runner.rs       # Bridges Bevy physics to SimulationStepper
            ├── server.rs       # HTTP server (tiny_http) for WASM uploads
            ├── data.rs         # StoreExecDataPlugin: records execution data per step
            ├── visualizer.rs   # Playback of recorded ExecutionData
            ├── ui.rs / ui_runner.rs / ui_test.rs  # egui-based GUI and playback controls
            └── utils.rs        # Side enum, EntityFeatures, geometry helpers
```

---

## Components

### WIT Interface (`wit/world.wit`)

Defines the complete contract between robot code and the simulator host. The world exports three interfaces:

- **`devices` (imported by robot):** device operations the robot can invoke — reading 16 line sensors (two banks of 8), motor angle encoders, gyroscope (roll/pitch/yaw rate in deg/s), IMU fused angles, elapsed time, simulation period, sleep/wait primitives, and motor PWM control (`-1000` to `1000`). Operations can be invoked in three modes: immediate (returns cached value), blocking (advances simulation time), or async (returns a `future-handle` for later polling).

- **`diagnostics` (imported by robot):** `write_line` for serial-style logging (100 µs/character cost) and `write_file` for binary or CSV data output (10 µs/byte cost).

- **`robot` (exported by robot):** `setup() -> Configuration` and `run()`. `Configuration` specifies robot name, main and secondary colors, axle width (100–200 mm), front/back body lengths, wheel diameter (20–40 mm), gear ratio, and line sensor spacing and height.

### Bot Crate (`bot/`)

Robot logic compiled to a `wasm32-wasip2` WASM component via `cargo-component`.

**Blocking API (`blocking_api.rs`):** Synchronous wrappers around WIT device imports. Key functions:

| Function | Description |
|---|---|
| `get_line_sensors() -> [u8; 16]` | Read all 16 line sensors (0 = white, ~255 = black after calibration inversion) |
| `get_motor_angles() -> (u16, u16)` | Left and right encoder angles |
| `read_gyro() -> (i16, i16, i16)` | Pitch, roll, yaw angular velocity |
| `get_imu_fused_data() -> (i16, i16, i16)` | Absolute pitch, roll, yaw angles |
| `get_time_us() -> u32` | Elapsed simulation time in microseconds |
| `sleep_for(us)` / `sleep_until(us)` | Advance simulation time |
| `set_motors_pwm(left, right)` | Set motor duty cycles (-1000 to 1000) |
| `wait_remote_enabled()` | Block until start signal |
| `console_log(text)` | Serial-style diagnostic output |
| `write_csv_file(name, data, spec)` | Structured telemetry output |
| `write_plain_file(name, data)` | Binary file output |
| `get_steps_and_period_us() -> (u32, u32)` | Steps count and simulation period |
| `remote_enabled() -> bool` | Non-blocking check of start signal |
| `wait_remote_disabled()` | Block until stop signal |

**Async API (`async_api.rs`):** The same operations as async functions. `get_line_sensors()` issues both sensor bank reads concurrently using `futures_micro::zip!`.

**Async framework (`async_framework.rs`):** A minimal cooperative single-threaded executor (`run(future)`) built on Rust's standard `Future`/`Poll` traits with a no-op waker. Key types:

- `FutureValue`: wraps a `FutureHandle` from WIT, polls it via `device_poll`. Calls `forget_handle` on drop.
- `ValueWatcher<T>`: a single-value channel that retains only the most recent update. Supports `.next()` (one-shot future), `.stream()` (reusable), `.map()`, and `.filter()` combinators.

### Simulator (`sim/`)

**`execution-data/`:** Shared data structures used across crates. Defines `SimulationStepper` (the trait bridging the executor to physics), `ExecutionData` (per-step body and wheel transforms), `SensorsData`, `MotorDriversDutyCycles`, `BotPosition`, and `BotFinalStatus` (with ranking logic for multi-bot comparison).

**`executor/`:** Wasmtime host implementation. Loads a WASM component, instantiates the WIT world, and implements each device import by delegating to a `SimulationStepper`. Also exposes `wasm_bindings` and a `mock_stepper` for testing.

**`sim/sim/`:** The main binary. Contains:

- **Physics model (`bot/`):** Bevy ECS plugins. `BotPlugin` spawns body and wheel entities. `MotorsModelPlugin` models DC motor torque-speed curves and converts PWM duty cycles to Rapier forces via revolute joints. `SensorsModelPlugin` implements ray-cast line sensors with height attenuation, line-edge smoothstep, and Gaussian noise. The `EntityFeatures` enum distinguishes `Physics`, `Visualization`, and `PhysicsAndVisualization` modes to share entity setups across headless and visual runs.

- **Track system (`track.rs`, `track_selection.rs`):** Tracks are chains of `TrackSegment` values with associated `SegmentTransform`. Segment types: `Start`, `End`, `Straight(length)`, `NinetyDegTurn(line_half_length, side)`, `CyrcleTurn(radius, angle, side)`. Five predefined tracks: `line`, `angle`, `turn`, `simple` (default), `race`.

- **Runner (`runner.rs`):** Connects Bevy physics to `SimulationStepper`. Drives the simulation loop and collects `ExecutionData`.

- **Visualizer (`visualizer.rs`):** Replays recorded `ExecutionData` using Bevy transforms, providing VCR-style playback with camera presets and bot ranking display.

- **Server (`server.rs`):** HTTP server via `tiny_http`. A `GET` returns a ready message; a `POST` with a raw `.wasm` body runs the robot and sends results back via an `mpsc` channel.

- **GUI (`ui.rs`, `ui_runner.rs`):** `bevy_egui`-based interface for playback controls, file dialogs, and run management.

### Predefined Example Algorithms

| Example | Style | Description |
|---|---|---|
| `toy` | blocking | Bang-bang: checks outermost sensors, reads center-of-mass error |
| `basic_pid` | blocking | Weighted-centroid PID on all 16 sensors |
| `pid_with_memory` | blocking | PID with last-known-direction recovery when line is lost |
| `nb/toy` | async | Async rewrite of `toy` |
| `nb/parallel_tasks` | async | PID with concurrent left/right sensor reads; `or!` combinator for termination |
| `nb/tasks_with_channels` | async | PID using `ValueWatcher` to decouple sensor reading from control |
| `nb/pid_with_memory` | async | Async PID with out-of-line recovery |
| `telemetry_test` | blocking | Demonstrates `write_csv_file` for structured telemetry output |

---

## Getting Started

### Prerequisites

- **Rust toolchain** (edition 2024): install via [rustup](https://rustup.rs/)
- **`cargo-component`**: required to build the bot WASM component
  ```bash
  cargo install cargo-component
  ```
- **`wasm32-wasip2` target:**
  ```bash
  rustup target add wasm32-wasip2
  ```
- A graphics driver supporting Bevy's renderer (required for the default visualizer; not needed with `--cli`)

### Building the Bot

```bash
cd bot/
cargo component build --release
```

This produces:

```
bot/target/wasm32-wasip2/release/line_follower_robot.wasm
```

To switch which algorithm runs, edit `bot/src/lib.rs` and change the call inside `fn run()`. The commented-out lines show all available examples.

### Building the Simulator

```bash
cd sim/
cargo build --release -p sim
```

---

## Running

All simulator commands are run from the `sim/` directory. Global options (`--period`, `--track`) must appear before the subcommand.

### Run mode — simulate and replay

```bash
cargo run --release -p sim -- [global options] run -i <robot.wasm> [run options]
```

**Global options:**

| Flag | Default | Description |
|---|---|---|
| `-p` / `--period` | `500` | Simulation step period in microseconds (100–1000) |
| `-t` / `--track` | `simple` | Track selection: `line`, `angle`, `turn`, `simple`, `race` |

**`run` subcommand options:**

| Flag | Default | Description |
|---|---|---|
| `-i` / `--input` | required | Path to robot `.wasm` file |
| `-o` / `--output` | `.` | Directory for output data |
| `-l` / `--logs` | false | Save robot diagnostic logs |
| `--time-limit` / `--tl` | `60` | Simulation time limit in seconds |
| `-s` / `--start-time` | `1000000` | Racing start delay in microseconds |
| `-c` / `--cli` | false | Headless mode — skip the 3D visualizer |

**Examples:**

```bash
# Simulate with default settings, open 3D replay:
cargo run --release -p sim -- run -i ../bot/target/wasm32-wasip2/release/line_follower_robot.wasm

# Headless run on the race track with a 30-second limit:
cargo run --release -p sim -- --track race run \
  -i ../bot/target/wasm32-wasip2/release/line_follower_robot.wasm \
  --time-limit 30 --cli

# Save output and logs:
cargo run --release -p sim -- run \
  -i ../bot/target/wasm32-wasip2/release/line_follower_robot.wasm \
  -o /tmp/run_output --logs
```

### Test mode — inspect physics manually

```bash
cargo run --release -p sim -- test [-i <robot.wasm>]
```

Loads the robot configuration from the WASM file if provided; otherwise uses a built-in default. Opens the 3D view with physics active for manual inspection.

### Serve mode — accept WASM over HTTP

```bash
cargo run --release -p sim -- serve [-p <port>] [-a <address>]
```

| Flag | Default | Description |
|---|---|---|
| `-p` / `--port` | `9999` | HTTP server port |
| `-a` / `--address` | `0.0.0.0` | Bind address |
| `--time-limit` / `--tl` | `60` | Simulation time limit per submission |
| `-s` / `--start-time` | `1000000` | Racing start delay in microseconds |

Submit a robot:
```bash
curl -X POST http://localhost:9999 --data-binary @robot.wasm
```

A `GET` request to any path returns a plain-text ready message.

---

## Configuration

Robot physical parameters are set in `bot/src/lib.rs` inside the `setup()` function. All dimension values are in millimeters and are validated by the simulator against the ranges defined in `wit/world.wit`:

| Field | Range | Description |
|---|---|---|
| `width_axle` | 100–200 mm | Wheel-to-wheel axle width |
| `length_front` | 100–300 mm | Body length ahead of axle |
| `length_back` | 10–50 mm | Body length behind axle |
| `clearing_back` | 1 – wheel radius mm | Ground clearance at back |
| `wheel_diameter` | 20–40 mm | Wheel outer diameter |
| `gear_ratio_num` / `gear_ratio_den` | 1–100 each | Motor-to-wheel gear ratio |
| `front_sensors_spacing` | 1–15 mm | Spacing between adjacent line sensors |
| `front_sensors_height` | 1 – wheel radius mm | Sensor mount height above ground |

Motor PWM values passed to `set_motors_pwm` range from `-1000` to `1000`. Internally the simulator converts PWM to force using a DC motor torque-speed curve model.

Simulation coordinates use Z-up convention. Dimensions are converted from mm to meters internally.

---

## Development

### Selecting an example algorithm

Edit `bot/src/lib.rs`. The `run()` function contains commented-out calls for each example. Uncomment the desired one and rebuild with `cargo component build --release`.

### Writing a new robot algorithm

1. Create a new file under `bot/src/examples/` (or add a module alongside `massi.rs`).
2. Implement a `run()` function (sync) or `async fn run()` (async).
3. For blocking style, import from `crate::blocking_api`. For async style, import from `crate::async_api`.
4. Call your function from `bot/src/lib.rs`. For async functions, wrap in `async_framework::run(...)`.

**Minimal blocking example:**
```rust
use crate::blocking_api::*;

pub fn run() {
    wait_remote_enabled();
    loop {
        let sensors = get_line_sensors();
        // sensors[0..8]  = left bank, sensors[8..16] = right bank
        // 0 = white surface, 255 = black line (after inversion)
        set_motors_pwm(300, 300);
    }
}
```

**Minimal async example:**
```rust
use crate::async_api::*;

pub async fn run() {
    wait_remote_enabled().await;
    loop {
        let sensors = get_line_sensors().await;
        set_motors_pwm(300, 300);
    }
}
// In lib.rs: async_framework::run(examples::my_example::run());
```

### Telemetry and diagnostics

Use `console_log(text)` for serial-style logging. Use `write_csv_file(name, data, spec)` with the helpers in `blocking_api::csv` to produce structured CSV output saved to the output directory. Logging costs simulated time (100 µs per character; 10 µs per byte for file writes).

### Codebase notes

- No automated test suite is present in the repository.
- The `bevy-experiment/` directory listed in `.gitignore`-tracked deletions was a prior Bevy-only prototype; it is no longer part of the workspace.
- `sim/sample-robot-code/` is excluded from the workspace build via `Cargo.toml` `exclude`.
- The simulator uses `bevy` with the `dynamic_linking` feature enabled for faster iteration builds.

---

## License

MIT License — see [LICENSE](LICENSE) for details.
