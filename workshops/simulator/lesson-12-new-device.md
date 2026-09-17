# Lesson 12 — Add a device end to end

> **Capstone.** The only lesson that touches all five layers, and the one that
> finally justifies calling `wit/world.wit` a contract.

> **Goal.** Re-add the gyroscope and the fused IMU, starting from the component
> contract and working down to the Bevy system that computes them.

Needs the WASM toolchain and the `wit-bindgen` CLI.

## The setup

Trace the path a single sensor value takes, *upwards*:

```
Rapier Velocity.angular                     physics truth
  -> compute_imu_data                       Bevy system, BotUpdate schedule
  -> SensorsData.gyro                       Bevy resource
  -> RunnerStepper::get_gyro                the SimulationStepper seam
  -> BotHost::step latches into SteppedData every 2 ticks
  -> FutureOperation::compute_value         packed into 8 bytes as 3x i16
  -> DeviceValue                            across the component boundary
  -> blocking_api::read_gyro                unpacked into (i16, i16, i16)
  -> the robot's PID
```

Eight layers for one number. Is that over-engineering? A fair question. The
defensible answer is that each boundary buys something specific: the seam buys
testability without Bevy, the latch buys honest device timing, the byte packing buys
a realistic register interface, the component boundary buys the sandbox. Remove any
one and something in this workshop stops working.

## The actual skill being taught

**Work top down, and let the compiler drive.** Edit the contract first, regenerate,
then fix errors until it builds. The type system will walk you through all eight
layers.

A wall of errors is the *plan*, not a setback.

## Your task

| | Where | What |
|---|---|---|
| 12.1 | `wit/world.wit` | the two `device-operation` variants — **write the doc comments too**, the timing is part of the interface |
| 12.2 | `bot/` | regenerate the guest bindings |
| 12.3 | `execution-data/src/lib.rs` | `GyroData`, `ImuFusedData`, their `From<Vec3>`, and the `SensorsData` fields |
| 12.4 | `executor/src/wasm_host.rs` | `FutureOperation` variants, `compute_value`, the byte packing |
| 12.5 | `executor/src/wasm_host.rs` | the sample rates, `SteppedData`, and the latch in `step()` |
| 12.6 | `execution-data`, `mock_stepper.rs`, `runner.rs` | the seam: two trait methods, two implementors |
| 12.7 | `bot/src/{blocking,async}_api.rs` | the robot-facing API |
| 12.8 | `sim/src/bot/sensors/imu.rs` | the Bevy system, plus its place in the chain |

Step 12.2 is the one nobody expects to be manual:

```bash
cd bot
wit-bindgen rust ../wit/world.wit --out-dir .
mv ./line_follower_robot.rs src/wasm_bindings.rs
```

(The recipe is at the top of `bot/build.sh`, commented out.) The generated file is
checked in on purpose: so you can build a robot without the CLI, and so a change to
the contract is **visible in review**. A contract change should be a reviewable event.

## The detail worth noticing

Line sensors and motor angles are read *live* off the stepper. The gyro and the IMU
are read from a **latch** (`SteppedData`). Why the asymmetry?

Because a gyro that integrates over 2 ms genuinely cannot give you a fresh value on
demand, and pretending otherwise would let a robot poll its way to a higher effective
sample rate than the hardware has. The latch is not an optimisation — it is the
model.

## How to verify

```bash
cd sim && cargo test && cargo run --release -p sim -- test
cd ../bot && ./build.sh
```

Unlike the other lessons, **this branch starts with all 38 tests green** — there is
no red-to-green signal, because the assertions for the removed devices were
themselves removed. So write your own: `wasm_host_tests.rs` has a marked spot where
the `ready_steps` assertions used to be. Put them back as you go.

Then actually *use* the sensor. Add a gyro read to a robot and log it while driving
in `test` mode. The yaw rate should be near zero on the straight and clearly non-zero
in a corner, with the sign matching the turn direction.

## Why this matters for a real robot

The best `KD` tuning does not use the apparent line rotation speed alone — it uses
**actual rotation speed** (from the gyro) minus **apparent rotation speed** (from
dE/dt). With the gyro restored, that becomes implementable. The capstone hands you
the sensor the advanced control strategy needs.

## Hints

<details>
<summary>Hint 1 — the component fails to instantiate after I edit the WIT</summary>

You skipped 12.2, or ran it and did not move the output over
`bot/src/wasm_bindings.rs`. Guest and host must be generated from the same contract:
the variant encoding is positional.
</details>

<details>
<summary>Hint 2 — which axis is which?</summary>

This world is Z-up: X across the axle, Y forward, Z height. So roll is about y, pitch
about x, yaw about z. Look at the `From<Vec3>` conversions you are writing — that
mapping is the only place it is recorded.
</details>

<details>
<summary>Hint 3 — the gyro always reads zero</summary>

Either the latch in `step()` is not refreshing the field, or `compute_value` is
reading from the stepper instead of from `stepped_data`.
</details>

<details>
<summary>Hint 4 — the values are one tick stale in the recording</summary>

`store_data` in `data.rs` is ordered `.after(compute_imu_data)`. Check where you put
your system in `SensorsModelPlugin`'s chain.
</details>

## Solution

```bash
git diff ws/simulator/12-new-device..main
```

## And then?

This is the honest answer to "how would I add a distance sensor, a camera, or a
second line array?" — which is the question you will actually have if you go and
build a simulator of your own.
