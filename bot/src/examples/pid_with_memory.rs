use crate::blocking_api::*;

const LINE_THRESHOLD: f32 = 150.0;

const PWM_MAX: i16 = 400;
const MAX_TIME: u32 = 50_000_000;

const ERR_INTEGRAL_CLIP: f32 = 1_000_000.0;
const KP: f32 = 90.0;
const KD: f32 = 2_000_000.0;
const KI: f32 = 0.0003;

const OUT_PWM_INNER: i16 = -300;

fn calibrated_line_value(raw: u8) -> f32 {
    (255 - raw) as f32
}

#[derive(Default, Debug)]
enum Direction {
    #[default]
    Left,
    Right,
}

#[derive(Default)]
struct Pid {
    sensor_spacing_mm: f32,
    time_us: u32,
    last_time_us: u32,
    dt_us: f32,
    dir: Direction,
    out: bool,
    err_mm: f32,
    last_err: f32,
    err_derivative: f32,
    err_integral: f32,
    steering: f32,
    pwm_left: i16,
    pwm_right: i16,
}

impl Pid {
    fn new(sensor_spacing_mm: f32) -> Self {
        Self {
            sensor_spacing_mm,
            time_us: get_time_us(),
            err_integral: 0.0,
            out: false,
            ..Default::default()
        }
    }

    fn update_time(&mut self) {
        self.last_time_us = self.time_us;
        self.time_us = get_time_us();
        self.dt_us = (self.time_us - self.last_time_us) as f32;
    }

    fn compute_pwm(&mut self, vals: [u8; 16]) -> (i16, i16) {
        self.out = !vals
            .into_iter()
            .map(calibrated_line_value)
            .map(|v| v > LINE_THRESHOLD)
            .reduce(|acc, v| acc | v)
            .unwrap();

        if self.out {
            (self.pwm_left, self.pwm_right) = match self.dir {
                Direction::Left => (OUT_PWM_INNER, PWM_MAX),
                Direction::Right => (PWM_MAX, OUT_PWM_INNER),
            }
        } else {
            let err_mm_num: f32 = vals
                .into_iter()
                .map(calibrated_line_value)
                .enumerate()
                .map(|(i, v)| {
                    let x = (i as f32 - 7.5) * self.sensor_spacing_mm;
                    x * v as f32
                })
                .sum();
            let err_mm_den: f32 = vals.into_iter().map(|v| v as f32).sum();
            self.err_mm = err_mm_num / err_mm_den;

            self.dir = if self.err_mm < 0.0 {
                Direction::Left
            } else {
                Direction::Right
            };

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

            (self.pwm_left, self.pwm_right) = if self.steering < 0.0 {
                (inner_pwm, outer_pwm)
            } else {
                (outer_pwm, inner_pwm)
            };

            // lastly:
            self.last_err = self.err_mm;
        };

        (self.pwm_left, self.pwm_right)
    }

    #[allow(unused)]
    fn log_vars(&self) {
        console_log(&format!(
            "pwm [ {} {} ] STEER < {:.2} > OUT {} | ERR {:.2} DER {:.10} INT {:.0}",
            self.pwm_left,
            self.pwm_right,
            self.steering,
            self.out,
            self.err_mm,
            self.err_derivative,
            self.err_integral
        ));
    }
}

pub fn run(sensor_spacing_mm: f32) {
    wait_remote_enabled();

    console_log("started");

    let mut pid = Pid::new(sensor_spacing_mm);

    while remote_enabled() {
        pid.update_time();

        let vals = get_line_sensors();
        let (pwm_l, pwm_r) = pid.compute_pwm(vals);

        
        set_motors_pwm(pwm_l, pwm_r);
        
        // console_log(&format!("LINE {:?}", vals));
        // pid.log_vars();
        sleep_for(500);

        if get_time_us() > MAX_TIME {
            console_log("timeout");
            break;
        }
    }
}
