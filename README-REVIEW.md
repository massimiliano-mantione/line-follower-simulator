# README Review Results

## Accuracy Issues

- **[FIXED] License section was wrong.** The README stated "No license file is present in this repository." A `LICENSE` file exists — MIT License, copyright 2025 Massimiliano Mantione. Corrected to reference the license file.

- **[FIXED] `read_gyro()` return order was wrong.** The README documented `(roll, pitch, yaw)`. The actual implementation in `blocking_api.rs` unpacks `(pitch, roll, yaw)`. Same error in `get_imu_fused_data()`. Both corrected.

- **[FIXED] Track segment type `NinetyDegTurn` parameter was inaccurate.** Documented as `NinetyDegTurn(radius, side)` but the actual struct field is `line_half_length`, not `radius`. Corrected.

- **[FIXED] Project structure tree was missing files.** `app_builder.rs`, `data.rs`, `ui_test.rs`, and `utils.rs` were omitted from `sim/sim/src/`. Added to the tree.

- **[FIXED] Bot crate dependency list was incomplete.** `pin-project-lite` was missing. Added.

- **[NOT FIXED] `massi.rs` is not listed in the project structure.** The file exists and is declared as a public module in `lib.rs`, referenced in a commented-out call. The Development section mentions it but the tree omits it.

- **[NOT FIXED] `get_line_sensors()` in blocking API uses `device_operation_immediate`, not blocking.** The blocking API table implies all functions are blocking wrappers, but `get_line_sensors()` calls `device_operation_immediate` for both banks (returns cached values without advancing simulation time). This distinction is not surfaced.

---

## Missing Content

- **[FIXED] Missing API functions.** `get_steps_and_period_us()`, `remote_enabled()`, `write_plain_file()`, and `wait_remote_disabled()` were absent from the API table. Added.

- **[FIXED] `telemetry_test` example was missing from the examples table.** Added with description.

- **[NOT FIXED] No mention of `sim/help.md`.** If it contains end-user documentation, it should be referenced from the README.

- **[NOT FIXED] Pre-compiled `.wasm` files in `sim/` are not mentioned.** Files like `line_follower_robot-liner.wasm` and `line_follower_robot-noname.wasm` exist. A new developer would benefit from knowing these can be used to test the simulator without building the bot first.

- **[NOT FIXED] No mention of `--cli` flag behavior in serve mode.** The `Serve` subcommand always opens a Bevy window (no `--cli` support). This could surprise someone trying headless serve mode.

- **[NOT FIXED] No explanation of the `or!` combinator.** The examples table mentions it for `nb/parallel_tasks` but doesn't explain it comes from `futures-micro`.

- **[NOT FIXED] `bevy_picking` feature not mentioned.** `sim/Cargo.toml` enables it alongside `dynamic_linking`.

---

## Quality Suggestions

- **The `sim/` vs `sim/sim/` distinction could confuse readers.** The architecture diagram labels `sim/sim/` as the Bevy/physics code, but a reader unfamiliar with Rust workspaces may not understand that `sim/` is the workspace root and `sim/sim/` is the binary crate inside it. A clarifying sentence would help.

- **Track segment descriptions mix API names with internal struct field names.** Describing what each segment type looks like geometrically (e.g., "a sharp 90-degree corner" vs. "a smooth arc of arbitrary angle") would be more useful than showing constructor-like signatures.

- **Getting Started section doesn't make the two-step build explicit.** A new developer might expect a single build command. Making the separate bot-then-simulator flow more prominent would reduce confusion.

- **The active example isn't identified.** The README says "edit `bot/src/lib.rs` and change the call inside `fn run()`" but doesn't mention that the currently active example is `examples::nb::parallel_tasks::run(4.0)`.

- **Host-side dependency versions are not listed.** From `sim/Cargo.toml`: Bevy 0.16.1, Wasmtime 36.0.2, bevy_egui 0.36.0, tiny_http 0.12.0. These are useful for evaluating compatibility.

---

## Overall Assessment

The README is well-structured and covers the architecture and workflow clearly. The critical issues (wrong license claim, incorrect API return orderings, missing API functions, incomplete project tree) have been fixed. The remaining items are minor completeness and polish improvements.
