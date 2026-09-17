# Speaker notes — Lesson 10: Declarative track segments

**Branch** `ws/simulator/10-track` · **Optional pool** · ~40 min

A quietly excellent lesson with the funniest failure modes in the workshop. Good for
anyone who prefers geometry to systems.

**Goal.** Turn a flat list of segment descriptions into a connected track — and see
that one declarative description yields *three* different artifacts.

---

## Removed on this branch

`sim/sim/src/track.rs`

- `TrackSegment::{transform, compute_next_origin, collider}`
- `SegmentTransform::{translate_in_direction, rotate}`

Kept: all mesh generation (`arc_mesh`, `ninety_deg_mesh`, `quad_mesh`,
`arc_collider`). That is vertex-buffer craft; it would eat the slot and teach little.

## Framing (6 min)

Open `track_selection.rs` and read `build_simple_track` aloud. It is a *sentence*:

```rust
vec![ start(), straight(2.0), t90(RIGHT, 0.5), turn(120.0, LEFT, 1.0),
      t90(LEFT, 1.0), turn(60.0, RIGHT, 2.0), end() ]
```

No coordinates. No transforms. Just "go straight two metres, then turn right". Ask
how that becomes a track, and let them arrive at the answer: each segment knows how
long it is and how much it turns, so you can **thread a moving frame through the
list** — a tiny turtle-graphics interpreter.

Then the point that makes this lesson worth doing. That same list produces three
completely different things:

1. **Colliders** — the physics surface the robot drives on (`collider()`).
2. **Meshes** — the black line you see (`mesh()`).
3. **An analytic distance field** — the per-segment "how far am I from the line
   centre" that Lesson 05's sensors consume.

And crucially they are *not the same shape*. `TRACK_HALF_WIDTH = 0.1` — the collider
is a 200 mm slab of drivable surface. `LINE_HALF_WIDTH = 0.01` — the mesh is a 20 mm
painted stripe. The distance field is not geometry at all, just arithmetic. One
declaration, three consumers, each taking what it needs. That is the design lesson,
and it generalises well beyond tracks.

## The pieces

**`SegmentTransform`** is the turtle: a `position: Vec2` and a `direction: Angle`.

```rust
pub fn translate_in_direction(&self, translation: Vec2) -> Self {
    Self { position: self.position + rotate_vec2(translation, self.direction.to_radians()),
           direction: self.direction }
}
pub fn rotate(&self, rotation: Angle) -> Self { /* direction += rotation */ }
```

Both return a new value rather than mutating. Immutability matters here: `spawn_bundles`
threads the origin through the list, and `transform()` must be able to compute a
segment's *own* placement without disturbing the chain.

**`compute_next_origin`** — where the next segment starts. Per type:

- `Straight`: forward by `length`.
- `Start` / `End`: forward by `TRACK_TIPS_LENGTH` (0.5 m of lead-in and run-out).
- `NinetyDegTurn`: forward *and* sideways by `line_half_length`, then rotate 90° —
  a square corner, so the exit is offset on both axes.
- `CyrcleTurn`: the chord of the arc, then rotate by the arc angle:
  ```rust
  Vec2::new(radius * (angle.cos() - 1.0) * side.sign(), radius * angle.sin())
  ```
  Let them derive this. It is the only real trigonometry in the workshop, and getting
  the sign convention right is most of the work.

**`transform`** — the segment's own placement, which is *not* its start point.
Colliders and meshes are built centred on their own origin, so each type shifts:
straights and tips by half their length, `CyrcleTurn` sideways by `radius` to sit at
the *centre of the circle* (which is why `line_sensors.rs` can use
`local_point.length() - radius`).

This split — "where do I sit" versus "where does the next one start" — is the thing to
make explicit. Conflating them is the most common bug and produces a track that
drifts apart segment by segment.

**`collider`** — per type, from the kept mesh/collider helpers. The `NinetyDegTurn`
compound of two overlapping cuboids is worth a look: a square corner is two
rectangles, not one rotated one.

## Verification

```bash
cd sim && cargo run --release -p sim -- test
cargo run --release -p sim -- -t race test     # the hard one: 15 segments
```

Purely visual, and unusually good feedback. The `race` track is the real test: any
sign error compounds over fifteen segments into something spectacular. Try each
track in turn — `line`, `angle`, `turn`, `simple`, `race` — in increasing difficulty.

Enable the Rapier debug renderer to see the colliders against the meshes; a common
outcome is a visually perfect line whose collider is rotated 90°, which the robot
then drives straight off.

## Notes to self

- The failure modes are genuinely funny — tracks that spiral, fold through
  themselves, or shoot off to infinity. Encourage screenshots; it keeps the mood up
  in the optional pool.
- If someone finishes fast, the natural extension is designing a new track in
  `track_selection.rs` and adding it to the `TrackId` enum. Cheap, satisfying, and
  it produces something you could actually use next year.
- The `Angle` newtype in `utils.rs` exists because this file used to mix degrees and
  radians and it went badly. Worth thirty seconds on units-as-types.
