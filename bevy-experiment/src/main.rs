use std::rc::Rc;
use std::sync::Arc;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::world;
use bevy::prelude::*;
use bevy_editor_cam::DefaultEditorCamPlugins;
use bevy_editor_cam::prelude::{EditorCam, OrbitConstraint};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::IntegrationParameters;

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

#[derive(Resource)]
struct MotorsTorque {
    left_torque: f32,
    right_torque: f32,
}

impl MotorsTorque {
    pub fn new() -> Self {
        Self {
            left_torque: 0.0,
            right_torque: 0.0,
        }
    }

    pub fn torque(&self, side: WheelSide) -> f32 {
        match side {
            WheelSide::Left => self.left_torque,
            WheelSide::Right => self.right_torque,
        }
    }
}

#[derive(Component)]
struct Motors {
    left_axle: Vec3,
    right_axle: Vec3,
}

#[derive(Component)]
struct Wheel {
    axle: Vec3,
    side: WheelSide,
}

fn handle_motors_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut torque: ResMut<MotorsTorque>,
) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let forward = if up {
        -1.0
    } else if down {
        1.0
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

    const FORWARD_TORQUE: f32 = 10.1;
    const SIDE_TORQUE: f32 = 10.1;

    torque.left_torque = forward * FORWARD_TORQUE + side * SIDE_TORQUE;
    torque.right_torque = forward * FORWARD_TORQUE - side * SIDE_TORQUE;
}

fn set_wheel_torque(
    torque: Res<MotorsTorque>,
    mut query: Query<(&Wheel, &Transform, &mut ExternalForce)>,
) {
    for (wheel, transform, mut ext_impulse) in &mut query {
        let torque = torque.torque(wheel.side) * wheel.side.sign();
        let wheel_axle = transform.rotation * wheel.axle;
        ext_impulse.torque = wheel_axle * torque;
    }
}

fn set_motors_torque(
    torque: Res<MotorsTorque>,
    mut query: Query<(&Motors, &Transform, &mut ExternalForce)>,
) {
    for (motors, transform, mut ext_torque) in &mut query {
        let left_torque = torque.left_torque * WheelSide::Left.sign() * -1.0;
        let left_axle = transform.rotation * motors.left_axle;
        let right_torque = torque.right_torque * WheelSide::Right.sign() * -1.0;
        let right_axle = transform.rotation * motors.right_axle;
        ext_torque.torque = (left_axle * left_torque) + (right_axle * right_torque);
    }
}

fn ray_cast_example(
    read_rapier_context: ReadRapierContext,
    body_query: Query<(&Motors, &GlobalTransform)>,
    gt_query: Query<&GlobalTransform>,
) {
    for (_, body_tf) in body_query {
        let origin = body_tf.translation();
        let dir = body_tf.rotation().mul_vec3(Vec3::X);
        let max_toi = 10.0;

        let rapier_context = read_rapier_context.single().unwrap();

        if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            origin,
            dir,
            max_toi,
            true,
            QueryFilter::default(),
        ) {
            let point: Vec3 = intersection.point.into();

            let gt = gt_query.get(entity).unwrap();

            println!(
                "Ray from {:?} hit {:?} at {} gt {:?}",
                origin, entity, point, gt
            );
        } else {
            println!("Ray from {:?} hit nothing", origin);
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default().with_custom_initialization(
                RapierContextInitialization::InitializeDefaultRapierContext {
                    rapier_configuration: {
                        let mut config = RapierConfiguration::new(1f32);
                        config.gravity = Vec3::NEG_Z * 9.81;
                        config
                    },
                    integration_parameters: IntegrationParameters {
                        length_unit: 1f32,
                        ..default()
                    },
                },
            ),
            DefaultEditorCamPlugins,
            RapierDebugRenderPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        // Add gravity to the physics simulation.
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        // Resource for motors torque values.
        .insert_resource(MotorsTorque::new())
        // Spawn text instructions for keybinds.
        .add_systems(
            RunFixedMainLoop,
            (handle_motors_input, set_wheel_torque, set_motors_torque)
                .chain()
                .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
        )
        .add_systems(
            RunFixedMainLoop,
            ray_cast_example.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
        )
        // Add systems for toggling the diagnostics UI and pausing and stepping the simulation.
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Static floor
    let _floor = commands
        .spawn((
            Collider::cuboid(0.5, 0.5, 0.5),
            RigidBody::Fixed,
            Friction::new(0.5),
            Transform::from_xyz(0.0, 0.0, -4.0).with_scale(Vec3::new(50.0, 50.0, 0.1)),
        ))
        .id();

    // Static car body with motors
    let car_body = commands
        .spawn((
            Collider::cuboid(0.5, 0.5, 0.5),
            RigidBody::Dynamic,
            Friction {
                coefficient: 0.5,
                combine_rule: CoefficientCombineRule::Min,
            },
            ColliderMassProperties::Density(0.5),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(3.0, 0.9, 0.5)),
            GlobalTransform::default(),
            Motors {
                left_axle: Vec3::Y,
                right_axle: Vec3::NEG_Y,
            },
            ExternalForce::default(),
            Velocity::zero(),
        ))
        .id();

    let wheel_joints_x = 1.5;

    let left_wheel_joint: RevoluteJointBuilder = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(Vec3::new(wheel_joints_x, 0.5, 0.0))
        .local_anchor2(Vec3::new(0.0, -0.5, 0.0));
    let _left_wheel = commands
        .spawn((
            Collider::cylinder(0.5, 0.5),
            Transform::from_xyz(wheel_joints_x, 1.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Friction {
                coefficient: 0.95,
                combine_rule: CoefficientCombineRule::Max,
            },
            ColliderMassProperties::Density(0.5),
            Wheel {
                axle: Vec3::Y,
                side: WheelSide::Left,
            },
            ExternalForce::default(),
            ImpulseJoint::new(car_body, left_wheel_joint),
        ))
        .id();

    let right_wheel_joint = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(Vec3::new(wheel_joints_x, -0.5, 0.0))
        .local_anchor2(Vec3::new(0.0, 0.5, 0.0));
    let _right_wheel = commands
        .spawn((
            Collider::cylinder(0.5, 0.5),
            Transform::from_xyz(wheel_joints_x, -1.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Friction {
                coefficient: 0.95,
                combine_rule: CoefficientCombineRule::Max,
            },
            ColliderMassProperties::Density(0.5),
            Wheel {
                axle: Vec3::NEG_Y,
                side: WheelSide::Right,
            },
            ExternalForce::default(),
            ImpulseJoint::new(car_body, right_wheel_joint),
        ))
        .id();

    // // Connect left wheel
    // commands.spawn(
    //     RevoluteJoint::new(car_body, left_wheel)
    //         .with_aligned_axis(Vector::Y)
    //         .with_local_anchor_1(Vector::Y * 1.0 + Vector::X * -0.5),
    // );

    // // Connect right wheel
    // commands.spawn(
    //     RevoluteJoint::new(car_body, right_wheel)
    //         .with_aligned_axis(Vector::Y)
    //         .with_local_anchor_1(Vector::Y * -1.0 + Vector::X * -0.5),
    // );

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 2000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_at(Vec3::new(-1.0, -1.5, -2.5), Vec3::Z),
    ));

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
        Transform::from_translation(Vec3::Z * 10.0).looking_at(
            Vec3::Y,
            Vec3 {
                x: 0.0,
                y: 5.0,
                z: 10.0,
            },
        ),
    ));
}
