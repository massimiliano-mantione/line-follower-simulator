# Lesson 10 — Declarative track segments

> **Optional.** Quietly excellent, with the funniest failure modes in the workshop.
> Good if you prefer geometry to systems.

> **Goal.** Turn a flat list of segment descriptions into a connected track — and see
> that one declaration yields three different artifacts.

## The setup

Open `track_selection.rs` and read `build_simple_track`. It is a *sentence*:

```rust
vec![ start(), straight(2.0), t90(RIGHT, 0.5), turn(120.0, LEFT, 1.0),
      t90(LEFT, 1.0), turn(60.0, RIGHT, 2.0), end() ]
```

No coordinates, no transforms. Just "go straight two metres, then turn right". Each
segment knows how long it is and how much it turns, so you can **thread a moving
frame through the list** — a tiny turtle-graphics interpreter.

And that same list produces three completely different things:

1. **Colliders** — the physics surface the robot drives on.
2. **Meshes** — the black line you see.
3. **An analytic distance field** — the per-segment "how far am I from the line
   centre" that lesson 05's sensors consume.

They are not even the same shape. `TRACK_HALF_WIDTH` is `0.1`, so the collider is a
200 mm slab of drivable surface; `LINE_HALF_WIDTH` is `0.01`, so the mesh is a 20 mm
painted stripe; and the distance field is not geometry at all, just arithmetic. One
declaration, three consumers, each taking what it needs.

## Your task

All in `sim/sim/src/track.rs`:

| | What |
|---|---|
| 10.1 | `SegmentTransform::translate_in_direction` — move forward |
| 10.2 | `SegmentTransform::rotate` — turn in place |
| 10.3 | `TrackSegment::transform` — where this segment *sits* |
| 10.4 | `TrackSegment::compute_next_origin` — where the *next* one starts |
| 10.5 | `TrackSegment::collider` — the physics surface |

All the mesh generation (`arc_mesh`, `ninety_deg_mesh`, `quad_mesh`) and
`arc_collider` are left alone — that is vertex-buffer craft.

Right now nothing advances, so the whole track is stacked at the origin.

## How to verify

Purely visual, and unusually good feedback:

```bash
cd sim
cargo run --release -p sim -- test               # `simple`, the default
cargo run --release -p sim -- -t line test       # easiest
cargo run --release -p sim -- -t angle test      # 90-degree corners
cargo run --release -p sim -- -t turn test       # arcs
cargo run --release -p sim -- -t race test       # 15 segments: the real test
```

Work up in that order. Any sign error compounds over `race`'s fifteen segments into
something spectacular.

Enable the Rapier debug renderer to see colliders against meshes — a common outcome
is a visually perfect line whose collider is rotated 90°, which the robot then drives
straight off.

## Hints

<details>
<summary>Hint 1 — 10.3 vs 10.4, I do not see the difference</summary>

10.4 is the turtle's *next position*: where the following segment begins. 10.3 is
where *this* segment's geometry should be placed, and because colliders and meshes
are centred on their own origin, that is generally the **middle** of the segment, not
its start.

For a straight of length L: the next origin is L ahead; this segment sits L/2 ahead.
</details>

<details>
<summary>Hint 2 — the arc chord in 10.4</summary>

Put the arc's centre at the local origin. Starting on the circle along +Y and
sweeping through angle `a`, the start point is at radius r and the end point is at
angle `a` around. Take the difference of those two positions, and remember that a
left turn and a right turn mirror each other.
</details>

<details>
<summary>Hint 3 — my arcs look right but lesson 05's sensors go mad over them</summary>

`transform` for `CyrcleTurn` must place the **centre of the circle**, not a point on
it. That is the contract `line_sensors.rs` relies on when it computes
`local_point.length() - radius`.
</details>

<details>
<summary>Hint 4 — the track spirals or folds through itself</summary>

Almost always a `side.sign()` sitting on the wrong term, or a rotation applied before
a translation instead of after. Try `-t angle` first: with two corners, the error is
obvious.
</details>

## Extension

If you finish early: design a new track in `track_selection.rs` and add it to the
`TrackId` enum in `main.rs`. Cheap, satisfying, and it produces something that could
actually be used next year.

## Solution

```bash
git diff ws/simulator/10-track..main -- sim/sim/src/track.rs
```
