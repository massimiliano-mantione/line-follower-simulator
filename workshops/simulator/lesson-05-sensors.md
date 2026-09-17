# Lesson 05 — Ray-cast light sensors

> **Goal.** Make sixteen reflectance sensors see a black line on a white floor —
> including what happens at the *edge* of the line, and when the sensor bar is
> *lifted* off the ground.

Self-contained, mathematical, and you can watch it work in real time.

## Before you start

**There is a worked example next door.** `sensors/bot_position.rs` contains
`compute_bot_position`, which uses the exact ray-cast idiom you need. Read it first.

**The sensor is a device, not a point sampler.** An IR LED next to a
phototransistor: it shines light down and measures how much comes back. Black paint
returns little, white card returns a lot. Two consequences that the naive model
misses, and they are the content of this lesson:

1. It illuminates a **cone**, so it averages over a disc on the floor. As the disc
   crosses the line's edge, the reading slides *continuously* from black to white.
   That gradient is what a PID steers on — a step function only permits bang-bang
   control.

2. **Lifting it collapses contrast.** The reflected signal weakens while ambient
   light stays constant, so black and white both converge on mid-scale. Lifted far
   enough the sensor is blind.

**And the trick that makes it all work.** In `track.rs`, `TRACK_HALF_WIDTH` is `0.1`
but `LINE_HALF_WIDTH` is `0.01`. The *collider* is a 200 mm slab of drivable surface;
the 20 mm line is paint, and **it has no collider at all**. So the ray-cast tells you
*which segment* you are over, and the segment's geometry tells you *where on it* —
analytically. Let the physics engine answer the question it is good at, and do the
maths yourself.

## Your task

All in `sim/sim/src/bot/sensors/line_sensors.rs`. Do them in this order, running the
visual check after each:

| | What | Roughly |
|---|---|---|
| 5.1 | `compute_sensor_readings` — cast a ray per sensor, plus one from the body | 10 min |
| 5.2 | `line_reflection` — the smooth edge | 10 min |
| 5.3 | `line_reflection_attenuation` — the lift model | 10 min |
| 5.4 | `intersection_to_sensor_value` — distance to the line, per segment type | 10 min |

Right now every sensor reports pure white, so the robot is blind but the simulator
runs.

## How to verify

```bash
cd sim
cargo test -p sim                    # ten tests, currently all red
cargo run --release -p sim -- test   # the live readout
```

The tests check full contrast near the ground, everything converging on mid-scale
when lifted, a strictly intermediate reading across the edge, monotonicity, symmetry,
and the per-segment distances for straights and arcs.

The visual check is the good one. **The bottom panel prints all sixteen values
live**, and its colour is the body ray-cast's verdict — green on track, red off it,
white over the end. Drive slowly across the line and watch the gradient sweep along
the bar. Nothing else in this workshop gives feedback that direct.

## Hints

<details>
<summary>Hint 1 — 5.2, how wide is the transition?</summary>

A 45-degree half-aperture cone from height `z` illuminates a disc of radius `z`.
tan 45° = 1. So the transition width is not a constant you need to invent — it is
already one of your parameters.
</details>

<details>
<summary>Hint 2 — 5.3, what does "collapses toward ambient" mean numerically?</summary>

At `Z_MIN` the sensor reads the full 0–100 range. At `Z_MAX` it reads
`VALUE_AMBIENT`–`VALUE_AMBIENT` — a range of zero width. In between, the *available
range* narrows linearly from both ends. So: work out the min and max readable values
for this `z`, then place `value` proportionally between them.
</details>

<details>
<summary>Hint 3 — my sensors work on the straight and go mad in the first corner</summary>

You are using the sensor's **world** X coordinate instead of its position in the
*segment's* local frame. `point_to_new_origin(point, transform)` is provided for
exactly this. (This is the single most common bug in this lesson, and a good one to
have hit.)
</details>

<details>
<summary>Hint 4 — the robot steers confidently the wrong way in turns</summary>

`data.side.sign()` on the turn segments. Your distance is correct in magnitude and
mirrored in sign.
</details>

<details>
<summary>Hint 5 — the lift model never seems to do anything</summary>

Check where your `z` comes from. A sensor's *local* transform has a constant z (it is
bolted to the chassis); you want its **global** one.
</details>

<details>
<summary>Hint 6 — sensors read strange values even off the track</summary>

Either the `QueryFilter` predicate is missing, so you are hitting the robot's own
bumper, or the miss case is returning a raw 100.0 without attenuating it for height.
</details>

## Things worth breaking on purpose

- fix the transition width to something tiny — the readings go binary and the bar
  snaps rather than sweeps;
- skip the attenuation entirely — lift stops mattering, which flatters the robot;
- set `front_sensors_height` to 15 mm in the robot's config and re-run.

## Solution

```bash
git diff ws/simulator/05-sensors..main -- sim/sim/src/bot/sensors/line_sensors.rs
```

## Where this goes next

It drives and it sees. But everything so far has been live. Lesson 06 turns a
60-second race into something you can scrub through.
