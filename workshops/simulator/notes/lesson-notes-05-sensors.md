# Speaker notes — Lesson 05: Ray-cast light sensors

**Branch** `ws/simulator/05-sensors` · **Slot 4**, 4:15–5:05 (50 min) · ~30 min hands-on

This lesson is self-contained, mathematical, unit-testable, and visible in real time.
It comes right after lunch, which is a good moment for something with a tight
feedback loop.

**Goal.** You make sixteen reflectance sensors see a black line on a white floor —
including what happens at the *edge* of the line and when the sensor bar is *lifted*
off the ground.

---

## Removed on this branch

`sim/sim/src/bot/sensors/line_sensors.rs`

- `line_reflection_attenuation`, `line_reflection`
- `TrackSimulateLine::intersection_to_sensor_value`
- body of `compute_sensor_readings` (both the 16-sensor loop and the body ray-cast
  that drives out-of-track / track-end detection)

I kept `LineSensor`, the `point_to_new_origin` helper, and every constant as a
documented value. **And I deliberately kept
`sensors/bot_position.rs::compute_bot_position` intact**: it uses the same ray-cast
idiom, forty lines away, so you can use it as a worked reference. Have a look at it
early; how much you lean on it is up to you.

*(Point at it in the first minute.)*

## Framing (7 min) — the time is best spent here; the code is easy once the model is clear

Let's start with the physical device. A line-follower sensor is an IR LED next to a
phototransistor. It shines light down and measures how much comes back. Black paint
returns little, white card returns a lot.

Now three questions that make this interesting. Think about them for a moment before
I answer.

*(Ask the room; do not answer yet.)*

**1. What does the sensor see at the edge of the line?**

The simple answer is: black or white, depending on which side of the boundary the
centre is. That model gives you a step function, though, and a PID fed a step
function cannot steer smoothly; it can only bang-bang.

The real device has a **finite spot size**. It illuminates a cone, so it averages over
a disc on the floor. As the disc crosses the edge, the reading slides continuously
from black to white. That gradient *is* the signal a PID needs. Without it the
`err_mm` weighted mean the robots compute would be quantised to sensor positions.

And the spot radius is not a constant: it depends on how high the sensor is. The
code assumes a 45° half-aperture cone, so **transition width = sensor height**:

```rust
let transition = z;   // 45-degree cone: radius equals height
```

That single line is the whole insight, and you can derive it yourself: tan(45°) = 1.

**2. What happens when the sensor bar lifts off the ground?**

This one is hard to predict, and it is the reason the model has a second function.
When the robot accelerates, brakes or crosses a bump, the front bar rises. You might
expect a lifted sensor to read whatever is below it, just blurrier.

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
interpolation between them. The comment admits the real relation is closer to
quadratic: an honest simplification, and exactly the kind of decision you make when
you write a simulator.

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

The general principle goes far beyond this project: **let the physics engine answer
the question it is good at (what am I over?) and answer the geometric question
analytically (where exactly am I?).** Making colliders line-accurate would mean
thousands of tiny colliders, a slower broad phase, and *worse* precision.

`point_to_new_origin` (provided) does the world → segment-local transform.

## Structure of the exercise

Three sub-problems, increasing in difficulty. I suggest you do them in this order and
run the visual check after each one:

1. **Ray-cast** (10 min) — for each of the 16 `LineSensor` children, cast down and
   find the segment. You can copy the idiom from `compute_bot_position`. The
   `QueryFilter` predicate restricting hits to track segments is essential; without
   it the ray hits the robot's own bumper.
2. **Distance to line** (10 min) — the three segment types above.
3. **The two perceptual models** (10 min) — smoothstep edge, then lift attenuation.

Then the tail of `compute_sensor_readings`: a *seventeenth* ray-cast from the robot
body, whose only job is to set `is_out_of_track` and `is_over_track_end`. That is how
a race ends — no hit means the robot drove off the track, and a hit on
`TrackSegment::End` means it finished. Two booleans that decide the whole
competition.

A detail so the numbers do not surprise you: values are 0–100 internally and become
u8 0–255 for the guest, in `DeviceValueRaw::from_sensor_values` (`s * 255.0 / 100.0`).
And the noise: `rng.noisy_value(value, 1.0)` then clamp to 0–100 — seeded
`SmallRng`, so it is reproducible noise. Why add noise at all? *(Ask the room.)*
Because a robot tuned against noiseless sensors will have a `KD` term that explodes
on real hardware.

## Verification

```bash
cd sim && cargo test -p sim                          # the perceptual models
cargo run --release -p sim -- test                   # the live readout
```

The tests check full contrast at z = 2 mm, everything ≈ 50 at z = 20 mm, and
smoothstep strictly between 0 and 100 across the edge.

The visual check is the good one. The bottom panel prints all 16 values live, and the
colour tells you the body ray-cast's verdict — green on track, red off, white over
the end. Drive slowly across the line and watch the gradient sweep along the bar.
Nothing else in this workshop gives you feedback that direct.

## Walkthrough (15 min)

Let me drive across the line with the reference implementation and read the numbers
out loud. Then let's break things live:

- `transition = 0.001` (a fixed tiny spot) → the readings go binary, and the bar
  snaps rather than sweeps;
- skip the attenuation → lift makes no difference, which flatters the robot;
- drop the `QueryFilter` predicate → sensors read the robot's own chassis;
- raise `front_sensors_height` to 15 mm in the robot config and re-run → the robot
  is nearly blind, and *this is a real failure mode competitors hit*.

Next: "it drives and it sees. But everything we have watched so far was live. In the
next slot: how does a 60-second race become something you can scrub through?"

---

## If something goes wrong

- **It works on the first straight and falls apart in the first corner.** Check
  that you use the segment-**local** X, not the sensor's **world** X.
- **The robot steers confidently the wrong way in turns.** The error is mirrored:
  check `side.sign()` on the turn segments.
- **The readings are fine on the flat but wrong when the robot pitches.** The ray
  direction should be `sensor_tf.rotation().mul_vec3(Vec3::NEG_Z)`, not a hardcoded
  `Vec3::NEG_Z`.
- **Lifting the bar makes no difference.** If `z` comes from the sensor's *local*
  transform, it is a constant and the lift model never fires; use the global one.
- **The lift and edge effects look off together.** Order matters: compute
  reflectance from geometry first, *then* degrade it for height.
- **An occasional sample jumps to a weird value.** Clamp to 0–100 after adding noise,
  otherwise it can wrap when cast to u8.
