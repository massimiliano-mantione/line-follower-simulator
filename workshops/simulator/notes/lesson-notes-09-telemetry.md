# Speaker notes — Lesson 09: Telemetry

**Branch** `ws/simulator/09-telemetry` · **Optional pool** · ~35 min

An advertised learning outcome — "collect telemetry data sets from the simulation and
explore them" — so this lesson exists even though it is short. It is also the most
directly *useful* thing in the day for anyone who goes on to build a real robot.

**Goal.** Design a packed binary sample, declare its CSV schema, buffer it during the
race, and flush it at the end. Then look at the data.

---

## Removed on this branch

`bot/src/examples/telemetry_test.rs`

- `struct TelBlock` and its `new` / `csv_spec`
- the buffer allocation, the per-iteration `push`, and the final `write_csv_file`

Kept: `csv::transmute_buf` (provided, and `unsafe`), the whole `Pid`, and the entire
host-side CSV writer. The API is given; the *schema design* is the exercise.

## Framing (7 min) — this is mostly a numbers argument, and it is a good one

Put the constraint on the board before any code.

We want to see **every decision the robot took**: timestamp, 16 sensor values, both
wheel angles, both PWM outputs, the line error. About 40 bytes. At a 500 µs period
that is 2 kHz.

Ask: stream it over Bluetooth serial while the robot races?

- 40 bytes × 2 kHz = **80 KB/s** over a BT serial link. No.
- And worse — remember Lesson 02: **transmitting costs simulated time.** `write_line`
  charges 100 µs per character. A 60-character log line is 6 ms, which at 2 kHz is
  twelve missed control cycles. *Logging inside the control loop destroys the thing
  you are trying to measure.* Most simulators let you print for free and therefore
  lie; ours charges you, which is why this lesson belongs in this workshop.

So invert it. **Buffer in RAM, flush once at the end:**

- 10,000 samples × 40 bytes = 400 KB. On an MCU with 512 KB that is plausible; halve
  the sample or the rate if not.
- at 2 kHz, 10,000 samples cover 5 seconds — enough for the interesting part of a lap.
  At 500 Hz it covers 20 s.
- `write_file` charges 10 µs/byte, so the flush costs real simulated time — but it
  happens *after* the race, where it cannot hurt.

This is exactly what you do on real hardware, and it is why `write_file` exists in the
WIT contract alongside `write_line`.

## The pieces

**The packed sample.** `#[repr(C)]`, fixed size, no padding surprises:

```rust
#[repr(C)]
struct TelBlock {
    time: u32, steps: u32,
    vals: [u8; 16],
    w: [u16; 2], pwm: [i16; 2],
    e: i16,
    pad: u16,          // explicit tail padding
}
```

The explicit `pad` field is the teaching point. `csv::transmute_buf` reinterprets the
`Vec<TelBlock>` as bytes and the host walks it with the column spec, so **the spec
must account for every byte the compiler inserted**, including tail padding to
satisfy alignment. Hence `ValueKind::Pad8` / `Pad16` in the WIT and
`csv::col(".", csv::PAD_16)` as the last column. Get this wrong and the CSV shears —
every row offset by two bytes, columns full of plausible nonsense. That is a
genuinely instructive bug; let them hit it.

**The column spec** is a parallel array describing the layout:

```rust
[ csv::col("time", csv::C_U32), csv::col("steps", csv::C_U32),
  csv::col("s1", csv::C_U8), /* ... s16 ... */
  csv::col("wl", csv::C_U16), csv::col("wr", csv::C_U16),
  csv::col("pwm_l", csv::C_I16), csv::col("pwm_r", csv::C_I16),
  csv::col("err", csv::C_I16), csv::col(".", csv::PAD_16) ]
```

Two arrays that must agree and cannot be checked by the compiler. Ask how they would
make this safe — a derive macro, or a const assertion on
`size_of::<TelBlock>() == spec_size()`. Good five-minute discussion, and a reasonable
thing for a keen participant to actually build.

Mention `ValueKind::Named`, which maps a `u8` to strings so an enum column
(`Direction::Left`) exports as text rather than `0`. Small feature, large quality-of-
life gain when you are reading a 10,000-row CSV.

**The loop.** `Vec::with_capacity(20000)` up front — never grow mid-race, an
allocation is an unbounded latency spike. Push one sample per iteration. Flush after
the loop:

```rust
write_csv_file("telemetry", csv::transmute_buf(&tel_buf), &csv_spec);
```

## Verification and *exploration*

```bash
cd bot && ./build.sh
cd ../sim && cargo run --release -p sim -- run -o . -i line_follower_robot.wasm
```

You get `telemetry.bin` and `telemetry.csv` in the output directory.

Now do the "explore" half properly, because it is the advertised outcome and it is
the part that changes how people think:

- open the CSV and plot `err` against `time`. The shape of the error signal *is* the
  robot's character: ringing means `KD` is too low, drift means the line is being
  tracked off-centre.
- plot `pwm_l` and `pwm_r` together and find the saturation — where the inner wheel
  hits its limit is where the robot stops being able to steer.
- plot the 16 sensor columns as a heat map over time and you can *see* the line
  crossing the bar; you can see it leave entirely at a 90° corner.
- correlate `steps` against `time`: the ratio tells you how many control iterations
  the robot got per physics tick, which is its real-world CPU headroom.

`workshops/simulator/samples/telemetry-example.csv` is committed as a fallback if someone's own run
is uninteresting — 14,000 rows from a real race. Note that its schema is *not* the
one above (it predates this `TelBlock`: `time,steps,pwm_l,pwm_r,err,d,out`, no sensor
columns). That is worth pointing out rather than hiding: the schema is chosen per
robot by whoever is debugging it, which is the entire point of designing it yourself.

## Notes to self

- This is the lesson to offer someone who says "I build real robots". It is the one
  with direct transfer.
- The `transmute_buf` helper is `unsafe` and provided deliberately. If asked: yes,
  `bytemuck`/`zerocopy` is how you would do this properly, and on an MCU you would
  want `Pod` bounds rather than a blind transmute. Worth naming, not worth a detour.
