# Speaker notes — Lesson 12: Add a device end to end

**Branch** `ws/simulator/12-new-device` · **Capstone** · ~50 min

The only lesson that touches all five layers, and therefore the best one for anyone
who wants to *own* the architecture rather than visit it. Also the lesson that
finally justifies calling the WIT file a contract.

**Goal.** Re-add the gyroscope and the fused IMU to the simulator, starting from the
component contract and working down to the Bevy system that computes them.

---

## Removed on this branch

`read-gyro` and `read-imu-fused-data`, from every layer:

| Layer | What is gone |
|---|---|
| `wit/world.wit` | the two `device-operation` variants and their docs |
| `bot/src/wasm_bindings.rs` | regenerated without them |
| `bot/src/{blocking,async}_api.rs` | `read_gyro`, `get_imu_fused_data` |
| `executor/src/wasm_host.rs` | the `FutureOperation` variants, their `compute_value` arms, `DeviceOperationExt` arms, `READY_STEPS_*`, the `SteppedData` fields and their sampling in `step()` |
| `execution-data/src/lib.rs` | `GyroData`, `ImuFusedData`, the `SimulationStepper` methods, the `SensorsData` fields |
| `sim/src/bot/sensors/imu.rs` | `compute_imu_data` — the whole file |
| `mock_stepper.rs`, `runner.rs` | the trait implementations |

Nothing in `bot/src/examples/` calls either operation, which is precisely what makes
this surgery possible without breaking every robot. Say that out loud — it is why
this lesson can exist at all, and it is a decent argument for keeping example code
free of incidental dependencies.

## Framing (8 min)

This is the lesson where the architecture diagram earns its keep. Put it back on the
board and trace the path a single sensor value takes, *upwards*:

```
Rapier Velocity.angular                      (physics truth)
  -> compute_imu_data                        Bevy system, BotUpdate schedule
  -> SensorsData.gyro                         Bevy resource
  -> RunnerStepper::get_gyro                  the SimulationStepper seam
  -> BotHost::step samples into SteppedData   latched every 2 ticks
  -> FutureOperation::compute_value           packed into 8 bytes as 3x i16
  -> DeviceValue across the component boundary
  -> blocking_api::read_gyro                  unpacked into (i16, i16, i16)
  -> the robot's PID
```

Eight layers for one number. Ask whether that is over-engineering — it is a real
question and worth two minutes. The defensible answer: each boundary buys something
specific. The seam buys testability without Bevy. The latch buys honest device
timing. The byte packing buys a realistic register interface. The component boundary
buys the sandbox. Remove any one and something in the workshop stops working.

Then the order of work, which is the actual skill being taught: **top down, and let
the compiler drive.** Edit the WIT first, regenerate, then fix errors until it
builds. The type system walks you through all eight layers. That is the experience to
engineer for — say up front that a wall of errors is the *plan*, not a setback.

## The steps

**1. The contract.** Add back the two variants, with the doc comments stating the
rates ("ready every 2 periods", "ready every 10 periods"). Make them write the docs;
on a contract, the timing *is* part of the interface.

**2. Regenerate the bindings.** The commented-out recipe at the top of
`bot/build.sh`:

```bash
wit-bindgen rust ../wit/world.wit --out-dir .
mv ./line_follower_robot.rs src/wasm_bindings.rs
```

This is the step nobody expects to be manual. Worth a sentence on why the generated
file is checked in: so a participant can build the robot without the `wit-bindgen`
CLI, and so the diff of a contract change is *visible in review*. A contract change
should be a reviewable event.

**3. The data types.** `GyroData` (rad/s) and `ImuFusedData` (radians), plus their
`From<Vec3>` conversions and the `SensorsData` fields. Note the axis mapping in those
conversions — `roll` from `y`, `pitch` from `x`, `yaw` from `z`. That is a *choice*
tied to the Z-up convention from Lesson 04, and it is exactly the kind of thing that
must be written down somewhere or it will be rediscovered painfully.

**4. The seam.** Two methods on `SimulationStepper`, implemented twice: `RunnerStepper`
forwards to `SensorsData`, `MockStepper` returns zeros. Mention that adding a trait
method means every implementor must be updated — which sounds like a cost and is
actually the seam doing its job.

**5. The host.** The `FutureOperation` variants; `compute_value` reading from
`stepped_data` rather than from the stepper (this is the latching — a point worth
dwelling on); `ready_steps()` returning 2 and 10; the `SteppedData` fields and the
modulo sampling in `step()`.

The detail to make them notice: line sensors are read *live* from the stepper, but the
gyro and IMU are read from the **latch**. Ask why the asymmetry. Because a gyro that
integrates over 2 ms genuinely cannot give you a fresh value on demand, and pretending
otherwise would let a robot poll its way to a higher effective sample rate. The latch
is not an optimisation, it is the model.

**6. The Bevy system.** `compute_imu_data`, which is only six lines — Rapier already
has the angular velocity, and the orientation comes from the transform's Euler angles.
Then registering it in `SensorsModelPlugin`'s chain, in the right position (`store_data`
in `data.rs` is ordered `.after(compute_imu_data)`, so the recording sees the fresh
values).

**7. The robot API.** `blocking_api::read_gyro` and its async twin, unpacking three
`i16`s out of the eight bytes.

## Verification

```bash
cd sim && cargo test && cargo run --release -p sim -- test
cd ../bot && ./build.sh
```

**This is the one lesson that starts with all 38 tests green**, because the
assertions covering the removed devices had to be removed too (they referenced
constants that no longer exist, so the branch would not compile). `wasm_host_tests.rs`
has a marked spot where the `ready_steps` assertions were; tell them to put those
back as they go. Mention it up front - a lesson with no red bar feels unmoored after
five that had one.

Then actually *use* it: have them add a gyro read to a robot and log it while driving
in `test` mode. The yaw rate should be near zero on the straight and clearly non-zero
in a corner, with the sign matching the turn direction.

The deeper check is the one connecting back to the slides: last year's material makes
the case that the best `KD` tuning uses **actual** rotation speed (from the gyro)
minus **apparent** line rotation speed (from dE/dt). With the gyro restored, that
algorithm becomes implementable. Point that out — the capstone hands them the sensor
the advanced control strategy needs.

## Notes to self

- Assign this only to someone who finished a core slot early *and* looked comfortable.
  It is not hard, but it is wide, and being lost across eight files is discouraging.
- The regeneration step is the one that trips people. Check they have `wit-bindgen`
  available, or hand them the pre-regenerated file and let them do the other seven
  layers.
- This is the best answer to "how would I add a distance sensor / a camera / a second
  line array?" — which is the question a participant building their own simulator will
  actually have. Frame it that way.
