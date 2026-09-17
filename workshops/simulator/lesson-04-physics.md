# Lesson 04 — Rapier bodies, joints and motors

> **Goal.** Build the robot's physical model from the configuration its own WASM
> component declared, and turn PWM duty cycles into wheel torque.

At the end of this one you drive your own robot around with the arrow keys.

## Before you start: two things about this world

**It is Z-up.** X is across the axle, Y is forward, **Z is height**. Bevy's
convention is Y-up; we use Z-up because that is what CAD, robotics and the track
plane all use. Half the confusion in this lesson comes from this one fact.

**The robot describes its own body.** `setup()` in the WASM component returns axle
width, front and back length, wheel diameter, ground clearance and sensor geometry.
All the dimensions in `setup_bot_model` are already derived from it — your job is the
physics, not the arithmetic.

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

<details>
<summary>Hint 1 — everything falls sideways</summary>

Y-up thinking. Look at the gravity vector in 4.5.
</details>

<details>
<summary>Hint 2 — the robot explodes the instant it spawns</summary>

The wheels are colliding with the chassis they are jointed to. Every bot part needs
to be in `BOT_COLLISION_GROUP` and to collide with everything *except* that group.
`CollisionGroups::new(memberships, filter)` — and `!group` is how you say "not this
one".
</details>

<details>
<summary>Hint 3 — the wheels orbit the robot instead of spinning</summary>

The two joint anchors are in different frames. `local_anchor1` is in the *body's*
frame, so it is where the axle sits relative to the body's origin; `local_anchor2` is
in the *wheel's* own frame, so it is the wheel's centre. One of them is not the value
you think it is.
</details>

<details>
<summary>Hint 4 — it trembles, or the joints look springy</summary>

The length unit in 4.5. Our robot is two orders of magnitude smaller than what
Rapier's default tolerances assume.
</details>

<details>
<summary>Hint 5 — it drives, but it never squats under acceleration</summary>

You are applying torque to the wheels but not the reaction to the chassis. Sum the
negated wheel torques and put the total on the `Motors` entity's `ExternalForce`.
</details>

<details>
<summary>Hint 6 — it drives straight but goes wrong in turns</summary>

You are applying torque along a world axis instead of the wheel's current axle. The
axle has to be rotated by the wheel's transform first.
</details>

## Things worth breaking on purpose

Once it works, try these — they are more instructive than the working version:

- delete the collision groups;
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
