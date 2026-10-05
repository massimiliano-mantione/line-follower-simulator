use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::BotBodyMarker;
use super::motors::{Motors, Wheel};
use super::sensors::bot_position::BotPositionDetector;
use super::sensors::line_sensors::LineSensor;
use crate::app_builder::BotConfigWrapper;

// The constants below are used by EXERCISE 4.1 and 4.2 once implemented.
#[allow(dead_code)]
const BOT_COLLISION_GROUP: Group = Group::GROUP_1;

const BOT_BODY_HEIGHT: f32 = 0.01;
const BOT_BUMPER_DIAMETER: f32 = BOT_BODY_HEIGHT / 2.0;
const BOT_BODY_TO_WHEEL: f32 = 0.003;

#[allow(dead_code)]
const BOT_BODY_WEIGHT: f32 = 0.1;
#[allow(dead_code)]
const BOT_WHEEL_QUAD_DENSITY: f32 = 20.0;

// The geometry below is all derived from the robot's configuration and is used by
// EXERCISE 4.1 and 4.2 once they are implemented.
#[allow(unused_variables)]
pub fn setup_bot_model(
    mut commands: Commands,
    config_wrapper: Res<BotConfigWrapper>,
    body_query: Query<Entity, With<BotBodyMarker>>,
    wheels_query: Query<(Entity, &Wheel)>,
) {
    let config = &config_wrapper.config;

    // Axle width from wheel to wheel (in mm, 100 to 200)
    let width_axle: f32 = config.width_axle / 1000.0;
    // Length from wheel axles to front (in mm, 100 to 300)
    let length_front: f32 = config.length_front / 1000.0;
    // Length from wheel axles to back (in mm, 10 to 50)
    let length_back: f32 = config.length_back / 1000.0;
    // Clearing from robot to ground at the robot back (in mm, from 1 to wheels radius)
    let clearing_back: f32 = config.clearing_back / 1000.0;
    // Diameter of robot wheels (in mm, from 20 to 40)
    let wheel_diameter: f32 = config.wheel_diameter / 1000.0;
    // Transmission gear ratio numerator (from 1 to 100)
    let gear_ratio_num: u32 = config.gear_ratio_num;
    // Transmission gear ratio denumerator (from 1 to 100)
    let gear_ratio_den: u32 = config.gear_ratio_den;
    // Spacing of line sensors (in mm, from 1 to 15)
    let front_sensors_spacing: f32 = config.front_sensors_spacing / 1000.0;
    // Height of line sensors from the ground (in mm, from 1 to wheels radius)
    let front_sensors_height: f32 = config.front_sensors_height / 1000.0;

    let body_world = Vec3::new(0.0, 0.0, wheel_diameter / 2.0);

    let bodypart_body = Vec3::new(
        0.0,
        0.0,
        clearing_back + (BOT_BODY_HEIGHT * 0.5) + BOT_BUMPER_DIAMETER,
    );

    // Cylinder bumpers
    let front_bumper_world = Vec3::new(0.0, length_front, BOT_BUMPER_DIAMETER / 2.0);
    let back_bumper_world = Vec3::new(0.0, -length_back, BOT_BUMPER_DIAMETER / 2.0 + clearing_back);

    let body_front_length = length_back / 2.0;

    let body_width = width_axle - 2.0 * BOT_BODY_TO_WHEEL;
    let bumper_width = body_width / 2.0;

    // Static body with motors
    let body = body_query.single().unwrap();
    commands.entity(body).apply_scene(bsn! {
        // EXERCISE 4.1: give the chassis a physical body.
        //
        // The dimensions above are already derived from the robot's own
        // configuration - your job is the physics. You need:
        //
        //  - a compound collider: the main body cuboid at `bodypart_body`, a shorter
        //    front cuboid `body_front_length` long, and a capsule bumper at each of
        //    `front_bumper_world` and `back_bumper_world`. Every part is positioned
        //    relative to `body_world`, which is why the constants above subtract it.
        //  - a dynamic rigid body.
        //  - friction: the chassis should *slide* rather than catch, so a low
        //    coefficient combined with `CoefficientCombineRule::Min`.
        //  - a mass of BOT_BODY_WEIGHT.
        //  - a collision group. Without one the wheels collide with the body they
        //    are jointed to and the robot explodes on spawn. Put every bot part in
        //    BOT_COLLISION_GROUP and have it collide with everything *except* that.
        //
        // In `bsn!`, a value built by a function or an enum variant (a `Collider`,
        // `RigidBody::Dynamic`, ...) goes in `template_value(...)`; a struct with
        // named fields (`Friction { .. }`) can be written directly. Or insert them
        // with a plain `commands.entity(body).insert((...))` if you prefer.
        //
        // See: Collider::{compound, cuboid, capsule_x}, RigidBody, Friction,
        //      ColliderMassProperties, CollisionGroups
        Transform { translation: body_world }
        GlobalTransform
        template(move |_| Ok(Motors::new(gear_ratio_num, gear_ratio_den)))
        BotPositionDetector
        ExternalForce
        Velocity
    });

    // Wheels
    for (entity, wheel) in wheels_query {
        let side = wheel.side;
        let wheel_world = Vec3::new(width_axle / 2.0 * -side.sign(), 0.0, wheel_diameter / 2.0);

        commands.entity(entity).apply_scene(bsn! {
            // EXERCISE 4.2: give each wheel a body, and bolt it to the chassis.
            //
            //  - a collider. Note the reference solution uses a *ball*, not a
            //    cylinder: a cylinder contacting a plane along an edge makes contact
            //    points flicker between rim and face and the robot jitters, while a
            //    sphere touches at one point and is also a fair model of a tyre's
            //    contact patch. Try a cylinder if you want to see it.
            //  - a dynamic rigid body, high friction combined with
            //    `CoefficientCombineRule::Max` so the wheel grips whatever it
            //    touches, and a mass of
            //    BOT_WHEEL_QUAD_DENSITY * wheel_diameter * wheel_diameter.
            //  - the same collision group as the chassis.
            //  - a revolute joint to `body`, with one rotational degree of freedom
            //    about the axle. Mind the two frames: anchor 1 is where the axle
            //    sits in the *body's* frame (`wheel_world - body_world`), anchor 2
            //    is the wheel's centre in its *own* frame. Swap them and the wheel
            //    orbits the robot instead of spinning on it.
            //
            // `ImpulseJoint` has no `Default`, so `bsn!` cannot take it through
            // `template_value(...)`: build it before the macro and add it with
            // `template(move |_| Ok(joint))`.
            //
            // See: Collider::ball, ImpulseJoint::new, TypedJoint::RevoluteJoint,
            //      RevoluteJointBuilder::{new, local_anchor1, local_anchor2}
            Transform { translation: wheel_world }
            Velocity
            ExternalForce
        });
    }

    // Sensors
    for i in (0..16).into_iter().map(|i| i as f32 - 7.5) {
        let sensor_world = Vec3::new(
            i * front_sensors_spacing,
            length_front,
            front_sensors_height,
        );
        let sensor_body = sensor_world - body_world;

        commands.spawn_scene(bsn! {
            ChildOf(body)
            Transform { translation: sensor_body }
            LineSensor
        });
    }
}
