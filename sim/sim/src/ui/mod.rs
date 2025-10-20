use bevy::prelude::*;
use bevy_editor_cam::prelude::{EditorCam, OrbitConstraint};
use execution_data::{MotorDriversDutyCycles, PWM_MAX};

fn handle_motors_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut pwm: ResMut<MotorDriversDutyCycles>,
) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let forward = if up {
        1
    } else if down {
        -1
    } else {
        0
    };
    let side = if left {
        -1
    } else if right {
        1
    } else {
        0
    };

    const USE_PWM: i16 = PWM_MAX / 2;

    pwm.left = (forward * USE_PWM + side * USE_PWM).clamp(-PWM_MAX, PWM_MAX);
    pwm.right = (forward * USE_PWM - side * USE_PWM).clamp(-PWM_MAX, PWM_MAX);
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

pub fn add_ui_setup(app: &mut App) {
    app.add_systems(Startup, setup_ui)
        .add_systems(Update, handle_motors_input)
        // Background color
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)));
}
