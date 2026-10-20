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
// Used by EXERCISE 9.2 and 9.3 once they are implemented.
#[allow(dead_code)]
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

#[allow(unused_variables)]
pub async fn run(sensor_spacing_mm: f32) {
    // EXERCISE 9.2: allocate the sample buffer here, before the race starts.
    //
    // Use `Vec::with_capacity(TEL_CAPACITY)` and never let it grow mid-race: an
    // allocation in a control loop is an unbounded latency spike. Make it a ring
    // buffer: once it is full, each new sample overwrites the oldest one, so you keep
    // the last seconds of the race, which are the ones you want when the robot leaves
    // the track. That takes an index of the oldest sample, too.
    //
    // Get the column spec once, here, too.

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

        // EXERCISE 9.3: record one sample.
        //
        // Everything worth recording is already in scope: `time`, `steps`, the 16
        // sensor values in `vals`, the wheel angles `wl`/`wr`, the outputs `pwm_l`/
        // `pwm_r`, the line error `err` and its derivative `der`, and what the robot
        // concluded about the track: `dir` (0 left, 1 right) and `out` (1 when the
        // line is lost). Build a `TelBlock`, push it while the buffer has room, and
        // overwrite the oldest sample once it is full.

        //console_log(&format!("LINE {:?}", vals));
        //pid.log_vars();
        // sleep_for(500).await;

        set_motors_pwm(pwm_l, pwm_r);

        if get_time_us() > MAX_TIME {
            console_log("timeout");
            break;
        }
    }

    // EXERCISE 9.4: flush the buffer, now that the race is over.
    //
    // If the buffer wrapped around, its oldest sample is not at the start:
    // `rotate_left` puts it there, so the CSV comes out in time order.
    //
    // `write_csv_file(name, bytes, spec)` takes the raw bytes plus the column spec,
    // and the host writes both a .bin and a .csv. `csv::transmute_buf` (provided)
    // reinterprets your `Vec<TelBlock>` as bytes.
    //
    // Note *where* this happens: after the loop. The host charges 10 us per byte, so
    // flushing 800 KB costs 8 s of simulated time - but it happens once the race has
    // ended, where it cannot hurt. That cost is also what limits the buffer size: the
    // whole run has a 60 s budget, flush included. Doing it inside the loop is what
    // this whole design exists to avoid.
}

// EXERCISE 9.1: design the telemetry sample.
//
// We want to see *every decision the robot took*: a timestamp, the step count, all
// 16 sensor values, both wheel angles, both PWM outputs, the line error and its
// derivative, and what the robot concluded about the track: `dir` and `out`, one
// byte each. About 40 bytes.
//
// Streaming that live is not an option. 40 bytes at 2 kHz is 80 KB/s over a
// Bluetooth serial link - and remember lesson 02: transmitting *costs simulated
// time*. `write_line` charges 100 us per character, so a 60-character log line is 6
// ms, which at 2 kHz is twelve missed control cycles. Logging inside the control loop
// destroys the thing you are trying to measure. So we buffer in RAM and flush once.
//
// Two things to write:
//
//  - the packed sample. `#[repr(C)]`, fixed size. Give it a field per quantity
//    above.
//
//  - `csv_spec()`, a parallel array describing the byte layout, built with
//    `csv::col(name, kind)` and the `csv::C_U32` / `C_U16` / `C_I16` / `C_U8`
//    constants. For `dir` and `out`, `csv::named` maps each byte value to a name,
//    so the CSV says `left`/`right` and `in`/`out` rather than `0`/`1`.
//
// The subtle part, and the instructive one: the host walks your bytes using *only*
// the spec, so **the spec must account for every byte the compiler inserted**,
// including padding added for alignment. That is what `csv::PAD_8` / `PAD_16` are
// for - and an explicit padding field in the struct makes it visible rather than
// implicit. If the spec is off, the CSV shears: every row offset by a couple of
// bytes, every column full of plausible nonsense.
//
// Two arrays that must agree and that the compiler cannot check for you. Worth
// asking yourself how you would make that safe - a derive macro, or a const
// assertion on `size_of::<TelBlock>()`.
#[repr(C)]
#[allow(dead_code)]
struct TelBlock {
    time: u32,
}

#[allow(dead_code)]
impl TelBlock {
    fn csv_spec() -> Vec<csv::CsvColumn> {
        todo!("describe the byte layout of TelBlock")
    }
}
