use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
#[allow(unused_imports)]
use execution_data::{ExecutionData, MotorDriversDutyCycles, PWM_MAX, PWM_MIN};

// Both are used by EXERCISE 4.3 and 4.4 once they are implemented.
#[allow(unused_imports)]
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

// The gear ratio is read by EXERCISE 4.3 once implemented.
#[allow(dead_code)]
#[derive(Component)]
pub struct Motors {
    gear_ratio_num: u32,
    gear_ratio_den: u32,
}

impl Motors {
    pub fn new(gear_ratio_num: u32, gear_ratio_den: u32) -> Self {
        Self {
            gear_ratio_num,
            gear_ratio_den,
        }
    }
}

// Called by EXERCISE 4.3 once implemented.
#[allow(dead_code, unused_variables)]
fn pwm_to_torque(
    pwm: i16,     // -1000 .. 1000
    ang_vel: f32, // rad/s
    gear_ratio_num: u32,
    gear_ratio_den: u32,
) -> f32 {
    0.0
}

#[allow(unused_variables, unused_mut)]
fn apply_motors_pwm(
    pwm: Res<MotorDriversDutyCycles>,
    data: Res<ExecutionData>,
    mut wheels_query: Query<(&Wheel, &Transform, &Velocity, &mut ExternalForce), Without<Motors>>,
    mut motors_query: Query<(&Motors, &mut ExternalForce), Without<Wheel>>,
) {
}

pub struct MotorsModelPlugin;

impl Plugin for MotorsModelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MotorDriversDutyCycles::default())
            .add_systems(
                RunFixedMainLoop,
                (apply_motors_pwm)
                    .chain()
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            );
    }
}
