use std::f32::consts::{FRAC_PI_2, PI};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::text::cosmic_text::Angle;
use bevy_editor_cam::DefaultEditorCamPlugins;
use bevy_editor_cam::prelude::{EditorCam, OrbitConstraint};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::IntegrationParameters;

const FLOOR_HEIGHT: f32 = 0.03;
const FLOOR_SIZE: f32 = 20.0;
const TRACK_HALF_WIDTH: f32 = 0.1;
const TRACK_HALF_HEIGHT: f32 = 0.001;
const TRACK_TIPS_LENGTH: f32 = 0.5;
const TRACK_Z_OFFSET: f32 = -TRACK_HALF_HEIGHT - FLOOR_HEIGHT;
const TRACK_CIRCLE_SEGMENTS_PER_PI: usize = 40;

fn line_reflection(x: f32) -> f32 {
    const LINE_SIZE: f32 = 0.02; // 20 mm

    // Model: black line of width LINE_SIZE centered at 0 on a white floor.
    // The sensor doesn't have infinite spatial resolution, so we smooth the
    // transition between black and white across a finite transition region.
    // We return 0.0 for pure black, 100.0 for pure white.

    // Transition width (how quickly the sensor goes from black to white).
    // Using one line-width gives a reasonably soft sensor response; tweak if needed.
    const TRANSITION: f32 = LINE_SIZE;

    // Treat NaN/inf as far away (white)
    if !x.is_finite() {
        return 100.0;
    }

    let half = LINE_SIZE * 0.5;
    let d = x.abs();

    if d <= half {
        // Fully over the black line
        0.0
    } else if d >= half + TRANSITION {
        // Far enough to see full white
        100.0
    } else {
        // Smooth interpolation between black and white using smoothstep
        let t = (d - half) / TRANSITION; // normalized 0..1
        // smoothstep (cubic hermite) -> smooth start/end
        let s = t * t * (3.0 - 2.0 * t);
        100.0 * s
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn sign(&self) -> f32 {
        match self {
            Side::Left => 1.0,
            Side::Right => -1.0,
        }
    }
}

/// Helper to rotate a Vec2 by angle in radians
/// # Arguments
/// * `v`     - The vector to rotate
/// * `angle` - The angle in radians
fn rotate_vec2(v: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

fn point_to_new_origin(point: Vec3, transform: &GlobalTransform) -> Vec2 {
    rotate_vec2(
        (point - transform.translation()).truncate(),
        -transform.rotation().to_euler(EulerRot::ZYX).0,
    )
}

/// Generates a curved "track turn" collider (an arc section)
///
/// # Arguments
/// * `radius`  - Inner radius of the arc
/// * `width`   - Thickness (distance between inner and outer edges)
/// * `angle`   - Total arc angle in radians (e.g., PI/2 for 90° turn)
/// * `height`  - Collider height/thickness
/// * `segments` - Number of convex segments for smoothness
pub fn arc_collider(radius: f32, width: f32, angle: f32, side: Side, height: f32) -> Collider {
    // Approximate the curved arc by composing `segments` small box colliders
    // placed along the arc. Each box is oriented so its long side follows
    // the tangent of the arc. The arc is generated so that angle=0 points
    // in the +Y direction and increases toward +X (so it matches the
    // TrackSegment transform conventions used elsewhere).

    let angle = angle.abs() * side.sign();
    let segments: usize =
        ((TRACK_CIRCLE_SEGMENTS_PER_PI as f32 * angle.abs() / PI).round() as usize).max(1);
    let delta = angle / segments as f32;
    let offset = match side {
        Side::Left => 0.0,
        Side::Right => PI,
    };

    // Collider::compound for bevy_rapier expects parts as (Vec3, Quat, Collider)
    let mut parts: Vec<(Vec3, Quat, Collider)> = Vec::with_capacity(segments);

    for i in 0..segments {
        // angular bounds for this piece
        let theta0 = (i as f32) * delta + offset;
        let theta1 = theta0 + delta;

        let r_in = radius - width / 2.0;
        let r_out = radius + width / 2.0;
        let hz = height * 0.5;

        // build 8 vertices for the prism: inner/out x theta0/theta1 x z-/+
        let mut pts: Vec<Vec3> = Vec::with_capacity(8);

        let p =
            |r: f32, theta: f32, z: f32| -> Vec3 { Vec3::new(r * theta.cos(), r * theta.sin(), z) };

        // inner theta0, z-
        pts.push(p(r_in, theta0, -hz));
        // inner theta0, z+
        pts.push(p(r_in, theta0, hz));
        // inner theta1, z-
        pts.push(p(r_in, theta1, -hz));
        // inner theta1, z+
        pts.push(p(r_in, theta1, hz));

        // outer theta0, z-
        pts.push(p(r_out, theta0, -hz));
        // outer theta0, z+
        pts.push(p(r_out, theta0, hz));
        // outer theta1, z-
        pts.push(p(r_out, theta1, -hz));
        // outer theta1, z+
        pts.push(p(r_out, theta1, hz));

        // place the convex hull at origin; positions are absolute in world-space
        // but Collider::compound wants local translations per part. We'll compute
        // the center of these points and use a local transform so vertices are
        // relative to that center.
        let mut center = Vec3::ZERO;
        for v in &pts {
            center += v;
        }
        center /= pts.len() as f32;

        let rel_pts_vec3: Vec<Vec3> = pts
            .into_iter()
            .map(|v| Vec3::new(v[0] - center.x, v[1] - center.y, v[2] - center.z))
            .collect();

        // Collider::convex_hull commonly accepts a slice of Vec3 and returns
        // an Option<Collider>. Use that if available, otherwise fall back to
        // a cuboid approximation.
        let convex = if let Some(c) = Collider::convex_hull(&rel_pts_vec3) {
            c
        } else {
            Collider::cuboid((r_out - r_in) * 0.5, (radius * delta) * 0.5, hz)
        };

        // The compound part takes translation and rotation; we keep identity
        // rotation because vertices already oriented in world XY plane, and
        // translate by the computed center.
        parts.push((center, Quat::IDENTITY, convex));
    }

    Collider::compound(parts)
}

#[derive(Debug, Clone, Copy)]
struct SegmentTransform {
    position: Vec2,
    direction: Angle,
}

impl SegmentTransform {
    pub fn new(position: Vec2, direction: Angle) -> Self {
        Self {
            position,
            direction,
        }
    }

    pub fn translate_in_direction(&self, translation: Vec2) -> Self {
        Self {
            position: self.position + rotate_vec2(translation, self.direction.to_radians()),
            direction: self.direction,
        }
    }

    pub fn rotate(&self, rotation: Angle) -> Self {
        Self {
            position: self.position,
            direction: Angle::from_radians(self.direction.to_radians() + rotation.to_radians()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StraightSegment {
    length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NinetyDegTurnSegment {
    line_half_length: f32,
    side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CyrcleTurnSegment {
    radius: f32,
    angle: Angle,
    side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
enum TrackSegment {
    Start,
    End,
    Straight(StraightSegment),
    NinetyDegTurn(NinetyDegTurnSegment),
    CyrcleTurn(CyrcleTurnSegment),
}

impl TrackSegment {
    pub fn start() -> Self {
        Self::Start
    }

    pub fn end() -> Self {
        Self::End
    }

    pub fn straight(length: f32) -> Self {
        Self::Straight(StraightSegment { length })
    }

    pub fn ninety_deg_turn(line_half_length: f32, side: Side) -> Self {
        Self::NinetyDegTurn(NinetyDegTurnSegment {
            line_half_length: line_half_length,
            side,
        })
    }

    pub fn cyrcle_turn(radius: f32, angle: Angle, side: Side) -> Self {
        Self::CyrcleTurn(CyrcleTurnSegment {
            radius,
            angle,
            side,
        })
    }

    pub fn collider(&self) -> Collider {
        match *self {
            TrackSegment::Start | TrackSegment::End => {
                Collider::cuboid(TRACK_HALF_WIDTH, TRACK_TIPS_LENGTH / 2.0, TRACK_HALF_HEIGHT)
            }
            TrackSegment::Straight(data) => {
                Collider::cuboid(TRACK_HALF_WIDTH, data.length / 2.0, TRACK_HALF_HEIGHT)
            }
            TrackSegment::NinetyDegTurn(data) => {
                let hl: f32 = (data.line_half_length + TRACK_HALF_WIDTH) / 2.0;
                let ht = (data.line_half_length - TRACK_HALF_WIDTH) / 2.0;
                // Collider::cuboid(hl, hl, TRACK_HALF_HEIGHT);
                Collider::compound(vec![
                    (
                        Vec3::ZERO,
                        Quat::IDENTITY,
                        Collider::cuboid(TRACK_HALF_WIDTH, hl, TRACK_HALF_HEIGHT),
                    ),
                    (
                        Vec3::new(ht * -data.side.sign(), ht, 0.0),
                        Quat::from_rotation_z(FRAC_PI_2),
                        Collider::cuboid(TRACK_HALF_WIDTH, hl, TRACK_HALF_HEIGHT),
                    ),
                ])
            }
            TrackSegment::CyrcleTurn(data) => arc_collider(
                data.radius,
                TRACK_HALF_WIDTH * 2.0,
                data.angle.to_radians(),
                data.side,
                TRACK_HALF_HEIGHT * 2.0,
            ),
        }
    }

    pub fn transform(&self, origin: SegmentTransform) -> Transform {
        let transform_origin = match *self {
            TrackSegment::Start | TrackSegment::End => {
                origin.translate_in_direction(Vec2::Y * TRACK_TIPS_LENGTH / 2.0)
            }
            TrackSegment::Straight(data) => {
                origin.translate_in_direction(Vec2::Y * data.length / 2.0)
            }
            TrackSegment::NinetyDegTurn(data) => origin
                .translate_in_direction(Vec2::Y * (data.line_half_length + TRACK_HALF_WIDTH) / 2.0),
            TrackSegment::CyrcleTurn(data) => {
                origin.translate_in_direction(Vec2::NEG_X * data.radius * data.side.sign())
            }
        };
        Transform::from_translation(transform_origin.position.extend(TRACK_Z_OFFSET)).with_rotation(
            Quat::from_rotation_z(transform_origin.direction.to_radians()),
        )
    }

    pub fn compute_next_origin(&self, origin: SegmentTransform) -> SegmentTransform {
        match *self {
            TrackSegment::Start | TrackSegment::End => {
                origin.translate_in_direction(Vec2::Y * TRACK_TIPS_LENGTH)
            }
            TrackSegment::Straight(data) => origin.translate_in_direction(Vec2::Y * data.length),
            TrackSegment::NinetyDegTurn(data) => origin
                .translate_in_direction(Vec2::new(
                    -data.line_half_length * data.side.sign(),
                    data.line_half_length,
                ))
                .rotate(Angle::from_degrees(90.0 * data.side.sign())),
            TrackSegment::CyrcleTurn(data) => origin
                .translate_in_direction(Vec2::new(
                    data.radius * (data.angle.to_radians().cos() - 1.0) * data.side.sign(),
                    data.radius * data.angle.to_radians().sin(),
                ))
                .rotate(Angle::from_radians(
                    data.angle.to_radians() * data.side.sign(),
                )),
        }
    }

    pub fn intersection_to_sensor_value(
        &self,
        intersection: Option<Vec3>,
        transform: &GlobalTransform,
    ) -> f32 {
        match intersection {
            Some(point) => {
                let local_point = point_to_new_origin(point, transform);

                match *self {
                    TrackSegment::Start | TrackSegment::End => line_reflection(local_point.x),
                    TrackSegment::Straight(_) => line_reflection(local_point.x),
                    TrackSegment::NinetyDegTurn(data) => {
                        let turn_y = (data.line_half_length - TRACK_HALF_WIDTH) / 2.0;
                        let dist_to_line =
                            if local_point.y < data.side.sign() * local_point.x + turn_y {
                                local_point.x
                            } else {
                                data.side.sign() * (local_point.y - turn_y)
                            };
                        line_reflection(dist_to_line)
                    }
                    TrackSegment::CyrcleTurn(data) => {
                        let dist_to_line = (local_point.length() - data.radius) * data.side.sign();
                        line_reflection(dist_to_line)
                    }
                }
            }
            None => 100.0,
        }
    }
}

#[derive(Resource)]
struct Track {
    origin: SegmentTransform,
    segments: Vec<TrackSegment>,
}

impl Track {
    pub fn new(segments: Vec<TrackSegment>) -> Self {
        Self {
            origin: SegmentTransform::new(Vec2::NEG_Y * TRACK_TIPS_LENGTH / 2.0, Angle::ZERO),
            segments,
        }
    }

    pub fn spawn_bundles(&self, mut commands: Commands) {
        let mut segment_origin = self.origin;

        for segment in &self.segments {
            commands.spawn((
                segment.collider(),
                *segment,
                segment.transform(segment_origin),
                RigidBody::Fixed,
                Friction {
                    coefficient: 0.5,
                    combine_rule: CoefficientCombineRule::Average,
                },
            ));
            segment_origin = segment.compute_next_origin(segment_origin);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WheelSide {
    Left,
    Right,
}

impl WheelSide {
    pub fn sign(&self) -> f32 {
        match self {
            WheelSide::Left => 1.0,
            WheelSide::Right => -1.0,
        }
    }
}

#[derive(Component)]
struct Motors {
    left_axle: Vec3,
    right_axle: Vec3,
    gear_ratio_num: u32,
    gear_ratio_den: u32,
}

#[derive(Component)]
struct Wheel {
    axle: Vec3,
    side: WheelSide,
}

#[derive(Resource)]
struct MotorsPwm {
    left_pwm: f32,
    right_pwm: f32,
}

impl MotorsPwm {
    pub fn new() -> Self {
        Self {
            left_pwm: 0.0,
            right_pwm: 0.0,
        }
    }

    pub fn pwm(&self, side: WheelSide) -> f32 {
        match side {
            WheelSide::Left => self.left_pwm,
            WheelSide::Right => self.right_pwm,
        }
    }
}

fn handle_motors_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut pwm: ResMut<MotorsPwm>) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let forward = if up {
        1.0
    } else if down {
        -1.0
    } else {
        0.0
    };
    let side = if left {
        -1.0
    } else if right {
        1.0
    } else {
        0.0
    };

    const MAX_PWM: f32 = 1.0;
    const USE_PWM: f32 = 1.0;

    pwm.left_pwm = (forward * USE_PWM + side * USE_PWM).clamp(-MAX_PWM, MAX_PWM);
    pwm.right_pwm = (forward * USE_PWM - side * USE_PWM).clamp(-MAX_PWM, MAX_PWM);
}

fn pwm_to_torque(
    pwm: f32,     // -1.0 .. 1.0
    ang_vel: f32, // rad/s
    gear_ratio_num: u32,
    gear_ratio_den: u32,
) -> f32 {
    // Model a simple DC motor: torque is proportional to PWM (drive) and
    // decreases linearly with angular velocity, reaching zero at the motor
    // no-load speed. This is a common, simple approximation of a brushed DC
    // motor's torque-speed curve.

    // Reference-ish values (Core DC Motor 6V 750 RPM by Jsumo or similar):
    // - no-load speed: ~750 RPM -> 750/60*2*pi = ~78.54 rad/s
    // - stall torque: small toy motor ~0.15..0.25 N·m; choose conservative 0.18
    // These are rough; tune to your robot size.
    const NO_LOAD_RPM: f32 = 2000.0;
    const NO_LOAD_OMEGA: f32 = NO_LOAD_RPM / 60.0 * std::f32::consts::TAU; // rad/s
    const STALL_TORQUE: f32 = 0.000005; // N·m at PWM = 1.0 and zero speed

    // Saturate PWM
    let pwm = pwm.clamp(-1.0, 1.0);

    // Gear ratio as floating point (motor rotations per wheel rotation).
    let gear_ratio = if gear_ratio_den == 0 {
        1.0
    } else {
        gear_ratio_num as f32 / gear_ratio_den as f32
    };

    // Motor angular velocity = wheel angular velocity * gear_ratio
    let motor_omega = ang_vel * gear_ratio.abs();

    // Motor torque magnitude scales with |pwm|
    let drive = pwm.abs();

    // Effective motor no-load speed for this drive (assume linear scaling with drive)
    let omega_noload_motor = NO_LOAD_OMEGA * drive;

    // If drive is zero, no torque.
    if drive <= 0.0 {
        return 0.0;
    }

    // Motor torque falls linearly with motor speed: Tm = T_stall * (1 - |omega_m|/omega_noload_m)
    let torque_ratio = if omega_noload_motor > 1e-6 {
        (1.0 - motor_omega.abs() / omega_noload_motor).max(0.0)
    } else {
        0.0
    };

    let motor_torque = STALL_TORQUE * drive * torque_ratio;

    // Wheel torque = motor torque * gear_ratio (torque amplified by gearbox)
    let wheel_torque = motor_torque * gear_ratio.abs();

    if pwm >= 0.0 {
        wheel_torque
    } else {
        -wheel_torque
    }
}

fn apply_motors_pwm(
    pwm: Res<MotorsPwm>,
    mut wheels_query: Query<(&Wheel, &Transform, &Velocity, &mut ExternalForce), Without<Motors>>,
    mut motors_query: Query<(&Motors, &Transform, &mut ExternalForce), Without<Wheel>>,
) {
    let mut body_torque = Vec3::ZERO;

    struct MotorsAxle {
        left: Vec3,
        right: Vec3,
    }
    impl MotorsAxle {
        fn new(left: Vec3, right: Vec3) -> Self {
            Self { left, right }
        }
        fn axle(&self, side: WheelSide) -> Vec3 {
            match side {
                WheelSide::Left => self.left,
                WheelSide::Right => self.right,
            }
        }
    }
    let (motors, motors_transform, mut motors_ext_force) = motors_query.single_mut().unwrap();
    let motors_axle = MotorsAxle::new(
        motors_transform.rotation * motors.left_axle,
        motors_transform.rotation * motors.right_axle,
    );

    for (wheel, transform, velocity, mut ext_impulse) in &mut wheels_query {
        let ang_vel = -velocity.angvel.dot(transform.rotation * wheel.axle.abs()); // rad/s
        let pwm_value = pwm.pwm(wheel.side);
        let torque = pwm_to_torque(
            pwm_value,
            ang_vel,
            motors.gear_ratio_num,
            motors.gear_ratio_den,
        );

        let wheel_axle = transform.rotation * wheel.axle.abs();
        ext_impulse.torque = -wheel_axle * torque;

        body_torque += motors_axle.axle(wheel.side) * torque;

        // println!(
        //     "Wheel {:?} torque {:.10} vel {:.2}",
        //     wheel.side, torque, ang_vel
        // );
    }

    motors_ext_force.torque = body_torque;
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BotPosition {
    OnTrack,
    Out,
    End,
}

#[derive(Component)]
struct BotPositionDetector {}

#[derive(Component)]
struct LineSensor {}

fn compute_sensor_readings(
    read_rapier_context: ReadRapierContext,
    sensors_query: Query<&GlobalTransform, With<LineSensor>>,
    track_segments_query: Query<(&TrackSegment, &GlobalTransform)>,
) {
    let rapier_context = read_rapier_context.single().unwrap();
    println!("--- Sensor readings ---");
    for sensor_tf in sensors_query {
        let origin = sensor_tf.translation();
        let dir = Vec3::NEG_Z; // sensor_tf.rotation().mul_vec3(Vec3::NEG_Z);
        let max_toi = 0.1;

        if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            origin,
            dir,
            max_toi,
            true,
            QueryFilter::default().predicate(&|entity| track_segments_query.get(entity).is_ok()),
        ) {
            // Sensor is over the track
            let point: Vec3 = intersection.point.into();
            let (track_segment, transform) = track_segments_query.get(entity).unwrap();
            println!(
                "Ray hit {} at {:.2} X {:.2}",
                entity,
                point_to_new_origin(point, transform),
                track_segment.intersection_to_sensor_value(Some(point), transform)
            );
        } else {
            println!("Ray hit nothing");
        }
    }
    println!("-----------------------");
}

fn compute_bot_position(
    read_rapier_context: ReadRapierContext,
    bot_query: Query<&GlobalTransform, With<BotPositionDetector>>,
    track_segments_query: Query<&TrackSegment>,
) {
    let rapier_context = read_rapier_context.single().unwrap();
    let origin = bot_query.single().unwrap().translation();
    let dir = Vec3::NEG_Z;
    let max_toi = 0.1;

    let bot_position = if let Some((entity, _)) = rapier_context.cast_ray_and_get_normal(
        origin,
        dir,
        max_toi,
        true,
        QueryFilter::default().predicate(&|entity| track_segments_query.get(entity).is_ok()),
    ) {
        // Bot is over the track
        // println!("Ray from {:.2} hit {} at {:.2}", origin, entity, point);

        if track_segments_query.get(entity).unwrap() == &TrackSegment::End {
            BotPosition::End
        } else {
            BotPosition::OnTrack
        }
    } else {
        // println!("Ray from {:.2} hit nothing", origin);
        BotPosition::Out
    };
    println!("bot position: {:?}", bot_position);
}

fn time_checker(time: Res<Time<Fixed>>) {
    println!("Fixed dt: {}", time.delta_secs());
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default().with_custom_initialization(
                RapierContextInitialization::InitializeDefaultRapierContext {
                    rapier_configuration: {
                        let mut config = RapierConfiguration::new(0.001);
                        config.gravity = Vec3::NEG_Z * 9.81;
                        config
                    },
                    integration_parameters: IntegrationParameters::default(),
                },
            ),
            DefaultEditorCamPlugins,
            RapierDebugRenderPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        // Set fixed timestep
        .insert_resource(Time::<Fixed>::from_hz(10000.0))
        // Add gravity to the physics simulation.
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        // Resource for motors pwm values.
        .insert_resource(MotorsPwm::new())
        // Define the track layout and spawn it.
        .insert_resource(Track::new(vec![
            TrackSegment::start(),
            TrackSegment::straight(2.0),
            TrackSegment::ninety_deg_turn(0.5, Side::Right),
            TrackSegment::cyrcle_turn(1.0, Angle::from_degrees(120.0), Side::Left),
            TrackSegment::ninety_deg_turn(1.0, Side::Left),
            TrackSegment::cyrcle_turn(2.0, Angle::from_degrees(60.0), Side::Right),
            TrackSegment::end(),
        ]))
        // Spawn text instructions for keybinds.
        .add_systems(
            RunFixedMainLoop,
            (handle_motors_input, apply_motors_pwm)
                .chain()
                .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
        )
        .add_systems(
            RunFixedMainLoop,
            (compute_sensor_readings, compute_bot_position, time_checker)
                .chain()
                .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
        )
        // Add systems for toggling the diagnostics UI and pausing and stepping the simulation.
        .add_systems(Startup, (setup_bot, setup_track, setup_ui).chain())
        .run();
}

// Robot configuration structure:

// pub struct Configuration {
//     /// Robot name
//     pub name: _rt::String,
//     /// Main color
//     pub color_main: Color,
//     /// Secondary color
//     pub color_secondary: Color,
//     /// Axle width from wheel to wheel (in mm, 100 to 200)
//     pub width_axle: f32,
//     /// Length from wheel axles to front (in mm, 100 to 300)
//     pub length_front: f32,
//     /// Length from wheel axles to back (in mm, 10 to 50)
//     pub length_back: f32,
//     /// Clearing from robot to ground at the robot back (in mm, from 1 to wheels radius)
//     pub clearing_back: f32,
//     /// Diameter of robot wheels (in mm, from 20 to 40)
//     pub wheel_diameter: f32,
//     /// Transmission gear ratio numerator (from 1 to 100)
//     pub gear_ratio_num: u32,
//     /// Transmission gear ratio denumerator (from 1 to 100)
//     pub gear_ratio_den: u32,
//     /// Spacing of line sensors (in mm, from 1 to 15)
//     pub front_sensors_spacing: f32,
//     /// Height of line sensors from the ground (in mm, from 1 to wheels radius)
//     pub front_sensors_height: f32,
// }

const BOT_BODY_LENGHT_MIN: f32 = 0.04;
const BOT_BODY_LENGHT_PERCENT_OF_TOTAL: f32 = 0.6;
const BOT_BODY_WIDTH: f32 = 0.09;
const BOT_BODY_HEIGHT: f32 = 0.02;

const BOT_BUMPER_DIAMETER: f32 = BOT_BODY_HEIGHT / 2.0;
const BOT_BUMPER_WIDTH: f32 = BOT_BODY_WIDTH / 2.0;

const BOT_SENSORS_DIAMETER: f32 = 0.001;

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
        (length_front - length_back) / 2.0,
        clearing_back + (BOT_BODY_HEIGHT * 0.5) + BOT_BUMPER_DIAMETER,
    );

    // Static body with motors
    let body = commands
        .spawn((
            Collider::cuboid(
                BOT_BODY_WIDTH * 0.5,
                (BOT_BODY_LENGHT_MIN
                    + BOT_BODY_LENGHT_PERCENT_OF_TOTAL * (length_front + length_back))
                    * 0.5,
                BOT_BODY_HEIGHT * 0.5,
            ),
            RigidBody::Dynamic,
            Friction {
                coefficient: 0.1,
                combine_rule: CoefficientCombineRule::Min,
            },
            ColliderMassProperties::Density(1.0),
            Transform::from_xyz(body_world.x, body_world.y, body_world.z),
            GlobalTransform::default(),
            Motors {
                left_axle: Vec3::X,
                right_axle: Vec3::NEG_X,
                gear_ratio_num,
                gear_ratio_den,
            },
            BotPositionDetector {},
            ExternalForce::default(),
            Velocity::zero(),
        ))
        .id();

    // Cylinder bumpers
    let front_bumper_world = Vec3::new(
        0.0,
        length_front - (BOT_BUMPER_WIDTH + BOT_SENSORS_DIAMETER) / 2.0,
        BOT_BUMPER_DIAMETER / 2.0,
    );
    let back_bumper_world = Vec3::new(0.0, -length_back, BOT_BUMPER_DIAMETER / 2.0 + clearing_back);

    for bumper_world in [front_bumper_world, back_bumper_world] {
        commands.spawn((
            Collider::capsule_x(BOT_BUMPER_WIDTH / 2.0, BOT_BUMPER_DIAMETER / 2.0),
            RigidBody::Dynamic,
            Friction {
                coefficient: 0.1,
                combine_rule: CoefficientCombineRule::Min,
            },
            Transform::from_xyz(bumper_world.x, bumper_world.y, bumper_world.z),
            ImpulseJoint::new(
                body,
                FixedJointBuilder::new()
                    .local_anchor1(bumper_world - body_world) // parent's local anchor
                    .local_anchor2(Vec3::ZERO),
            ),
        ));
    }

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
            ColliderMassProperties::Density(1.0),
            Wheel {
                axle: Vec3::NEG_X * side.sign(),
                side,
            },
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

        commands.spawn((
            Collider::ball(BOT_SENSORS_DIAMETER / 2.0),
            Transform::from_xyz(sensor_world.x, sensor_world.y, sensor_world.z),
            RigidBody::Dynamic,
            LineSensor {},
            ImpulseJoint::new(
                body,
                FixedJointBuilder::new()
                    .local_anchor1(sensor_world - body_world) // parent's local anchor
                    .local_anchor2(Vec3::ZERO),
            ),
        ));
    }
}

fn setup_track(mut commands: Commands, track: Res<Track>) {
    // Static floor
    commands.spawn((
        Collider::cuboid(FLOOR_SIZE / 2.0, FLOOR_SIZE / 2.0, FLOOR_HEIGHT / 2.0),
        RigidBody::Fixed,
        Friction::new(0.5),
        Transform::from_xyz(0.0, 0.0, -FLOOR_HEIGHT / 2.0),
    ));

    track.spawn_bundles(commands);
}

fn setup_ui(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        EditorCam {
            orbit_constraint: OrbitConstraint::Fixed {
                up: Vec3::Z,
                can_pass_tdc: false,
            },
            ..Default::default()
        },
        Transform::from_translation(Vec3::X * 0.5).looking_at(
            Vec3::X,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.1,
            },
        ),
    ));
}
