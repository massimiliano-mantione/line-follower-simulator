# Speaker notes — Lesson 04: Rapier bodies, joints and motors

**Branch** `ws/simulator/04-physics` · **Slot 3**, 2:40–3:30 (50 min) · ~30 min hands-on

The most immediately satisfying lesson: at the end they drive their own robot around
with the arrow keys. It is also the one where wrong answers are funniest, so leave
time to enjoy them.

**Goal.** Build the robot's physical model from the configuration the WASM component
declared, and turn PWM duty cycles into wheel torque.

This branch ships with `RapierDebugRenderPlugin` **enabled**, so colliders are
visible. Point that out first — it is the difference between debugging and guessing.

---

## Removed on this branch

`sim/sim/src/bot/model.rs` — in `setup_bot_model`: the compound `Collider`,
`RigidBody::Dynamic`, `Friction`, `ColliderMassProperties`, `CollisionGroups`, and
the revolute `ImpulseJoint` per wheel.

`sim/sim/src/bot/motors.rs` — `pwm_to_torque` and the body of `apply_motors_pwm`.

`sim/sim/src/app_builder.rs` — body of `RapierPhysicsSetupPlugin::build`.

Kept: every geometry constant and every derived dimension (`body_world`,
`bodypart_body`, bumper positions…). Deriving those from config is fiddly arithmetic
that teaches nothing. Also kept: the `LineSensor` child-spawn loop (Lesson 05's
territory), and `Transform` / `Motors` / `BotPositionDetector` / `Velocity` /
`ExternalForce`.

Why those last ones stay: removing `Velocity` makes `compute_imu_data` panic on an
empty query, which sends the participant hunting through `sensors/imu.rs` for a bug
that is really in `model.rs`. Declaring a component is not the physics lesson; the
colliders, bodies, joints, friction, mass and groups are.

`pwm_to_torque` returns `0.0` and `apply_motors_pwm` is empty rather than `todo!()`,
so the app runs and the symptom is a robot that does not move. Gravity defaults to
Rapier's `-Y`, so it visibly falls sideways until 4.5 is done - lead with that.

## Framing (5 min)

Start from `setup()`. The WASM component declared its own body: axle width, front and
back length, wheel diameter, ground clearance, sensor spacing and height. Our job is
to turn that declaration into rigid bodies.

Then the thing nobody expects, so say it before they hit it:

> **This world is Z-up.** Gravity is `Vec3::NEG_Z * 9.81`.

Bevy's convention is Y-up. We use Z-up because that is what roboticists, CAD and ROS
use, and because the track is naturally an XY plane. Every `Vec3` in the codebase
follows it: X is left/right across the axle, Y is forward, Z is height. Half the
confusion in this slot comes from this one line, so put it on the board.

Second framing point — `RapierConfiguration::new(0.001)`:

Our robot is 10 cm long and its wheels are 25 mm across. Rapier's default tolerances
assume human-scale objects, and at our scale those tolerances are enormous relative
to the parts — bodies jitter, contacts get resolved wrongly, joints look springy.
The length-unit argument scales Rapier's internal tolerances to our world. Worth
saying out loud: *this is the single setting that separates a simulator that works
from one that visibly trembles*, and it is one number.

## The pieces

**The compound collider.** Four parts: the main body cuboid, a shorter front cuboid,
and two `capsule_x` bumpers front and back. All positions are expressed relative to
`body_world` (the body's own origin, at wheel-axle height), hence the `- body_world`
on every part. Do not let them redesign this; the provided constants give the shape.

**Wheels are spheres.** `Collider::ball(wheel_diameter / 2.0)`. Somebody will ask.
The honest answer: a cylinder contacting a plane along an edge is numerically nasty —
contact points flicker between the rim and the face and the robot jitters. A sphere
touches at exactly one point, always, which is also a decent model of a real
line-follower tyre's contact patch. It is a *modelling* decision, not laziness, and
it is the kind of tradeoff simulator authors make constantly.

**The revolute joint** is the core of the exercise:

```rust
ImpulseJoint::new(body, TypedJoint::RevoluteJoint(
    RevoluteJointBuilder::new(Vec3::X)
        .local_anchor1(wheel_world - body_world)   // in the body's frame
        .local_anchor2(Vec3::ZERO)                 // the wheel's own centre
        .build()))
```

One rotational degree of freedom about the axle (X). Anchor 1 is where the axle sits
in the *body's* frame; anchor 2 is the wheel's centre in its *own* frame. Getting
these two frames mixed up is the classic mistake and it is very visible — the wheel
orbits the robot instead of spinning on it.

**Collision groups.** `CollisionGroups::new(BOT_COLLISION_GROUP, !BOT_COLLISION_GROUP)`
on every bot part: "I am in group 1, and I collide with everything *except* group 1."
Without it the wheels collide with the body they are joined to, and the robot
explodes on spawn. Worth demonstrating — delete the line live, it is spectacular.

**Friction is where the robot's character lives.** Four different values, all
deliberate:

| Part | Coefficient | Rule | Why |
|---|---|---|---|
| wheels | 0.8 | `Max` | traction — the wheels must grip |
| body | 0.1 | `Min` | the chassis should slide, not catch |
| track segments | 0.0 | `Min` | the line must not be a speed bump |
| floor | 0.5 | — | the surface the wheels actually grip |

The combine rule matters as much as the number: `Max` on the wheels means the pair
takes the higher of the two coefficients, so the wheel grips regardless of what it is
touching; `Min` on the body means it slides regardless. Ask what happens if the track
segments had friction — the robot would feel the painted line, which is absurd, but
it is exactly the sort of bug that produces "my robot mysteriously slows in corners".

**Masses.** Body 0.1 kg; wheel `20.0 * d²`, so a 25 mm wheel is 12.5 g. Plausible for
a small robot, and the ratio is what determines whether it wheelies under
acceleration.

**`pwm_to_torque`** — the DC motor model. A brushed motor's torque falls linearly
from stall torque at zero speed to zero at no-load speed:

```
T = T_stall · |pwm| · (1 − |ω_motor| / (ω_noload · |pwm|))
```

Then gearing: `motor_omega = ang_vel / gear_ratio` and
`wheel_torque = motor_torque / gear_ratio`, with `gear_ratio = num/den` (default
1/20). **Point out the comment/code mismatch**: the comment says "torque amplified by
gearbox … motor torque * gear_ratio" while the code divides. Both are right given
what `gear_ratio` means here (wheel revolutions per motor revolution = 0.05, so
dividing amplifies by 20) — but it is a genuinely confusing name, and it is a nice
30-second lesson on why physical quantities deserve unambiguous names.

This function is the **bonus** for fast finishers. Ship it as `todo!()` returning 0.0
so the robot simply does not move, and let the ones who finish early implement the
curve. A linear `torque = k * pwm` also works and is a good first approximation —
have them compare top speed and acceleration.

**`apply_motors_pwm`** — two subtleties:

```rust
if !data.activity_data.is_active_now() { return; }
```

Dead motors before the start signal and after the race ends. Without this, robots
crawl away during the 1-second countdown.

And Newton's third law, which is easy to miss and physically important:

```rust
ext_impulse.torque = torque_vec;     // on the wheel
body_torque -= torque_vec;           // reaction on the chassis
```

The motor pushes against its own mount. Omit the reaction and the robot never
squats under acceleration — which also means the sensor bar never lifts, which
means Lesson 05's height-attenuation model never triggers. The lessons are connected.

Finally, the schedule slot: `RunFixedMainLoopSystems::BeforeFixedMainLoop`. Forces
must be applied *before* Rapier integrates, not after.

## Verification

```bash
cd sim && cargo run --release -p sim -- test
```

Arrow keys or WASD to drive; the PWM sliders on the right set the forward and
differential magnitudes. Success is a robot that drives straight, turns, and sits
still on its wheels without sinking, jittering or launching itself.

The debug renderer shows every collider — if the wheels orbit the chassis, the joint
anchors are in the wrong frames.

## Walkthrough (15 min)

Drive the reference robot, then break it live, one line at a time — it is the best
teaching tool in this lesson:

- drop `CollisionGroups` → the robot self-destructs on spawn;
- give the track segments friction 0.8 → it stumbles over the line;
- delete the body reaction torque → no squat under acceleration;
- set the length unit to 1.0 → the visible tremble.

### Aside: how fragile determinism is (3 min)

Worth a few minutes after the break-it-live list, because it is the opposite kind of
change: not a physics bug, just a *different but equivalent* way to spawn entities.
We spent Slots 1 and 2 making the simulation deterministic to the instruction, and
it is: the same binary gives bit-identical results, run after run. But "the same
binary" is doing a lot of work in that sentence.

Look at the `LineSensor` loop at the end of `setup_bot_model` (still present on this
branch; it is Lesson 05's territory). It parents each sensor with `ChildOf` at spawn
time:

```rust
commands.spawn_scene(bsn! {
    ChildOf(body)
    Transform { translation: sensor_body }
    LineSensor
});
```

Before the code moved to `bsn!`, it spawned the sensor first and attached it after:

```rust
let sensor = commands.spawn((Transform::from_translation(sensor_body), LineSensor)).id();
commands.entity(body).add_child(sensor);
```

The two produce the same entity tree with the same components. The sensors have no
collider and no rigid body, so Rapier ignores them. Yet the race result changes
(run on `main`, `cd sim && cargo run --release -p sim -- run --cli -i bots/bot.wasm`,
add `-t race` for the second row):

| Track | `ChildOf` at spawn | `add_child` afterwards |
|---|---|---|
| `simple` | 21.3645 s, 44729 frames | 21.2355 s, 44471 frames |
| `race` | 49.2115 s, 100423 frames | 49.2205 s, 100441 frames |

The robot finishes both races either way. Each variant repeats exactly; only the
switch between them changes the result. The robot does not get "faster" with one of
them: on the `simple` track `add_child` wins by 129 ms, and on the `race` track it
loses by 9 ms. That is chaos, not a bias. The frame count printed by `--cli` is
enough to show the difference live: swap the two snippets, re-run, compare.

The mechanism, verified by printing the Rapier handles in both variants:

1. Bevy stores entities by *archetype* (their exact set of components), and a query
   iterates archetypes in the order they were created.
2. `spawn` + `add_child` moves each sensor through an extra archetype (without
   `ChildOf`) before its final one. `ChildOf` at spawn skips it. Different history,
   different archetype order.
3. bevy_rapier creates the Rapier colliders by iterating a Bevy query. The rigid-body
   handles come out identical, but the **collider handles do not**: the chassis
   collider is handle 0 in one variant and handle 2 in the other.
4. Rapier's internal ordering follows those handles, and floating-point addition is
   not associative: sum the same contact forces in a different order and the last
   bit can change. The chassis position already differs in the last bit at the
   second physics step, while the robot is still waiting for the start signal. A
   line follower is a feedback loop, so one bit grows into 129 ms by the finish line.

The takeaway for the room: **determinism is a property of the whole program, not of
the physics engine.** Rapier is deterministic for the same inputs in the same order,
and that order can come from code that has nothing to do with physics. That is why
our guarantee is "same binary, same result", why replay (Lesson 06) stores
trajectories instead of re-simulating, and why a simulator update can legitimately
change a recorded race time.

If you swap the prebuilt robot (`sim/bots/bot.wasm`), re-measure the table; the
numbers belong to the current robot and code.

Hook for Slot 4: "it drives, but it is blind. Next we give it eyes."

---

## Common wrong turns

- Y-up thinking. Everything falls sideways. Cheap to diagnose: look at gravity.
- Joint anchors swapped or expressed in the wrong frame — wheels orbit the body.
- Forgetting `RigidBody::Dynamic` on the wheels — they hang in the air, jointed to a
  body that falls away.
- Applying torque in *world* axes instead of `transform.rotation * wheel.axle`. Drives
  correctly in a straight line and then diverges hilariously in turns.
