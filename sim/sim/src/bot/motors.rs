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
    // EXERCISE 4.4 (do 4.3 first): model a brushed DC motor.
    //
    // Returning 0.0 is why the robot does not move. A linear `k * pwm` is a fine
    // first approximation - get that working, then make it realistic:
    //
    // A brushed DC motor's torque falls linearly with speed, from stall torque at
    // zero speed to zero at its no-load speed:
    //
    //     T = STALL_TORQUE * |pwm| * (1 - |w_motor| / (NO_LOAD_OMEGA * |pwm|))
    //
    // Reference values for a small toy motor (Core DC Motor 6V, or similar):
    //     NO_LOAD_RPM   40000.0     rpm, at full drive
    //     STALL_TORQUE  0.001       N.m, at pwm = 1.0 and zero speed
    //
    // Then the gearbox. `gear_ratio` here is num/den (default 1/20 = 0.05), meaning
    // wheel revolutions per motor revolution - so the motor spins *faster* than the
    // wheel and the gearbox *amplifies* torque. Watch the direction of both
    // conversions, and beware a zero denominator.
    //
    // Finally: PWM is bounded by PWM_MIN..PWM_MAX, and the sign of the result must
    // follow the sign of the drive.
    0.0
}

#[allow(unused_variables, unused_mut)]
fn apply_motors_pwm(
    pwm: Res<MotorDriversDutyCycles>,
    data: Res<ExecutionData>,
    mut wheels_query: Query<(&Wheel, &Transform, &Velocity, &mut ExternalForce), Without<Motors>>,
    mut motors_query: Query<(&Motors, &mut ExternalForce), Without<Wheel>>,
) {
    // EXERCISE 4.3: turn duty cycles into torque on the wheels.
    //
    // Four things:
    //
    //  1. Do nothing unless the race is running. `data.activity_data` knows. Without
    //     this, robots crawl away during the one-second countdown.
    //
    //  2. For each wheel, work out its angular velocity about its own axle. The
    //     wheel's `axle` is in body-local space, so rotate it by the wheel's
    //     transform first; then project the angular velocity onto it. Pass that and
    //     the duty cycle for this side to `pwm_to_torque` (4.4).
    //
    //  3. Apply the torque along the wheel's *rotated* axle - using a world axis
    //     instead drives fine in a straight line and diverges hilariously in turns.
    //
    //  4. Newton's third law: the motor pushes against its own mount, so the
    //     chassis gets the *reaction* torque, summed over both wheels. Omit it and
    //     the robot never squats under acceleration - which also means the sensor
    //     bar never lifts, which means lesson 05's height attenuation never fires.
    //
    // See: GetBySide::get_by_side, ExternalForce::torque,
    //      ActivityData::is_active_now
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
