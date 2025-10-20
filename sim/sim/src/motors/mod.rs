use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use execution_data::{MotorDriversDutyCycles, PWM_MAX, PWM_MIN};

use crate::utils::{GetBySide, Side};

#[derive(Component)]
pub struct Wheel {
    pub axle: Vec3,
    pub side: Side,
}

impl Wheel {
    pub fn new(axle: Vec3, side: Side) -> Self {
        Self { axle, side }
    }
}

#[derive(Component)]
pub struct Motors {
    left_axle: Vec3,
    right_axle: Vec3,
    gear_ratio_num: u32,
    gear_ratio_den: u32,
}

impl Motors {
    pub fn new(
        left_axle: Vec3,
        right_axle: Vec3,
        gear_ratio_num: u32,
        gear_ratio_den: u32,
    ) -> Self {
        Self {
            left_axle,
            right_axle,
            gear_ratio_num,
            gear_ratio_den,
        }
    }
}

fn pwm_to_torque(
    pwm: i16,     // -1000 .. 1000
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
    const STALL_TORQUE: f32 = 0.02; // N·m at PWM = 1.0 and zero speed

    // Saturate PWM
    let pwm = (pwm.clamp(PWM_MIN, PWM_MAX) as f32) / (PWM_MAX as f32);

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
    pwm: Res<MotorDriversDutyCycles>,
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
        fn axle(&self, side: Side) -> Vec3 {
            match side {
                Side::Left => self.left,
                Side::Right => self.right,
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
        let pwm_value = pwm.get_by_side(wheel.side);
        let torque = pwm_to_torque(
            pwm_value,
            ang_vel,
            motors.gear_ratio_num,
            motors.gear_ratio_den,
        );

        let wheel_axle = transform.rotation * wheel.axle.abs();
        ext_impulse.torque = -wheel_axle * torque;

        body_torque += motors_axle.axle(wheel.side).abs() * torque;

        // println!(
        //     "Wheel {:?} torque {:.10} vel {:.2}",
        //     wheel.side, torque, ang_vel
        // );
    }

    // #FIXME: only works along positive Y axis, not negative...
    motors_ext_force.torque = body_torque;
}

pub fn add_motors(app: &mut App) {
    app.insert_resource(MotorDriversDutyCycles::default())
        .add_systems(
            RunFixedMainLoop,
            (apply_motors_pwm)
                .chain()
                .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
        );
}
