# Lesson 04 — Rapier bodies, joints and motors

> **Goal.** Build the robot's physical model from the configuration its own WASM
> component declared, and turn PWM duty cycles into wheel torque.

At the end of this one you drive your own robot around with the arrow keys.

## Before you start: three things about this world

**It is Z-up.** X is across the axle, Y is forward, **Z is height**. Bevy's
convention is Y-up; we use Z-up because that is what CAD, robotics and the track
plane all use. If something looks strange in this lesson, check this first.

**The robot describes its own body.** `setup()` in the WASM component returns axle
width, front and back length, wheel diameter, ground clearance and sensor geometry.
All the dimensions in `setup_bot_model` are already derived from it — your job is the
physics, not the arithmetic.

**The components go in a `bsn!` block.** The scaffolding uses Bevy 0.19's scene
macro: one component per line, no commas. Two rules cover this lesson: a value
built by a function or an enum variant goes in `template_value(...)`
(`template_value(RigidBody::Dynamic)`), and a type without `Default`, like
`ImpulseJoint`, goes in `template(move |_| Ok(joint))`. The exercise comments repeat
this where you need it. A plain `commands.entity(..).insert((...))` works just as well.

## Your task

| File | | What |
|---|---|---|
| `bot/model.rs` | 4.1 | the chassis: compound collider, rigid body, friction, mass, collision group |
| `bot/model.rs` | 4.2 | each wheel: collider, body, mass, and a revolute joint to the chassis |
| `bot/motors.rs` | 4.3 | `apply_motors_pwm` — duty cycles to torque, plus the reaction on the chassis |
| `bot/motors.rs` | 4.4 | `pwm_to_torque` — the DC motor torque-speed curve |
| `app_builder.rs` | 4.5 | gravity and the physics length unit |

Suggested order: **4.5 first** (one line, and you will see the difference
immediately), then 4.1, 4.2, then 4.3 with a placeholder `pwm_to_torque`, then 4.4.

Friction is where the robot's character lives, and the four values in the reference
solution are all deliberate:

| Part | Coefficient | Combine rule | Why |
|---|---|---|---|
| wheels | 0.8 | `Max` | traction: grip whatever you touch |
| chassis | 0.1 | `Min` | slide, do not catch |
| track segments | 0.0 | `Min` | the painted line is not a speed bump |
| floor | 0.5 | — | the surface the wheels grip |

## How to verify

```bash
cd sim
cargo run --release -p sim -- test
```

Arrow keys or `WASD` to drive; the sliders on the right set the forward and
differential power. **This lesson has the Rapier debug renderer enabled**, so you can
see every collider you have built.

Success is a robot that drives straight, turns, and sits still on its wheels without
sinking, jittering or launching itself across the room.

There are no unit tests here — the feedback is the window.

## Hints

### Hint 1 — everything falls sideways

Y-up thinking. Look at the gravity vector in 4.5.

### Hint 2 — what the collision group is for

The wheel spheres overlap the chassis, and Rapier does not skip contacts between
bodies joined by a joint, so without a group the wheels collide with the chassis they
are jointed to. Every bot part needs to be in `BOT_COLLISION_GROUP` and to collide
with everything *except* that group. `CollisionGroups::new(memberships, filter)` — and
`!group` is how you say "not this one".

### Hint 3 — it flips over backwards, or sits on its belly

The two bumpers are not decoration. The front one sits exactly on the ground
(`front_bumper_world` has a height of half its diameter), so the robot rests on three
points: two wheels and the front bumper. The back one hangs `clearing_back` above the
ground. When you accelerate hard the chassis pitches backwards around the axle, and
that bumper touches down before the robot can tip over. If the robot rests on its
belly, or jumps when the simulation starts, check that every part subtracts
`body_world`. If it flips over on a hard start, check that the back bumper is in the
compound.

### Hint 4 — the wheels orbit the robot instead of spinning

The two joint anchors are in different frames. `local_anchor1` is in the *body's*
frame, so it is where the axle sits relative to the body's origin; `local_anchor2` is
in the *wheel's* own frame, so it is the wheel's centre. One of them is not the value
you think it is.

### Hint 5 — it trembles, or the joints look springy

The length unit in 4.5. Our robot is two orders of magnitude smaller than what
Rapier's default tolerances assume.

### Hint 6 — it drives, but it never squats under acceleration

You are applying torque to the wheels but not the reaction to the chassis. Sum the
negated wheel torques and put the total on the `Motors` entity's `ExternalForce`.

### Hint 7 — it drives straight but goes wrong in turns

You are applying torque along a world axis instead of the wheel's current axle. The
axle has to be rotated by the wheel's transform first.

## Things worth breaking on purpose

Once it works, try these — they are more instructive than the working version:

- delete the collision groups, and work out why nothing visibly breaks;
- give the track segments friction 0.8;
- delete the chassis reaction torque;
- set the length unit to 1.0;
- use a cylinder collider for the wheels instead of a ball.

## Solution

```bash
git diff ws/simulator/04-physics..main -- sim/sim/src/bot/model.rs sim/sim/src/bot/motors.rs sim/sim/src/app_builder.rs
```

## Where this goes next

It drives, but it is blind. Lesson 05 gives it eyes.
