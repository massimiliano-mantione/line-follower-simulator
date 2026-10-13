use crate::blocking_api::{
    console_log, get_line_sensors, get_time_us, remote_enabled, set_motors_pwm, sleep_for,
    wait_remote_enabled,
};

const LINE: u8 = 80;
const PWM_MAX: i16 = 200;
const PWM_MIN: i16 = -250;
const MAX_TIME: u32 = 50_000_000;

pub fn run() {
    wait_remote_enabled();

    console_log("started");

    while remote_enabled() {
        let vals = get_line_sensors();

        let left_v = vals[0];
        let right_v = vals[15];

        let left = left_v < LINE;
        let right = right_v < LINE;

        console_log(&format!("LINE {:?}", vals));

        // console_log(&format!(
        //     " - val {} {} [{} {}] line {} {}",
        //     left_v, right_v, vals[0], vals[15], left, right
        // ));

        let (pwm_l, pwm_r) = match (left, right) {
            (true, _) => (PWM_MIN, PWM_MAX),
            (_, true) => (PWM_MAX, PWM_MIN),
            _ => (PWM_MAX, PWM_MAX),
        };
        set_motors_pwm(pwm_l, pwm_r);

        sleep_for(1000);
        if get_time_us() > MAX_TIME {
            console_log("timeout");
            break;
        }
    }
}
