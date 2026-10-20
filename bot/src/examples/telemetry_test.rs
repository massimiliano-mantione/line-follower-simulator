use crate::async_api::*;
use crate::blocking_api::{
    get_motor_angles as get_motor_angles_immediate, get_steps_and_period_us,
};

const LINE_THRESHOLD: f32 = 150.0;

const PWM_MAX: i16 = 200;
const MAX_TIME: u32 = 50_000_000;
// A ring buffer of 20,000 samples of 40 bytes: the last 10 s of the race at 2 kHz.
// The flush costs 10 us per byte, so 800 KB take 8 s of simulated time, which still
// fits in the 60 s budget after a race that runs until MAX_TIME (50 s).
const TEL_CAPACITY: usize = 20_000;

const ERR_INTEGRAL_CLIP: f32 = 1_000_000.0;
const KP: f32 = 90.0;
const KD: f32 = 1_000_000.0;
const KI: f32 = 0.0;

const OUT_PWM_INNER: i16 = -600;

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
pub struct Pid {
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

    fn err(&self) -> f32 {
        self.err_mm
    }

    fn err_derivative(&self) -> f32 {
        self.err_derivative
    }

    fn dir(&self) -> u8 {
        match self.dir {
            Direction::Left => 0,
            Direction::Right => 1,
        }
    }

    fn out(&self) -> u8 {
        self.out as u8
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

            let inner_pwm = PWM_MAX - self.steering.abs().min(PWM_MAX as f32) as i16;
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

    pub fn log_vars(&self) {
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

pub async fn run(sensor_spacing_mm: f32) {
    let mut tel_buf = Vec::with_capacity(TEL_CAPACITY);
    let mut tel_oldest = 0;
    let csv_spec = TelBlock::csv_spec();

    wait_remote_enabled().await;

    console_log("started");

    let mut pid = Pid::new(sensor_spacing_mm);

    while remote_enabled() {
        let time = get_time_us();
        let (steps, _) = get_steps_and_period_us();

        pid.update_time();

        let vals = get_line_sensors().await;
        let (pwm_l, pwm_r) = pid.compute_pwm(vals);
        let (wl, wr) = get_motor_angles_immediate();
        let err = (pid.err() * 100.0) as i16; // hundredths of mm
        let der = (pid.err_derivative() * 1_000_000.0) as i16; // mm/s
        let dir = pid.dir();
        let out = pid.out();

        let sample = TelBlock::new(
            time,
            steps,
            vals,
            [wl, wr],
            [pwm_l, pwm_r],
            err,
            der,
            dir,
            out,
        );
        // Never grow the buffer mid-race: once it is full, overwrite the oldest sample.
        if tel_buf.len() < TEL_CAPACITY {
            tel_buf.push(sample);
        } else {
            tel_buf[tel_oldest] = sample;
            tel_oldest = (tel_oldest + 1) % TEL_CAPACITY;
        }

        //console_log(&format!("LINE {:?}", vals));
        //pid.log_vars();
        // sleep_for(500).await;

        set_motors_pwm(pwm_l, pwm_r);

        if get_time_us() > MAX_TIME {
            console_log("timeout");
            break;
        }
    }

    // Oldest sample first.
    tel_buf.rotate_left(tel_oldest);
    write_csv_file("telemetry", csv::transmute_buf(&tel_buf), &csv_spec);
}

#[repr(C)]
struct TelBlock {
    time: u32,
    steps: u32,
    vals: [u8; 16],
    w: [u16; 2],
    pwm: [i16; 2],
    e: i16,
    d: i16,
    dir: u8,
    out: u8,
    pad: u16,
}

impl TelBlock {
    fn new(
        time: u32,
        steps: u32,
        vals: [u8; 16],
        w: [u16; 2],
        pwm: [i16; 2],
        e: i16,
        d: i16,
        dir: u8,
        out: u8,
    ) -> Self {
        Self {
            time,
            steps,
            vals,
            w,
            pwm,
            e,
            d,
            dir,
            out,
            pad: 0,
        }
    }

    fn csv_spec() -> [csv::CsvColumn; 27] {
        [
            csv::col("time", csv::C_U32),
            csv::col("steps", csv::C_U32),
            csv::col("s1", csv::C_U8),
            csv::col("s2", csv::C_U8),
            csv::col("s3", csv::C_U8),
            csv::col("s4", csv::C_U8),
            csv::col("s5", csv::C_U8),
            csv::col("s6", csv::C_U8),
            csv::col("s7", csv::C_U8),
            csv::col("s8", csv::C_U8),
            csv::col("s9", csv::C_U8),
            csv::col("s10", csv::C_U8),
            csv::col("s11", csv::C_U8),
            csv::col("s12", csv::C_U8),
            csv::col("s13", csv::C_U8),
            csv::col("s14", csv::C_U8),
            csv::col("s15", csv::C_U8),
            csv::col("s16", csv::C_U8),
            csv::col("wl", csv::C_U16),
            csv::col("wr", csv::C_U16),
            csv::col("pwm_l", csv::C_I16),
            csv::col("pwm_r", csv::C_I16),
            csv::col("err", csv::C_I16),
            csv::col("der", csv::C_I16),
            csv::col("dir", csv::named([csv::nv("left", 0), csv::nv("right", 1)])),
            csv::col("out", csv::named([csv::nv("in", 0), csv::nv("out", 1)])),
            csv::col(".", csv::PAD_16),
        ]
    }
}
