use bevy::{math::Vec2, text::cosmic_text::Angle};

use crate::{
    TrackId,
    track::{SegmentTransform, Track, TrackSegment},
    utils::Side,
};

fn build_line_track() -> Track {
    Track::new(
        v2(2.0, 6.0),
        origin(0.0, -1.75, 0.0),
        vec![start(), straight(3.0), end()],
    )
}

fn build_angle_track() -> Track {
    Track::new(
        v2(3.0, 5.5),
        origin(-0.5, -1.75, 0.0),
        vec![
            start(),
            straight(1.5),
            t90(RIGHT, 0.5),
            t90(LEFT, 0.5),
            straight(0.5),
            end(),
        ],
    )
}

fn build_turn_track() -> Track {
    Track::new(
        v2(4.0, 4.5),
        origin(-1.0, -1.25, 0.0),
        vec![
            start(),
            straight(1.0),
            turn(135.0, RIGHT, 0.75),
            straight(0.25),
            turn(180.0, LEFT, 0.5),
            straight(0.25),
            end(),
        ],
    )
}

fn build_simple_track() -> Track {
    Track::new(
        v2(5.2, 7.2),
        origin(0.4, -2.5, 0.0),
        vec![
            start(),
            straight(2.0),
            t90(RIGHT, 0.5),
            turn(120.0, LEFT, 1.0),
            t90(LEFT, 1.0),
            turn(60.0, RIGHT, 2.0),
            end(),
        ],
    )
}

pub fn build_track(id: TrackId) -> Track {
    match id {
        TrackId::Line => build_line_track(),
        TrackId::Angle => build_angle_track(),
        TrackId::Turn => build_turn_track(),
        TrackId::Simple => build_simple_track(),
        TrackId::Race => unimplemented!(),
    }
}

fn v2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn start() -> TrackSegment {
    TrackSegment::start()
}

fn end() -> TrackSegment {
    TrackSegment::end()
}

fn straight(length: f32) -> TrackSegment {
    TrackSegment::straight(length)
}

fn t90(side: Side, radius: f32) -> TrackSegment {
    TrackSegment::ninety_deg_turn(radius, side)
}

fn turn(angle: f32, side: Side, radius: f32) -> TrackSegment {
    TrackSegment::cyrcle_turn(radius, Angle::from_degrees(angle), side)
}

fn origin(x: f32, y: f32, angle: f32) -> SegmentTransform {
    SegmentTransform::new(Vec2::new(x, y), Angle::from_degrees(angle))
}

const LEFT: Side = Side::Left;
const RIGHT: Side = Side::Right;
