# Speaker notes — Lesson 04: Rapier bodies, joints and motors

**Branch** `ws/simulator/04-physics` · **Slot 3**, 2:40–3:30 (50 min) · ~30 min hands-on

This is the most immediately satisfying lesson: by the end of it you drive your own
robot around with the arrow keys. It is also the lesson where a small mistake gives
the most spectacular result, and those results are worth looking at together.

**Goal.** You build the robot's physical model from the configuration the WASM
component declared, and you turn PWM duty cycles into wheel torque.

This branch ships with `RapierDebugRenderPlugin` **enabled**, so you can see every
collider. Keep an eye on it: it is the difference between debugging and guessing.

---

## Removed on this branch

`sim/sim/src/bot/model.rs` — in `setup_bot_model`: the compound `Collider`,
`RigidBody::Dynamic`, `Friction`, `ColliderMassProperties`, `CollisionGroups`, and
the revolute `ImpulseJoint` per wheel.

`sim/sim/src/bot/motors.rs` — `pwm_to_torque` and the body of `apply_motors_pwm`.

`sim/sim/src/app_builder.rs` — body of `RapierPhysicsSetupPlugin::build`.

The chassis and wheel code sits inside `bsn!` blocks, and this may be the first time
you meet Bevy 0.19's scene macro. The brief and the exercise comments give you the
two rules you need (`template_value(...)` for function-built and enum values,
`template(...)` for `ImpulseJoint`). If the macro gets in your way, a plain
`.insert((...))` is perfectly fine: this slot is about physics, not about macros.

I kept every geometry constant and every derived dimension (`body_world`,
`bodypart_body`, bumper positions…). Deriving those from the config is fiddly
arithmetic that would not teach you anything. I also kept the `LineSensor`
child-spawn loop (that is Lesson 05's territory), and `Transform` / `Motors` /
`BotPositionDetector` / `Velocity` / `ExternalForce`.

Why those last ones stay: without `Velocity`, `compute_imu_data` panics on an empty
query, and you would end up hunting through `sensors/imu.rs` for a problem that is
really in `model.rs`. Declaring a component is not the physics lesson; the
colliders, bodies, joints, friction, mass and groups are.

`pwm_to_torque` returns `0.0` and `apply_motors_pwm` is empty rather than `todo!()`,
so the app runs and what you see is a robot that does not move. Gravity defaults to
Rapier's `-Y`, so the robot visibly falls sideways until you do 4.5. That is a good
place to start.

## Framing (11 min)

### How Bevy holds the world (6 min)

Until now you have used Bevy from the outside: in Slot 2 you ran its schedules by
hand and paused one of its resources. Today you put things *inside* it, so let me
spend a few minutes on how it is organised. It is not object-oriented, and if you
read the code in this lesson expecting objects, it looks strange.

Remember the picture from this morning: a database plus a scheduler.

**An entity is a row id.** Our robot is three of them, and this is all of
`setup_bot_entities`:

```rust
commands.spawn(BotBodyMarker);
for side in [Side::Left, Side::Right] {
    commands.spawn(Wheel::new(Vec3::NEG_X * side.sign(), side));
}
```

Three rows, each with a single column filled in. Nothing about them says "rigid body"
or "robot" yet.

**A component is a column.** `Wheel`, `Motors`, `Transform`, `Velocity`,
`ExternalForce`, `Collider` are plain structs. An entity "is a wheel" only in the sense
that it has a value in the `Wheel` column. `setup_bot_model`, the function you complete
today, queries those three rows and *adds* columns to them: `RigidBody`, `Collider`,
`Friction`, the joint. That is the whole act of building the robot.

**An archetype is a table.** Bevy groups the entities that have *exactly the same
set* of component types into one table, with one contiguous array per component. The
body and the wheels have different sets, so they sit in different tables; the 16 line
sensors all share one. Add or remove a component and the entity moves to another
table. Please keep that last sentence in mind: it comes back at the end of this slot.

**A system is a query plus a loop.** Here is the signature of `apply_motors_pwm`,
which you write today:

```rust
fn apply_motors_pwm(
    pwm: Res<MotorDriversDutyCycles>,
    data: Res<ExecutionData>,
    mut wheels_query: Query<(&Wheel, &Transform, &Velocity, &mut ExternalForce), Without<Motors>>,
    mut motors_query: Query<(&Motors, &mut ExternalForce), Without<Wheel>>,
)
```

Read the first query as SQL: *select `Wheel`, `Transform`, `Velocity` and a writable
`ExternalForce` from every entity that has all of them and has no `Motors`.* The two
`Res` are resources, the one-row tables: `MotorDriversDutyCycles` is what the robot
last asked for, and it is how the WASM host talks to the physics.

Why the `Without<…>` filters? *(Ask the room.)* Both queries borrow
`&mut ExternalForce`. An entity with both `Wheel` and `Motors` would match both, and the
system would hold two mutable references to the same value. Bevy checks the declared
access when it builds the system and refuses to run it unless it can prove the two
queries are disjoint. It is the borrow checker, moved from compile time to app
startup, and applied to queries. The same declarations let the scheduler run systems
in parallel when they touch different columns.

**Why build an engine this way?** Two reasons, and I want to be honest about which
one matters to us.

1. **Memory.** A query walks contiguous arrays: all the matching `Transform`s sit next
   to each other, so every 64-byte cache line the CPU fetches is full of useful data,
   and the prefetcher sees a linear scan. The object-oriented alternative is a list of
   pointers to game objects scattered over the heap, and every entity costs a cache
   miss. With a hundred thousand particles or units, that is a large factor in frame
   time, and it is why the design exists.

   In our world it buys us nothing measurable. We have three rigid bodies, 16 sensors
   and a few dozen track segments. Rapier does not even keep its state in Bevy's
   columns: bevy_rapier copies transforms and forces into Rapier's own structures each
   step, and copies the results back.

2. **Composition.** This is the one that matters here. No class hierarchy decides
   what a robot is: a robot is whatever columns you attach, and its behaviour is
   whatever systems you install. In Slot 5 you will see the same robot built two
   different ways, flat for physics and as a tree for replay, with one enum,
   `EntityFeatures`, choosing which components and systems go in. With a base class
   you would be fighting the hierarchy; here you leave out a column.

And because systems share columns rather than calling each other, `apply_motors_pwm`
never calls Rapier, and Rapier never calls it. Their order is declared separately, and
you will do that at the end of this lesson.

### This world (5 min)

Let's start from `setup()`. The WASM component declares its own body: axle width,
front and back length, wheel diameter, ground clearance, sensor spacing and height.
Our job is to turn that declaration into rigid bodies.

Then one thing that surprises almost everybody, so I want to say it before you run
into it:

> **This world is Z-up.** Gravity is `Vec3::NEG_Z * 9.81`.

Bevy's convention is Y-up. We use Z-up because that is what roboticists, CAD and ROS
use, and because the track is naturally an XY plane. Every `Vec3` in the codebase
follows it: X is left/right across the axle, Y is forward, Z is height. If something
looks strange in this lesson, this is the first thing to check.

*(Write it on the board.)*

The second thing: `RapierConfiguration::new(0.001)`.

Our robot is 10 cm long and its wheels are 25 mm across. Rapier's default tolerances
assume human-sized objects, and at our scale those tolerances are enormous compared
to the parts: bodies jitter, contacts get resolved wrongly, joints look springy.
The length-unit argument scales Rapier's internal tolerances to our world. *This is
the single setting that separates a simulator that works from one that visibly
trembles*, and it is one number.

## The pieces

**The compound collider.** Four parts: the main body cuboid, a shorter front cuboid,
and two `capsule_x` bumpers front and back. All positions are relative to
`body_world` (the body's own origin, at wheel-axle height), which is why every part
has a `- body_world`. The provided constants already give the shape, so you do not
need to redesign it.

**Wheels are spheres.** `Collider::ball(wheel_diameter / 2.0)`. You may be wondering
why. A cylinder touching a plane along an edge is numerically nasty: the contact
points flicker between the rim and the face, and the robot jitters. A sphere touches
at exactly one point, always, which is also a decent model of a real line-follower
tyre's contact patch. It is a *modelling* decision, not laziness, and it is the kind
of tradeoff you make all the time when you write a simulator.

**The revolute joint** is the core of the exercise:

```rust
ImpulseJoint::new(body, TypedJoint::RevoluteJoint(
    RevoluteJointBuilder::new(Vec3::X)
        .local_anchor1(wheel_world - body_world)   // in the body's frame
        .local_anchor2(Vec3::ZERO)                 // the wheel's own centre
        .build()))
```

One rotational degree of freedom about the axle (X). Anchor 1 is where the axle sits
in the *body's* frame; anchor 2 is the wheel's centre in its *own* frame. These two
frames are easy to mix up, and when it happens you see it immediately: the wheel
orbits the robot instead of spinning on it.

**Collision groups.** `CollisionGroups::new(BOT_COLLISION_GROUP, !BOT_COLLISION_GROUP)`
on every bot part: "I am in group 1, and I collide with everything *except* group 1."
Rapier does not skip contacts between bodies joined by a joint (`contacts_enabled`
defaults to `true`), and our wheel spheres do overlap the chassis: with the default
config, by almost 10 mm. The group is what keeps the robot from colliding with itself.

If you delete it, though, nothing dramatic happens, and the reason is a nice lesson in
contact geometry. The wheel's centre sits inside the chassis box's side-face slab, so
the contact normal runs along the axle and the contact point lies *on* the axle line.
The push is axial, which the revolute joint already resists, and it is mirrored on the
other wheel, so the forces cancel. Friction at a point on the rotation axis produces
no torque, so the wheel spins freely. Rapier also caps how fast it corrects
penetration, so the fight between contact and joint stays gentle. The robot drives.
Change the geometry (a cylinder wheel, or a chassis that ends below the axle) and the
contact moves off the axle and starts braking the wheel.

*(Delete the line live and ask why nothing breaks.)*

**Friction is where the robot's character lives.** Four different values, all
deliberate:

| Part | Coefficient | Rule | Why |
|---|---|---|---|
| wheels | 0.8 | `Max` | traction — the wheels must grip |
| body | 0.1 | `Min` | the chassis should slide, not catch |
| track segments | 0.0 | `Min` | the line must not be a speed bump |
| floor | 0.5 | — | the surface the wheels actually grip |

The combine rule matters as much as the number: `Max` on the wheels means the pair
takes the higher of the two coefficients, so the wheel grips whatever it is touching;
`Min` on the body means it slides whatever it is touching. What would happen if the
track segments had friction? *(Ask the room.)* The robot would feel the painted line,
which is absurd, but it is exactly the sort of bug that shows up as "my robot
mysteriously slows down in corners".

**Masses.** Body 0.1 kg; wheel `20.0 * d²`, so a 25 mm wheel is 12.5 g. Plausible for
a small robot, and the ratio decides whether it wheelies under acceleration.

**`pwm_to_torque`** — the DC motor model. A brushed motor's torque falls linearly
from stall torque at zero speed to zero at no-load speed:

```
T = T_stall · |pwm| · (1 − |ω_motor| / (ω_noload · |pwm|))
```

Then gearing: `motor_omega = ang_vel / gear_ratio` and
`wheel_torque = motor_torque / gear_ratio`, with `gear_ratio = num/den` (default
1/20). **Look at the comment and the code together**: the comment says "torque
amplified by gearbox … motor torque * gear_ratio" while the code divides. Both are
right given what `gear_ratio` means here (wheel revolutions per motor revolution =
0.05, so dividing amplifies by 20), but it is a genuinely confusing name. It is a
nice 30-second lesson on why physical quantities deserve unambiguous names.

This function is the **bonus** if you finish early. It ships returning 0.0, so the
robot simply does not move until you write it. A linear `torque = k * pwm` also
works and is a good first approximation: if you have time, try both and compare top
speed and acceleration.

**`apply_motors_pwm`** — two subtleties:

```rust
if !data.activity_data.is_active_now() { return; }
```

The motors are dead before the start signal and after the race ends. Without this,
robots crawl away during the 1-second countdown.

And Newton's third law, which is easy to miss and physically important:

```rust
ext_impulse.torque = torque_vec;     // on the wheel
body_torque -= torque_vec;           // reaction on the chassis
```

The motor pushes against its own mount. Leave out the reaction and the robot never
squats under acceleration, which also means the sensor bar never lifts, which means
Lesson 05's height-attenuation model never kicks in. The lessons are connected.

Finally, the schedule slot: `RunFixedMainLoopSystems::BeforeFixedMainLoop`. Forces
must be applied *before* Rapier integrates, not after.

## Verification

```bash
cd sim && cargo run --release -p sim -- test
```

Arrow keys or WASD to drive; the PWM sliders on the right set the forward and
differential magnitudes. You are done when the robot drives straight, turns, and sits
still on its wheels without sinking, jittering or launching itself.

The debug renderer shows every collider: if the wheels orbit the chassis, the joint
anchors are in the wrong frames.

## Walkthrough (15 min)

Let me drive the reference robot, and then break it live, one line at a time. Each
of these is worth seeing once:

- drop `CollisionGroups` → surprisingly, nothing visible: the wheel–chassis contact
  sits on the axle line, so it neither moves nor brakes anything;
- give the track segments friction 0.8 → it stumbles over the line;
- delete the body reaction torque → no squat under acceleration;
- set the length unit to 1.0 → the visible tremble.

### Aside: how fragile determinism is (3 min)

This one is the opposite kind of change: not a physics bug, just a *different but
equivalent* way to spawn entities. We spent Slots 1 and 2 making the simulation
deterministic to the instruction, and it is: the same binary gives bit-identical
results, run after run. But "the same binary" is doing a lot of work in that
sentence.

Look at the `LineSensor` loop at the end of `setup_bot_model` (it is still there on
this branch; it belongs to Lesson 05). It parents each sensor with `ChildOf` at spawn
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
switch between them changes the result. And the robot does not get "faster" with one
of them: on the `simple` track `add_child` wins by 129 ms, and on the `race` track it
loses by 9 ms. That is chaos, not a bias.

*(The frame count printed by `--cli` is enough to show this live: swap the two
snippets, re-run, compare.)*

Here is the mechanism; I verified it by printing the Rapier handles in both variants:

1. Remember the archetype tables from the framing: a query iterates them in the
   order they were created.
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

What I would like you to take away from this: **determinism is a property of the
whole program, not of the physics engine.** Rapier is deterministic for the same
inputs in the same order, and that order can come from code that has nothing to do
with physics. That is why our guarantee is "same binary, same result", why replay
(Lesson 06) stores trajectories instead of re-simulating, and why a simulator update
can legitimately change a recorded race time.

*(If the prebuilt robot `sim/bots/bot.wasm` changes, re-measure the table: the
numbers belong to the current robot and code.)*

Next: "it drives, but it is blind. In the next slot we give it eyes."

---

## If something goes wrong

- **Everything falls sideways.** That is Y-up gravity: look at the gravity vector
  in 4.5.
- **The wheels orbit the body.** The joint anchors are swapped, or expressed in the
  wrong frame.
- **The wheels hang in the air while the body falls away.** The wheels need
  `RigidBody::Dynamic` too.
- **It drives straight but goes wrong in turns.** The torque is applied along a
  *world* axis instead of `transform.rotation * wheel.axle`.
