# Speaker notes — Lesson 05: Ray-cast light sensors

**Branch** `ws/simulator/05-sensors` · **Slot 4**, 4:15–5:05 (50 min) · ~30 min hands-on

The best-designed lesson of the set: self-contained, mathematical, unit-testable, and
visible in real time. Right after lunch, which is when you want something with a
tight feedback loop.

**Goal.** Make sixteen reflectance sensors see a black line on a white floor —
including what happens at the *edge* of the line and when the sensor bar is *lifted*
off the ground.

---

## Removed on this branch

`sim/sim/src/bot/sensors/line_sensors.rs`

- `line_reflection_attenuation`, `line_reflection`
- `TrackSimulateLine::intersection_to_sensor_value`
- body of `compute_sensor_readings` (both the 16-sensor loop and the body ray-cast
  that drives out-of-track / track-end detection)

Kept: `LineSensor`, the `point_to_new_origin` helper, and every constant as a
documented value. **And deliberately kept intact:
`sensors/bot_position.rs::compute_bot_position`** — the same ray-cast idiom, forty
lines away, as a worked reference. Point at it in the first minute; it is the
difficulty dial for this lesson.

## Framing (7 min) — spend the time here, the code is easy once the model is clear

Start with the physical device. A line-follower sensor is an IR LED next to a
phototransistor. It shines light down and measures how much comes back. Black paint
returns little, white card returns a lot.

Now ask the three questions that make this interesting, and *do not answer them yet*:

**1. What does the sensor see at the edge of the line?**

Naive answer: black or white, depending on which side of the boundary the centre is.
Wrong, and the wrongness matters — that model gives you a step function, and a PID
fed a step function cannot steer smoothly; it can only bang-bang.

The real device has a **finite spot size**. It illuminates a cone, so it averages over
a disc on the floor. As the disc crosses the edge, the reading slides continuously
from black to white. That gradient *is* the signal a PID needs. Without it the
`err_mm` weighted mean the robots compute would be quantised to sensor positions.

And the spot radius is not a constant — it depends on how high the sensor is. The
code assumes a 45° half-aperture cone, so **transition width = sensor height**:

```rust
let transition = z;   // 45-degree cone: radius equals height
```

That single line is the whole insight. Let them derive it: tan(45°) = 1.

**2. What happens when the sensor bar lifts off the ground?**

This is the one nobody predicts, and it is the reason the model has a second
function. When the robot accelerates, brakes or crosses a bump, the front bar rises.
Naively, a lifted sensor reads whatever is below it, just blurrier.

In reality, as you lift the sensor the **reflected** signal weakens while the
**ambient** light it picks up stays constant. So contrast collapses: black stops
reading 0 and starts reading 30, white stops reading 100 and starts reading 70, and
both converge on ambient. At 20 mm the sensor is effectively blind — everything reads
about 50, and the robot's weighted-mean error goes to garbage.

```
value 100 ┤ white ───╮
          │           ╰──╮
       50 ┤  ambient ─────╳═════  both converge
          │           ╭──╯
        0 ┤ black ───╯
          └──┬──────────┬──────
           2mm        20mm      sensor height z
```

The constants: `Z_MIN = 0.002`, `Z_MAX = 0.02`, `VALUE_AMBIENT = 50.0`. Linear
interpolation between them (the comment admits the real relation is closer to
quadratic — an honest simplification worth pointing out as exactly the kind of
decision simulator authors make).

**3. How do we know where the line is?**

Here is the trick that makes the whole thing work, and it is worth slowing down for.

Look at the constants: `TRACK_HALF_WIDTH = 0.1` but `LINE_SIZE = 0.02`. The
*collider* is a 200 mm-wide slab — the track *surface*. The *line* is 20 mm of paint
in the middle of it, and **it has no collider at all.**

So the ray-cast does not find the line. It finds *which segment of track* the sensor
is over, and the hit point. Then the segment's own geometry tells us analytically how
far that point is from the line's centre:

- `Straight` / `Start` / `End`: the local X coordinate. Done.
- `CyrcleTurn`: `(local_point.length() − radius) * side.sign()` — distance from the
  arc's centre, minus the radius.
- `NinetyDegTurn`: whichever leg of the corner you are nearer to, selected by
  comparing against the diagonal:
  ```rust
  if local_point.y < data.side.sign() * local_point.x { local_point.x }
  else { data.side.sign() * local_point.y }
  ```

Land the general principle, because it generalises far beyond this project:
**let the physics engine answer the question it is good at (what am I over?) and
answer the geometric question analytically (where exactly am I?).** Trying to make
colliders line-accurate would mean thousands of tiny colliders, a slower broad phase,
and *worse* precision.

`point_to_new_origin` (provided) does the world → segment-local transform.

## Structure of the exercise

Three sub-problems, increasing in difficulty. Tell them to do them in this order and
to run the visual check after each one:

1. **Ray-cast** (10 min) — for each of the 16 `LineSensor` children, cast down and
   find the segment. Copy the idiom from `compute_bot_position`. The `QueryFilter`
   predicate restricting hits to track segments is essential; without it the ray hits
   the robot's own bumper.
2. **Distance to line** (10 min) — the three segment types above.
3. **The two perceptual models** (10 min) — smoothstep edge, then lift attenuation.

Then the tail of `compute_sensor_readings`: a *seventeenth* ray-cast from the robot
body, whose only job is to set `is_out_of_track` and `is_over_track_end`. That is how
a race ends — no hit means the robot drove off the track, and a hit on
`TrackSegment::End` means it finished. Two booleans that decide the whole
competition.

A detail to mention so nobody panics: values are 0–100 internally and become u8 0–255
for the guest, in `DeviceValueRaw::from_sensor_values` (`s * 255.0 / 100.0`). And the
noise: `rng.noisy_value(value, 1.0)` then clamp to 0–100 — seeded `SmallRng`, so it
is reproducible noise. Ask why add noise at all: because a robot tuned against
noiseless sensors will have a `KD` term that explodes on real hardware.

## Verification

```bash
cd sim && cargo test -p sim                          # the perceptual models
cargo run --release -p sim -- test                   # the live readout
```

Tests: full contrast at z = 2 mm; everything ≈ 50 at z = 20 mm; smoothstep strictly
between 0 and 100 across the edge.

The visual check is the good one. The bottom panel prints all 16 values live, and the
colour tells you the body ray-cast's verdict — green on track, red off, white over
the end. Drive slowly across the line and watch the gradient sweep along the bar.
Nothing else in this workshop gives feedback that direct.

## Walkthrough (15 min)

Drive across the line at the reference implementation and narrate the numbers. Then
break things live:

- `transition = 0.001` (a fixed tiny spot) → the readings go binary, and the bar
  snaps rather than sweeps;
- skip the attenuation → lift makes no difference, which flatters the robot;
- drop the `QueryFilter` predicate → sensors read the robot's own chassis;
- raise `front_sensors_height` to 15 mm in the robot config and re-run → the robot
  is nearly blind, and *this is a real failure mode competitors hit*.

Hook for Slot 5: "it drives and it sees. But everything we have watched so far was
live. Next: how does a 60-second race become something you can scrub through?"

---

## Common wrong turns

- Using the sensor's **world** X instead of the segment-**local** X. Works perfectly
  on the straight starting segment and falls apart in the first corner — a great bug
  to let them find.
- Forgetting `side.sign()` on turns, so the error is mirrored and the robot steers
  confidently the wrong way.
- Ray direction hardcoded to `Vec3::NEG_Z` instead of
  `sensor_tf.rotation().mul_vec3(Vec3::NEG_Z)`. Fine on the flat, wrong as soon as
  the robot pitches — which is exactly when you need it to be right.
- Reading `z` from the sensor's *local* transform rather than its global one. Local z
  is a constant, so the lift model silently never fires.
- Attenuating before the edge model instead of after. Order matters: compute
  reflectance from geometry, *then* degrade it for height.
- Forgetting the clamp to 0–100 after adding noise, so an occasional sample wraps
  when cast to u8.
