//! Unit tests for the light-sensor perceptual model.
//!
//! These cover the two things a naive implementation gets wrong: the *edge* of the
//! line is a smooth ramp whose width equals the sensor's height above the ground
//! (a 45-degree illumination cone), and *lifting* the sensor collapses contrast
//! toward ambient light rather than merely blurring the reading.

use super::*;
use crate::utils::{Angle, Side};

/// Half the line width, in metres. The line is 20 mm wide.
const HALF_LINE: f32 = 0.01;
/// Sensor height at which the model assumes no attenuation at all.
const Z_GROUND: f32 = 0.002;
/// Sensor height at which the sensor is effectively blind.
const Z_BLIND: f32 = 0.02;
/// Reading a blind sensor converges on: it sees only ambient light.
const AMBIENT: f32 = 50.0;

fn assert_close(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 0.01,
        "{}: expected {:.4}, got {:.4}",
        what,
        expected,
        actual
    );
}

/// What a sensor at height `z` reads over pure white floor.
fn white_at(z: f32) -> f32 {
    line_reflection_attenuation(100.0, z)
}

/// What a sensor at height `z` reads over the middle of the line.
fn black_at(z: f32) -> f32 {
    line_reflection_attenuation(0.0, z)
}

#[test]
fn full_contrast_near_the_ground() {
    assert_close(line_reflection(0.0, Z_GROUND), 0.0, "centred on the line");
    assert_close(line_reflection(0.05, Z_GROUND), 100.0, "well clear of the line");
}

#[test]
fn lifting_the_sensor_collapses_contrast() {
    // At 20 mm the reflected signal is gone and only ambient light remains, so
    // black and white become indistinguishable.
    assert_close(line_reflection(0.0, Z_BLIND), AMBIENT, "black when blind");
    assert_close(line_reflection(0.05, Z_BLIND), AMBIENT, "white when blind");
}

#[test]
fn contrast_shrinks_monotonically_with_height() {
    let mut last = f32::INFINITY;
    for i in 0..=10 {
        let z = Z_GROUND + (Z_BLIND - Z_GROUND) * (i as f32 / 10.0);
        let contrast = white_at(z) - black_at(z);
        assert!(
            contrast <= last + 1e-4,
            "contrast grew with height at z={}: {} after {}",
            z,
            contrast,
            last
        );
        assert!(contrast >= -1e-4, "contrast went negative at z={}", z);
        last = contrast;
    }
    assert_close(last, 0.0, "no contrast left at maximum height");
}

#[test]
fn the_line_edge_is_a_smooth_ramp() {
    let z = 0.004;
    // Straddling the edge must give an intermediate value: this gradient is the
    // signal a PID steers on. A step function would only permit bang-bang control.
    let mid = line_reflection(HALF_LINE + z * 0.5, z);
    assert!(
        mid > black_at(z) + 1.0 && mid < white_at(z) - 1.0,
        "edge reading {:.4} should sit between {:.4} and {:.4}",
        mid,
        black_at(z),
        white_at(z)
    );
}

#[test]
fn the_ramp_is_monotonic_across_the_edge() {
    let z = 0.004;
    let mut last = -1.0;
    for i in 0..=40 {
        let d = (i as f32 / 40.0) * 0.03;
        let v = line_reflection(d, z);
        assert!(
            v >= last - 1e-4,
            "reading dipped at distance {}: {} after {}",
            d,
            v,
            last
        );
        last = v;
    }
}

#[test]
fn transition_width_equals_sensor_height() {
    // The sensor illuminates a 45-degree cone, so the spot radius equals the
    // height: the black-to-white ramp is exactly `z` wide.
    for z in [0.002, 0.004, 0.008] {
        assert_close(
            line_reflection(HALF_LINE - 0.0001, z),
            black_at(z),
            "just inside the line",
        );
        assert_close(
            line_reflection(HALF_LINE + z * 1.5, z),
            white_at(z),
            "beyond one cone radius past the edge",
        );
        let inside_ramp = line_reflection(HALF_LINE + z * 0.5, z);
        assert!(
            inside_ramp < white_at(z) - 1.0,
            "still inside the ramp at half a cone radius (z={})",
            z
        );
    }
}

#[test]
fn readings_are_symmetric_about_the_line() {
    let z = 0.004;
    for d in [0.0, 0.005, 0.011, 0.013, 0.05] {
        assert_close(
            line_reflection(-d, z),
            line_reflection(d, z),
            "symmetry at distance",
        );
    }
}

#[test]
fn a_non_finite_distance_reads_as_white() {
    let z = 0.004;
    assert_close(line_reflection(f32::NAN, z), white_at(z), "NaN distance");
    assert_close(line_reflection(f32::INFINITY, z), white_at(z), "infinite distance");
}

// --- per-segment distance to the line ----------------------------------------
//
// The collider is a 200 mm slab of drivable surface; the 20 mm line has no
// collider at all. The ray-cast finds *which segment* the sensor is over, and
// these implementations answer *where on it* analytically.

#[test]
fn straight_segments_use_the_local_x_coordinate() {
    let segment = TrackSegment::straight(1.0);
    let tf = GlobalTransform::default();
    let z = 0.004;

    assert_close(
        segment.intersection_to_sensor_value(Vec3::new(0.0, 0.5, 0.0), z, &tf),
        black_at(z),
        "on the line, half way along",
    );
    assert_close(
        segment.intersection_to_sensor_value(Vec3::new(0.05, 0.5, 0.0), z, &tf),
        white_at(z),
        "50 mm off the line",
    );
}

#[test]
fn circle_turns_measure_distance_from_the_arc() {
    let radius = 1.0;
    let segment = TrackSegment::cyrcle_turn(radius, Angle::from_degrees(90.0), Side::Left);
    let tf = GlobalTransform::default();
    let z = 0.004;

    // The segment transform places the arc's *centre* at the origin, so a point
    // exactly `radius` away is on the line.
    assert_close(
        segment.intersection_to_sensor_value(Vec3::new(radius, 0.0, 0.0), z, &tf),
        black_at(z),
        "on the arc",
    );
    assert_close(
        segment.intersection_to_sensor_value(Vec3::new(radius + 0.05, 0.0, 0.0), z, &tf),
        white_at(z),
        "outside the arc",
    );
    assert_close(
        segment.intersection_to_sensor_value(Vec3::new(radius - 0.05, 0.0, 0.0), z, &tf),
        white_at(z),
        "inside the arc",
    );
}
