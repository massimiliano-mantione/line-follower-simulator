use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use bevy::text::cosmic_text::Angle;
use bevy_rapier3d::prelude::*;

use crate::utils::{Side, rotate_vec2};

const FLOOR_HEIGHT: f32 = 0.01;
const FLOOR_SIZE: f32 = 20.0;
pub const TRACK_HALF_WIDTH: f32 = 0.1;
const TRACK_HALF_HEIGHT: f32 = 0.001;
const TRACK_TIPS_LENGTH: f32 = 0.5;
const TRACK_Z_OFFSET: f32 = -TRACK_HALF_HEIGHT - FLOOR_HEIGHT;
const TRACK_CIRCLE_SEGMENTS_PER_PI: usize = 40;

/// Generates a curved "track turn" collider (an arc section)
///
/// # Arguments
/// * `radius`  - Inner radius of the arc
/// * `width`   - Thickness (distance between inner and outer edges)
/// * `angle`   - Total arc angle in radians (e.g., PI/2 for 90° turn)
/// * `height`  - Collider height/thickness
/// * `segments` - Number of convex segments for smoothness
pub fn arc_collider(radius: f32, width: f32, angle: f32, side: Side, height: f32) -> Collider {
    // Approximate the curved arc by composing `segments` small box colliders
    // placed along the arc. Each box is oriented so its long side follows
    // the tangent of the arc. The arc is generated so that angle=0 points
    // in the +Y direction and increases toward +X (so it matches the
    // TrackSegment transform conventions used elsewhere).

    let angle = angle.abs() * side.sign();
    let segments: usize =
        ((TRACK_CIRCLE_SEGMENTS_PER_PI as f32 * angle.abs() / PI).round() as usize).max(1);
    let delta = angle / segments as f32;
    let offset = match side {
        Side::Left => 0.0,
        Side::Right => PI,
    };

    // Collider::compound for bevy_rapier expects parts as (Vec3, Quat, Collider)
    let mut parts: Vec<(Vec3, Quat, Collider)> = Vec::with_capacity(segments);

    for i in 0..segments {
        // angular bounds for this piece
        let theta0 = (i as f32) * delta + offset;
        let theta1 = theta0 + delta;

        let r_in = radius - width / 2.0;
        let r_out = radius + width / 2.0;
        let hz = height * 0.5;

        // build 8 vertices for the prism: inner/out x theta0/theta1 x z-/+
        let mut pts: Vec<Vec3> = Vec::with_capacity(8);

        let p =
            |r: f32, theta: f32, z: f32| -> Vec3 { Vec3::new(r * theta.cos(), r * theta.sin(), z) };

        // inner theta0, z-
        pts.push(p(r_in, theta0, -hz));
        // inner theta0, z+
        pts.push(p(r_in, theta0, hz));
        // inner theta1, z-
        pts.push(p(r_in, theta1, -hz));
        // inner theta1, z+
        pts.push(p(r_in, theta1, hz));

        // outer theta0, z-
        pts.push(p(r_out, theta0, -hz));
        // outer theta0, z+
        pts.push(p(r_out, theta0, hz));
        // outer theta1, z-
        pts.push(p(r_out, theta1, -hz));
        // outer theta1, z+
        pts.push(p(r_out, theta1, hz));

        // place the convex hull at origin; positions are absolute in world-space
        // but Collider::compound wants local translations per part. We'll compute
        // the center of these points and use a local transform so vertices are
        // relative to that center.
        let mut center = Vec3::ZERO;
        for v in &pts {
            center += v;
        }
        center /= pts.len() as f32;

        let rel_pts_vec3: Vec<Vec3> = pts
            .into_iter()
            .map(|v| Vec3::new(v[0] - center.x, v[1] - center.y, v[2] - center.z))
            .collect();

        // Collider::convex_hull commonly accepts a slice of Vec3 and returns
        // an Option<Collider>. Use that if available, otherwise fall back to
        // a cuboid approximation.
        let convex = if let Some(c) = Collider::convex_hull(&rel_pts_vec3) {
            c
        } else {
            Collider::cuboid((r_out - r_in) * 0.5, (radius * delta) * 0.5, hz)
        };

        // The compound part takes translation and rotation; we keep identity
        // rotation because vertices already oriented in world XY plane, and
        // translate by the computed center.
        parts.push((center, Quat::IDENTITY, convex));
    }

    Collider::compound(parts)
}

#[derive(Debug, Clone, Copy)]
pub struct SegmentTransform {
    position: Vec2,
    direction: Angle,
}

impl SegmentTransform {
    pub fn new(position: Vec2, direction: Angle) -> Self {
        Self {
            position,
            direction,
        }
    }

    pub fn translate_in_direction(&self, translation: Vec2) -> Self {
        Self {
            position: self.position + rotate_vec2(translation, self.direction.to_radians()),
            direction: self.direction,
        }
    }

    pub fn rotate(&self, rotation: Angle) -> Self {
        Self {
            position: self.position,
            direction: Angle::from_radians(self.direction.to_radians() + rotation.to_radians()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StraightSegment {
    pub(crate) length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NinetyDegTurnSegment {
    pub(crate) line_half_length: f32,
    pub(crate) side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CyrcleTurnSegment {
    pub(crate) radius: f32,
    pub(crate) side: Side,
    pub(crate) angle: Angle,
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum TrackSegment {
    Start,
    End,
    Straight(StraightSegment),
    NinetyDegTurn(NinetyDegTurnSegment),
    CyrcleTurn(CyrcleTurnSegment),
}

impl TrackSegment {
    pub fn start() -> Self {
        Self::Start
    }

    pub fn end() -> Self {
        Self::End
    }

    pub fn straight(length: f32) -> Self {
        Self::Straight(StraightSegment { length })
    }

    pub fn ninety_deg_turn(line_half_length: f32, side: Side) -> Self {
        Self::NinetyDegTurn(NinetyDegTurnSegment {
            line_half_length: line_half_length,
            side,
        })
    }

    pub fn cyrcle_turn(radius: f32, angle: Angle, side: Side) -> Self {
        Self::CyrcleTurn(CyrcleTurnSegment {
            radius,
            angle,
            side,
        })
    }

    pub fn collider(&self) -> Collider {
        match *self {
            TrackSegment::Start | TrackSegment::End => {
                Collider::cuboid(TRACK_HALF_WIDTH, TRACK_TIPS_LENGTH / 2.0, TRACK_HALF_HEIGHT)
            }
            TrackSegment::Straight(data) => {
                Collider::cuboid(TRACK_HALF_WIDTH, data.length / 2.0, TRACK_HALF_HEIGHT)
            }
            TrackSegment::NinetyDegTurn(data) => {
                let hl: f32 = (data.line_half_length + TRACK_HALF_WIDTH) / 2.0;
                let ht = (data.line_half_length - TRACK_HALF_WIDTH) / 2.0;
                // Collider::cuboid(hl, hl, TRACK_HALF_HEIGHT);
                Collider::compound(vec![
                    (
                        Vec3::ZERO,
                        Quat::IDENTITY,
                        Collider::cuboid(TRACK_HALF_WIDTH, hl, TRACK_HALF_HEIGHT),
                    ),
                    (
                        Vec3::new(ht * -data.side.sign(), ht, 0.0),
                        Quat::from_rotation_z(FRAC_PI_2),
                        Collider::cuboid(TRACK_HALF_WIDTH, hl, TRACK_HALF_HEIGHT),
                    ),
                ])
            }
            TrackSegment::CyrcleTurn(data) => arc_collider(
                data.radius,
                TRACK_HALF_WIDTH * 2.0,
                data.angle.to_radians(),
                data.side,
                TRACK_HALF_HEIGHT * 2.0,
            ),
        }
    }

    pub fn transform(&self, origin: SegmentTransform) -> Transform {
        let transform_origin = match *self {
            TrackSegment::Start | TrackSegment::End => {
                origin.translate_in_direction(Vec2::Y * TRACK_TIPS_LENGTH / 2.0)
            }
            TrackSegment::Straight(data) => {
                origin.translate_in_direction(Vec2::Y * data.length / 2.0)
            }
            TrackSegment::NinetyDegTurn(data) => origin
                .translate_in_direction(Vec2::Y * (data.line_half_length + TRACK_HALF_WIDTH) / 2.0),
            TrackSegment::CyrcleTurn(data) => {
                origin.translate_in_direction(Vec2::NEG_X * data.radius * data.side.sign())
            }
        };
        Transform::from_translation(transform_origin.position.extend(TRACK_Z_OFFSET)).with_rotation(
            Quat::from_rotation_z(transform_origin.direction.to_radians()),
        )
    }

    pub fn compute_next_origin(&self, origin: SegmentTransform) -> SegmentTransform {
        match *self {
            TrackSegment::Start | TrackSegment::End => {
                origin.translate_in_direction(Vec2::Y * TRACK_TIPS_LENGTH)
            }
            TrackSegment::Straight(data) => origin.translate_in_direction(Vec2::Y * data.length),
            TrackSegment::NinetyDegTurn(data) => origin
                .translate_in_direction(Vec2::new(
                    -data.line_half_length * data.side.sign(),
                    data.line_half_length,
                ))
                .rotate(Angle::from_degrees(90.0 * data.side.sign())),
            TrackSegment::CyrcleTurn(data) => origin
                .translate_in_direction(Vec2::new(
                    data.radius * (data.angle.to_radians().cos() - 1.0) * data.side.sign(),
                    data.radius * data.angle.to_radians().sin(),
                ))
                .rotate(Angle::from_radians(
                    data.angle.to_radians() * data.side.sign(),
                )),
        }
    }
}

#[derive(Resource)]
pub struct Track {
    origin: SegmentTransform,
    segments: Vec<TrackSegment>,
}

impl Track {
    pub fn new(segments: Vec<TrackSegment>) -> Self {
        Self {
            origin: SegmentTransform::new(Vec2::NEG_Y * TRACK_TIPS_LENGTH / 2.0, Angle::ZERO),
            segments,
        }
    }

    pub fn spawn_bundles(&self, mut commands: Commands) {
        let mut segment_origin = self.origin;

        for segment in &self.segments {
            commands.spawn((
                segment.collider(),
                *segment,
                segment.transform(segment_origin),
                RigidBody::Fixed,
                Friction {
                    coefficient: 0.5,
                    combine_rule: CoefficientCombineRule::Average,
                },
            ));
            segment_origin = segment.compute_next_origin(segment_origin);
        }
    }
}

fn setup_track(mut commands: Commands, track: Res<Track>) {
    // Static floor
    commands.spawn((
        Collider::cuboid(FLOOR_SIZE / 2.0, FLOOR_SIZE / 2.0, FLOOR_HEIGHT / 2.0),
        RigidBody::Fixed,
        Friction::new(0.5),
        Transform::from_xyz(0.0, 0.0, -FLOOR_HEIGHT / 2.0),
    ));

    track.spawn_bundles(commands);
}

pub fn add_track(app: &mut App) -> &mut App {
    app.insert_resource(Track::new(vec![
        TrackSegment::start(),
        TrackSegment::straight(2.0),
        TrackSegment::ninety_deg_turn(0.5, Side::Right),
        TrackSegment::cyrcle_turn(1.0, Angle::from_degrees(120.0), Side::Left),
        TrackSegment::ninety_deg_turn(1.0, Side::Left),
        TrackSegment::cyrcle_turn(2.0, Angle::from_degrees(60.0), Side::Right),
        TrackSegment::end(),
    ]))
    .add_systems(Startup, setup_track)
}
