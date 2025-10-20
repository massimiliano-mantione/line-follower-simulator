use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::motors::{Motors, Wheel, WheelSide};
use crate::sensors::{BotPositionDetector, LineSensor};

const BOT_COLLISION_GROUP: Group = Group::GROUP_1;

const BOT_BODY_WIDTH: f32 = 0.09;
const BOT_BODY_HEIGHT: f32 = 0.01;

const BOT_BUMPER_DIAMETER: f32 = BOT_BODY_HEIGHT / 2.0;
const BOT_BUMPER_WIDTH: f32 = BOT_BODY_WIDTH / 2.0;

const BOT_BODY_WEIGHT: f32 = 0.1;
const BOT_WHEEL_WEIGHT: f32 = 0.02;

fn setup_bot(mut commands: Commands) {
    // Axle width from wheel to wheel (in mm, 100 to 200)
    let width_axle: f32 = 100.0 / 1000.0;
    // Length from wheel axles to front (in mm, 100 to 300)
    let length_front: f32 = 100.0 / 1000.0;
    // Length from wheel axles to back (in mm, 10 to 50)
    let length_back: f32 = 20.0 / 1000.0;
    // Clearing from robot to ground at the robot back (in mm, from 1 to wheels radius)
    let clearing_back: f32 = 10.0 / 1000.0;
    // Diameter of robot wheels (in mm, from 20 to 40)
    let wheel_diameter: f32 = 20.0 / 1000.0;
    // Transmission gear ratio numerator (from 1 to 100)
    let gear_ratio_num: u32 = 1;
    // Transmission gear ratio denumerator (from 1 to 100)
    let gear_ratio_den: u32 = 1;
    // Spacing of line sensors (in mm, from 1 to 15)
    let front_sensors_spacing: f32 = 10.0 / 1000.0;
    // Height of line sensors from the ground (in mm, from 1 to wheels radius)
    let front_sensors_height: f32 = 2.0 / 1000.0;

    let body_world = Vec3::new(
        0.0,
        // (length_front - length_back) / 2.0,
        0.0,
        clearing_back + (BOT_BODY_HEIGHT * 0.5) + BOT_BUMPER_DIAMETER,
    );

    // Cylinder bumpers
    let front_bumper_world = Vec3::new(0.0, length_front, BOT_BUMPER_DIAMETER / 2.0);
    let back_bumper_world = Vec3::new(0.0, -length_back, BOT_BUMPER_DIAMETER / 2.0 + clearing_back);

    let body_front_length = length_back / 2.0;
    // Static body with motors
    let body = commands
        .spawn((
            Collider::compound(vec![
                (
                    Vec3::ZERO,
                    Quat::IDENTITY,
                    Collider::cuboid(
                        BOT_BODY_WIDTH * 0.5,
                        // (length_front + length_back) * 0.5,
                        length_back,
                        BOT_BODY_HEIGHT * 0.5,
                    ),
                ),
                (
                    Vec3::new(0.0, length_front - body_front_length / 2.0, 0.0),
                    Quat::IDENTITY,
                    Collider::cuboid(
                        BOT_BODY_WIDTH * 0.5,
                        body_front_length * 0.5,
                        BOT_BODY_HEIGHT * 0.5,
                    ),
                ),
                (
                    front_bumper_world - body_world,
                    Quat::IDENTITY,
                    Collider::capsule_x(BOT_BUMPER_WIDTH / 2.0, BOT_BUMPER_DIAMETER / 2.0),
                ),
                (
                    back_bumper_world - body_world,
                    Quat::IDENTITY,
                    Collider::capsule_x(BOT_BUMPER_WIDTH / 2.0, BOT_BUMPER_DIAMETER / 2.0),
                ),
            ]),
            RigidBody::Dynamic,
            Friction {
                coefficient: 0.1,
                combine_rule: CoefficientCombineRule::Min,
            },
            ColliderMassProperties::Mass(BOT_BODY_WEIGHT),
            CollisionGroups::new(BOT_COLLISION_GROUP, !BOT_COLLISION_GROUP),
            Transform::from_xyz(body_world.x, body_world.y, body_world.z),
            GlobalTransform::default(),
            Motors::new(Vec3::X, Vec3::NEG_X, gear_ratio_num, gear_ratio_den),
            BotPositionDetector::default(),
            ExternalForce::default(),
            Velocity::zero(),
        ))
        .id();

    // Wheels
    for side in [WheelSide::Left, WheelSide::Right] {
        let wheel_world = Vec3::new(
            (width_axle + wheel_diameter) / 2.0 * -side.sign(),
            0.0,
            wheel_diameter / 2.0,
        );

        commands.spawn((
            Collider::ball(wheel_diameter / 2.0),
            Transform::from_xyz(wheel_world.x, wheel_world.y, wheel_world.z),
            RigidBody::Dynamic,
            Friction {
                coefficient: 0.95,
                combine_rule: CoefficientCombineRule::Max,
            },
            ColliderMassProperties::Mass(BOT_WHEEL_WEIGHT),
            CollisionGroups::new(BOT_COLLISION_GROUP, !BOT_COLLISION_GROUP),
            Wheel::new(Vec3::NEG_X * side.sign(), side),
            Velocity::zero(),
            ExternalForce::default(),
            ImpulseJoint::new(
                body,
                RevoluteJointBuilder::new(Vec3::X)
                    .local_anchor1(wheel_world - body_world) // parent's local anchor
                    .local_anchor2(Vec3::ZERO),
            ),
        ));
    }

    // Sensors
    for i in [
        -7.5, -6.5, -5.5, -4.5, -3.5, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5,
    ] {
        let sensor_world = Vec3::new(
            i * front_sensors_spacing,
            length_front,
            front_sensors_height,
        );
        let sensor_body = sensor_world - body_world;

        let sensor = commands
            .spawn((
                Transform::from_xyz(sensor_body.x, sensor_body.y, sensor_body.z),
                LineSensor::default(),
            ))
            .id();
        commands.entity(body).add_child(sensor);
    }
}

pub fn add_bot_setup(app: &mut App) {
    app.add_systems(Startup, setup_bot);
}
