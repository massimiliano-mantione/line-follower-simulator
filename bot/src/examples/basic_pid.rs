use crate::blocking_api::{
    console_log, get_line_sensors, get_time_us, remote_enabled, set_motors_pwm, wait_remote_enabled,
};

const PWM_MAX: i16 = 500;
const MAX_TIME: u32 = 50_000_000;

const ERR_INTEGRAL_CLIP: f32 = 1_000_000.0;
const KP: f32 = 0.5;
const KD: f32 = 0.0;
const KI: f32 = 0.0;

#[derive(Default)]
struct Pid {
    sensor_spacing_mm: f32,
    time_us: u32,
    last_time_us: u32,
    dt_us: f32,
    err_mm: f32,
    last_err: f32,
    err_derivative: f32,
    err_integral: f32,
    steering: f32,
}

impl Pid {
    fn new(sensor_spacing_mm: f32) -> Self {
        Self {
            sensor_spacing_mm,
            time_us: get_time_us(),
            err_integral: 0.0,
            ..Default::default()
        }
    }

    fn update_time(&mut self) {
        self.last_time_us = self.time_us;
        self.time_us = get_time_us();
        self.dt_us = (self.time_us - self.last_time_us) as f32;
    }

    fn compute_pwm(&mut self, vals: [u8; 16]) -> (i16, i16) {
        let err_mm_num: f32 = vals
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                let x = (i as f32 - 7.5) * self.sensor_spacing_mm;
                x * v as f32
            })
            .sum();
        let err_mm_den: f32 = vals.into_iter().map(|v| v as f32).sum();
        self.err_mm = -err_mm_num / err_mm_den;

        self.err_derivative = if self.dt_us <= 0.0 {
            0.0
        } else {
            (self.err_mm - self.last_err) / self.dt_us
        };

        self.err_integral += self.err_mm * self.dt_us as f32;
        self.err_integral = self
            .err_integral
            .max(-ERR_INTEGRAL_CLIP)
            .min(ERR_INTEGRAL_CLIP);

        self.steering = KP * self.err_mm + KD * self.err_derivative + KI * self.err_integral;

        let inner_pwm = PWM_MAX - self.steering.abs() as i16;
        let outer_pwm = PWM_MAX;

        let (pwm_left, pwm_right) = if self.steering < 0.0 {
            (inner_pwm, outer_pwm)
        } else {
            (outer_pwm, inner_pwm)
        };

        // lastly:
        self.last_err = self.err_mm;

        (pwm_left, pwm_right)
    }

    fn log_vars(&self) {
        console_log(&format!(
            "STEER < {:.2} > ERR {:.2} DER {:.10} INT {:.0}",
            self.steering, self.err_mm, self.err_derivative, self.err_integral
        ));
    }
}

pub fn basic_pid_run(sensor_spacing_mm: f32) {
    wait_remote_enabled();

    console_log("started");

    let mut pid = Pid::new(sensor_spacing_mm);

    while remote_enabled() {
        pid.update_time();

        let vals = get_line_sensors();
        let (pwm_l, pwm_r) = pid.compute_pwm(vals);

        pid.log_vars();

        set_motors_pwm(pwm_l, pwm_r);

        if get_time_us() > MAX_TIME {
            console_log("timeout");
            break;
        }
    }
}
