use crate::blocking_api::{
    console_log, get_line_sensors, get_time_us, remote_enabled, set_motors_pwm, sleep_for,
    wait_remote_enabled,
};

const LINE: u8 = 80;
const PWM_MAX: i16 = 300;
const PWM_MIN: i16 = -100;
const MAX_TIME: u32 = 10_000_000;

fn calibrated_line_value(raw: u8) -> f32 {
    (255 - raw) as f32
}

pub fn run() {
    wait_remote_enabled();

    console_log("started");

    while remote_enabled() {
        let vals = get_line_sensors();

        let left_v = vals[0];
        let right_v = vals[15];

        let left = left_v < LINE;
        let right = right_v < LINE;

        // Compute error
        let err_mm_num: f32 = vals
            .into_iter()
            .map(calibrated_line_value)
            .enumerate()
            .map(|(i, v)| {
                let x = (i as f32 - 7.5) * 4.0;
                x * v
            })
            .sum();
        let err_mm_den: f32 = vals.into_iter().map(|v| v as f32).sum();
        let err_mm = err_mm_num / err_mm_den;

        console_log(&format!("ERR {} LINE {:?}", err_mm, vals));

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
