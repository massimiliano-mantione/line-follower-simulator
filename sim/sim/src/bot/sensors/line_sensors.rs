use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::track::TrackSegment;
// `point_to_new_origin` is used by EXERCISE 5.4 once it is implemented.
#[allow(unused_imports)]
use crate::utils::{NormalRandom, point_to_new_origin};
use execution_data::SensorsData;

use super::bot_position::BotPositionDetector;

#[cfg(test)]
#[path = "line_sensors_tests.rs"]
mod tests;

// Called by EXERCISE 5.1 and 5.2 once implemented.
#[inline]
#[allow(dead_code, unused_variables)]
fn line_reflection_attenuation(value: f32, z: f32) -> f32 {
    todo!("attenuate the reading toward ambient as z grows")
}

// Called by EXERCISE 5.4 once implemented.
#[allow(dead_code, unused_variables)]
fn line_reflection(x: f32, z: f32) -> f32 {
    const LINE_SIZE: f32 = 0.02; // 20 mm

    todo!("black line, white floor, smooth edge")
}

// Implemented for TrackSegment below, and called by EXERCISE 5.1.
#[allow(dead_code)]
trait TrackSimulateLine {
    fn intersection_to_sensor_value(&self, point: Vec3, z: f32, transform: &GlobalTransform)
    -> f32;
}

impl TrackSimulateLine for TrackSegment {
    #[allow(unused_variables)]
    fn intersection_to_sensor_value(
        &self,
        point: Vec3,
        z: f32,
        transform: &GlobalTransform,
    ) -> f32 {
        todo!("distance to the line, per segment type")
    }
}

#[derive(Component, Default, Clone)]
pub struct LineSensor {}

#[allow(unused_variables, unused_mut)]
pub fn compute_sensor_readings(
    read_rapier_context: ReadRapierContext,
    sensors_query: Query<&GlobalTransform, With<LineSensor>>,
    bot_body_query: Query<&GlobalTransform, With<BotPositionDetector>>,
    track_segments_query: Query<(&TrackSegment, &GlobalTransform)>,
    mut rng: ResMut<NormalRandom>,
    mut sensors_data: ResMut<SensorsData>,
) {
    #[allow(dead_code)]
    const NOISE: f32 = 1.0;

    for i in 0..16 {
        sensors_data.line_sensors[i] = 100.0;
    }
    sensors_data.is_out_of_track = false;
    sensors_data.is_over_track_end = false;
}
