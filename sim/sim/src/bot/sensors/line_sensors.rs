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
    // EXERCISE 5.3: model what happens when the sensor bar lifts off the ground.
    //
    // Nobody predicts this one. As the sensor rises, the *reflected* signal weakens
    // while the *ambient* light it picks up stays constant, so contrast collapses:
    // black stops reading 0 and starts reading 30, white stops reading 100 and
    // starts reading 70, and both converge on ambient. Lifted far enough, the sensor
    // is blind and everything reads about the same - and the robot's weighted-mean
    // error becomes garbage. This is a real failure mode competitors hit.
    //
    //   value 100 | white ---.
    //             |           `--.
    //          50 |  ambient -----X=====   both converge
    //             |           .--'
    //           0 | black ---'
    //             +----+-----------+------
    //                2mm         20mm      sensor height z
    //
    // Take `value` as 0.0 (pure black) to 100.0 (pure white) and return what the
    // sensor actually reads at height `z`:
    //
    //   Z_MIN         0.002    no attenuation at all at 2 mm
    //   Z_MAX         0.02     effectively blind at 20 mm
    //   VALUE_AMBIENT 50.0     what a blind sensor converges on
    //
    // Interpolate the available range linearly between those heights. (The real
    // relation is closer to quadratic; linear is an honest simplification, and
    // exactly the kind of decision simulator authors make all day.)
    todo!("attenuate the reading toward ambient as z grows")
}

// Called by EXERCISE 5.4 once implemented.
#[allow(dead_code, unused_variables)]
fn line_reflection(x: f32, z: f32) -> f32 {
    const LINE_SIZE: f32 = 0.02; // 20 mm

    // EXERCISE 5.2: model a reflectance sensor `x` metres from the line centre.
    //
    // The naive model is a step function: black inside the line, white outside.
    // That is wrong, and the wrongness matters - a PID fed a step function cannot
    // steer smoothly, only bang-bang.
    //
    // A real sensor is an IR LED next to a phototransistor, and it illuminates a
    // *cone*, so it averages over a disc on the floor. As that disc crosses the
    // edge, the reading slides continuously from black to white, and that gradient
    // is the signal the robot's weighted mean actually steers on.
    //
    // The spot radius is not a constant: the code assumes a 45-degree half-aperture
    // cone, so work out the transition width from `z`. (tan 45 = 1.)
    //
    // So: return 0.0 within half a line width of the centre, 100.0 beyond the
    // transition region, and a smooth ramp in between - smoothstep,
    // s = t * t * (3 - 2t), is what the reference uses. Treat a non-finite `x` as
    // far away.
    //
    // Then pass the result through `line_reflection_attenuation` (5.3): compute
    // reflectance from geometry *first*, then degrade it for height.
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
        // EXERCISE 5.4: how far is this point from the centre of the line?
        //
        // Here is the trick that makes the whole thing work. Look at the constants
        // in `track.rs`: TRACK_HALF_WIDTH is 0.1, so the *collider* is a 200 mm slab
        // of drivable surface. LINE_HALF_WIDTH is 0.01 - the line is 20 mm of paint
        // in the middle of it, and it has no collider at all.
        //
        // So the ray-cast does not find the line. It finds *which segment* the
        // sensor is over, and where it hit. The segment's own geometry then answers
        // the geometric question analytically. Let the physics engine answer what it
        // is good at, and do the maths yourself: line-accurate colliders would mean
        // thousands of tiny shapes, a slower broad phase and *worse* precision.
        //
        // `point_to_new_origin` (provided) converts the world hit point into the
        // segment's local frame. Then, per segment type:
        //
        //  - Start / End / Straight: the local x coordinate is the distance. Done.
        //  - CyrcleTurn: the segment transform puts the arc's *centre* at the local
        //    origin, so compare the distance from that origin against the radius.
        //    Mind `data.side.sign()`: get it wrong and the error is mirrored, so the
        //    robot steers confidently the wrong way.
        //  - NinetyDegTurn: a square corner is two legs. Work out which one you are
        //    nearer by comparing the local coordinates against the diagonal, then
        //    measure to that leg.
        //
        // Feed whichever distance you get to `line_reflection(distance, z)`.
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
    // EXERCISE 5.1: read all sixteen sensors, by casting a ray from each one.
    //
    // Right now every sensor reports pure white, which is why the robot cannot see
    // the line. There is a worked example of the ray-cast idiom forty lines away in
    // `sensors/bot_position.rs` - read `compute_bot_position` first.
    //
    // For each sensor entity, in query order (the index matters - it is the
    // sensor's position along the bar):
    //
    //  - cast a ray from its global translation, *downwards in its own frame*. Using
    //    a hardcoded world axis works on the flat and is wrong the moment the robot
    //    pitches, which is exactly when you need it to be right.
    //  - restrict the query to track segments with a `QueryFilter` predicate.
    //    Without it, sensors happily read the robot's own bumper.
    //  - on a hit: look up the segment and its transform, and ask
    //    `intersection_to_sensor_value` (5.4) what the reading is. Take `z` from the
    //    sensor's *global* transform - the local z is a constant, so the lift model
    //    would silently never fire.
    //  - on a miss the sensor is off the track entirely: it sees white floor, still
    //    attenuated for its height.
    //  - add noise with `rng.noisy_value(value, NOISE)` and clamp to 0.0 - 100.0.
    //    The noise is seeded, so it is reproducible. It is here because a robot
    //    tuned against noiseless sensors has a KD term that explodes on real
    //    hardware.
    //
    // Then a seventeenth ray-cast, from the robot body (`bot_body_query`), whose
    // only job is to set `is_out_of_track` and `is_over_track_end`. That is how a
    // race ends: no hit means the robot drove off the track, and a hit on
    // `TrackSegment::End` means it finished. Two booleans that decide the whole
    // competition.
    //
    // See: RapierContext::cast_ray_and_get_normal, QueryFilter::predicate,
    //      TrackSegment::is_end
    #[allow(dead_code)]
    const NOISE: f32 = 1.0;

    for i in 0..16 {
        sensors_data.line_sensors[i] = 100.0;
    }
    sensors_data.is_out_of_track = false;
    sensors_data.is_over_track_end = false;
}
