use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::track::{TRACK_HALF_WIDTH, TrackSegment};
use crate::utils::{NormalRandom, point_to_new_origin};
use execution_data::SensorsData;

#[inline]
fn line_reflection_attenuation(value: f32, z: f32) -> f32 {
    // Attenuation model: increased z makes to that the sensor perceives
    // more ambient light and less refrected light.
    // At "base" z (0.002, 2mm) there is no attenuetion.
    // At "max" z (0.02, 20mm) the sensor perceives only ambient light.
    // The relation should likely be quadratic but for simplicity we'll keep it linear
    // Total black is 0.0, total white is 100.0, ambient light is 50.0

    const VALUE_AMBIENT: f32 = 50.0;
    const VALUE_MIN: f32 = 0.0;
    const VALUE_MAX: f32 = 100.0;
    const Z_MIN: f32 = 0.002;
    const Z_MAX: f32 = 0.02;
    const Z_RANGE: f32 = Z_MAX - Z_MIN;
    const VALUE_MIN_RANGE: f32 = VALUE_AMBIENT - VALUE_MIN;
    const VALUE_MAX_RANGE: f32 = VALUE_MAX - VALUE_AMBIENT;
    const VALUE_RANGE: f32 = VALUE_MAX - VALUE_MIN;

    let z_normalized = ((z - Z_MIN) / Z_RANGE).clamp(0.0, 1.0);
    let value_min = VALUE_MIN + (VALUE_MIN_RANGE * z_normalized);
    let value_max = VALUE_MAX - (VALUE_MAX_RANGE * z_normalized);
    let value_normalized = (value / VALUE_RANGE).clamp(0.0, 1.0);
    let value = value_min + (value_max - value_min) * value_normalized;
    value
}

fn line_reflection(x: f32, z: f32) -> f32 {
    const LINE_SIZE: f32 = 0.02; // 20 mm

    // Model: black line of width LINE_SIZE centered at 0 on a white floor.
    // The sensor doesn't have infinite spatial resolution, so we smooth the
    // transition between black and white across a finite transition region.
    // We return 0.0 for pure black, 100.0 for pure white.

    // Transition width (how quickly the sensor goes from black to white).
    // Presuming the sensor projects (and senses) a cone of 45deg of half-aperture,
    // the transition width is the sensor z coordinate.
    let transition = z;

    let value = {
        if x.is_finite() {
            let half = LINE_SIZE * 0.5;
            let d = x.abs();

            if d <= half {
                // Fully over the black line
                0.0
            } else if d >= half + transition {
                // Far enough to see full white
                100.0
            } else {
                // Smooth interpolation between black and white using smoothstep
                let t = (d - half) / transition; // normalized 0..1
                // smoothstep (cubic hermite) -> smooth start/end
                let s = t * t * (3.0 - 2.0 * t);
                100.0 * s
            }
        } else {
            // Treat NaN/inf as far away (white)
            100.0
        }
    };
    line_reflection_attenuation(value, z)
}

trait TrackSimulateLine {
    fn intersection_to_sensor_value(&self, point: Vec3, z: f32, transform: &GlobalTransform)
    -> f32;
}

impl TrackSimulateLine for TrackSegment {
    fn intersection_to_sensor_value(
        &self,
        point: Vec3,
        z: f32,
        transform: &GlobalTransform,
    ) -> f32 {
        let local_point = point_to_new_origin(point, transform);

        match *self {
            TrackSegment::Start | TrackSegment::End => line_reflection(local_point.x, z),
            TrackSegment::Straight(_) => line_reflection(local_point.x, z),
            TrackSegment::NinetyDegTurn(data) => {
                let turn_y = (data.line_half_length - TRACK_HALF_WIDTH) / 2.0;
                let dist_to_line = if local_point.y < data.side.sign() * local_point.x + turn_y {
                    local_point.x
                } else {
                    data.side.sign() * (local_point.y - turn_y)
                };
                line_reflection(dist_to_line, z)
            }
            TrackSegment::CyrcleTurn(data) => {
                let dist_to_line = (local_point.length() - data.radius) * data.side.sign();
                line_reflection(dist_to_line, z)
            }
        }
    }
}

#[derive(Component, Default)]
pub struct LineSensor {}

pub fn compute_sensor_readings(
    read_rapier_context: ReadRapierContext,
    sensors_query: Query<&GlobalTransform, With<LineSensor>>,
    track_segments_query: Query<(&TrackSegment, &GlobalTransform)>,
    mut rng: ResMut<NormalRandom>,
    mut sensors_data: ResMut<SensorsData>,
) {
    let rapier_context = read_rapier_context.single().unwrap();

    for (i, sensor_tf) in sensors_query.iter().enumerate() {
        const NOISE: f32 = 1.0;
        let origin = sensor_tf.translation();
        let sensor_z = sensor_tf.translation().z;

        let dir = sensor_tf.rotation().mul_vec3(Vec3::NEG_Z);
        let max_toi = 0.1;

        if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            origin,
            dir,
            max_toi,
            true,
            QueryFilter::default().predicate(&|entity| track_segments_query.get(entity).is_ok()),
        ) {
            // Sensor is over the track
            let point: Vec3 = intersection.point.into();
            let (track_segment, transform) = track_segments_query.get(entity).unwrap();
            sensors_data.line_sensors[i] = rng
                .noisy_value(
                    track_segment.intersection_to_sensor_value(point, sensor_z, transform),
                    NOISE,
                )
                .clamp(0.0, 100.0);
        } else {
            // Sensor is out
            sensors_data.line_sensors[i] = rng
                .noisy_value(line_reflection_attenuation(100.0, sensor_z), NOISE)
                .clamp(0.0, 100.0);
        }
    }
}
