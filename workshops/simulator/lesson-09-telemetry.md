# Lesson 09 — Telemetry

> **Optional.** Short, and the most directly useful thing in the day if you ever
> build a real robot.

> **Goal.** Design a packed binary sample, declare its CSV schema, buffer it during
> the race, flush it at the end — then go and look at the data.

Needs the WASM toolchain: `rustup target add wasm32-wasip2`.

## The setup — it is a numbers argument

We want to see **every decision the robot took**: timestamp, step count, all 16
sensor values, both wheel angles, both PWM outputs, the line error. About 40 bytes.
At a 500 µs period that is 2 kHz.

Stream it live over Bluetooth serial?

- 40 bytes × 2 kHz = **80 KB/s** over a BT serial link. No.
- And worse — remember lesson 02: **transmitting costs simulated time.**
  `write_line` charges 100 µs per character, so a 60-character log line is 6 ms,
  which at 2 kHz is twelve missed control cycles. *Logging inside the control loop
  destroys the thing you are trying to measure.* Most simulators let you print for
  free and therefore lie to you; this one charges you.

So invert it. **Buffer in RAM, flush once at the end:**

- 10,000 samples × 40 bytes = 400 KB. Plausible on an MCU with 512 KB.
- At 2 kHz that covers 5 seconds; at 500 Hz, 20 seconds.
- `write_file` charges 10 µs/byte, so the flush costs real simulated time — but it
  happens *after* the race, where it cannot hurt.

This is exactly what you do on real hardware, and it is why `write_file` exists in
`wit/world.wit` alongside `write_line`.

## Your task

All in `bot/src/examples/telemetry_test.rs`:

| | What |
|---|---|
| 9.1 | `TelBlock` and `csv_spec()` — the packed sample and its byte layout |
| 9.2 | allocate the buffer and get the spec, before the race |
| 9.3 | push one sample per control iteration |
| 9.4 | flush after the loop |

`csv::transmute_buf` is provided (and `unsafe`). The whole `Pid` and the host-side
CSV writer are untouched — the API is given; the *schema design* is the exercise.

**The trap** is alignment padding. The host walks your bytes using only the spec, so
the spec must account for every byte the compiler inserted, including tail padding.
That is what `csv::PAD_8` / `PAD_16` are for. Get it wrong and the CSV shears: every
row offset by a couple of bytes, every column full of plausible nonsense.

## How to verify

```bash
# point bot/src/lib.rs::run() at this example:
#     async_framework::run(examples::telemetry_test::run(4.0));
cd bot && ./build.sh
cd ../sim && cargo run --release -p sim -- run -o . -i line_follower_robot.wasm
```

You get `telemetry.bin` and `telemetry.csv`. Check the columns line up with what you
expect — if they do not, it is the padding.

## Then actually explore it

This half is the point, and it changes how you think about tuning:

- plot `err` against `time`. The shape of the error signal *is* the robot's
  character: ringing means `KD` is too low, steady offset means it is tracking the
  line off-centre.
- plot `pwm_l` and `pwm_r` together and find the saturation. Where the inner wheel
  hits its limit is where the robot **stops being able to steer**.
- plot the 16 sensor columns as a heat map over time. You can *see* the line sweeping
  across the bar — and see it leave entirely at a 90° corner.
- divide `steps` by `time`: how many control iterations you got per physics tick,
  which is your real CPU headroom.

`workshops/simulator/samples/telemetry-example.csv` is committed as a fallback — 14,000 rows from a
real race. Note its schema is **not** the one you just designed
(`time,steps,pwm_l,pwm_r,err,d,out`, no sensor columns): it predates this `TelBlock`.
That is worth noticing rather than glossing over. The schema is chosen per robot by
whoever is debugging it, which is the entire reason you design it yourself.

## Hints

<details>
<summary>Hint 1 — how do I know how much padding to declare?</summary>

`std::mem::size_of::<TelBlock>()` versus the sum of your declared column sizes. If
they differ, the difference is padding the compiler inserted and your spec has to
name it.
</details>

<details>
<summary>Hint 2 — the first few columns are right and the rest are garbage</summary>

Classic shear. One column's declared width does not match its field's actual width,
so everything after it is offset. Check each `C_*` constant against its field type.
</details>

<details>
<summary>Hint 3 — where exactly should the flush go?</summary>

After the loop, before `run` returns. Inside the loop it would charge 10 µs per byte
per iteration and wreck the control loop — which is the failure this design exists to
avoid.
</details>

## Solution

```bash
git diff ws/simulator/09-telemetry..main -- bot/src/examples/telemetry_test.rs
```
