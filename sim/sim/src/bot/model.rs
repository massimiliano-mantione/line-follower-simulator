use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::BotBodyMarker;
use super::motors::{Motors, Wheel};
use super::sensors::bot_position::BotPositionDetector;
use super::sensors::line_sensors::LineSensor;
use crate::app_builder::BotConfigWrapper;

const BOT_COLLISION_GROUP: Group = Group::GROUP_1;

const BOT_BODY_HEIGHT: f32 = 0.01;
const BOT_BUMPER_DIAMETER: f32 = BOT_BODY_HEIGHT / 2.0;
const BOT_BODY_TO_WHEEL: f32 = 0.003;

const BOT_BODY_WEIGHT: f32 = 0.1;
const BOT_WHEEL_QUAD_DENSITY: f32 = 20.0;

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
    let collider = Collider::compound(vec![
        (
            bodypart_body - body_world,
            Quat::IDENTITY,
            Collider::cuboid(
                body_width * 0.5,
                // (length_front + length_back) * 0.5,
                length_back,
                BOT_BODY_HEIGHT * 0.5,
            ),
        ),
        (
            Vec3::new(0.0, length_front - body_front_length / 2.0, 0.0) + bodypart_body
                - body_world,
            Quat::IDENTITY,
            Collider::cuboid(
                body_width * 0.5,
                body_front_length * 0.5,
                BOT_BODY_HEIGHT * 0.5,
            ),
        ),
        (
            front_bumper_world - body_world,
            Quat::IDENTITY,
            Collider::capsule_x(bumper_width / 2.0, BOT_BUMPER_DIAMETER / 2.0),
        ),
        (
            back_bumper_world - body_world,
            Quat::IDENTITY,
            Collider::capsule_x(bumper_width / 2.0, BOT_BUMPER_DIAMETER / 2.0),
        ),
    ]);
    commands.entity(body).apply_scene(bsn! {
        template_value(collider)
        template_value(RigidBody::Dynamic)
        Friction {
            coefficient: 0.1,
            combine_rule: CoefficientCombineRule::Min,
        }
        template_value(ColliderMassProperties::Mass(BOT_BODY_WEIGHT))
        template_value(CollisionGroups::new(BOT_COLLISION_GROUP, !BOT_COLLISION_GROUP))
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

        let joint = ImpulseJoint::new(
            body,
            TypedJoint::RevoluteJoint(
                RevoluteJointBuilder::new(Vec3::X)
                    .local_anchor1(wheel_world - body_world) // parent's local anchor
                    .local_anchor2(Vec3::ZERO)
                    .build(),
            ),
        );

        commands.entity(entity).apply_scene(bsn! {
            template_value(Collider::ball(wheel_diameter / 2.0))
            Transform { translation: wheel_world }
            template_value(RigidBody::Dynamic)
            Friction {
                coefficient: 0.8,
                combine_rule: CoefficientCombineRule::Max,
            }
            template_value(ColliderMassProperties::Mass(
                BOT_WHEEL_QUAD_DENSITY * wheel_diameter * wheel_diameter,
            ))
            template_value(CollisionGroups::new(BOT_COLLISION_GROUP, !BOT_COLLISION_GROUP))
            Velocity
            ExternalForce
            template(move |_| Ok(joint))
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
