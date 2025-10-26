use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use bevy::text::cosmic_text::Angle;
use bevy_rapier3d::prelude::*;

use crate::utils::{EntityFeatures, Side, rotate_vec2};

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

pub fn arc_mesh(radius: f32, width: f32, angle: f32, side: Side) -> Mesh {
    // Generate a flat 2D ring/arc mesh in the XY plane (Z = 0).
    // The arc runs from angle=0 pointing along +Y and increases toward +X
    // to match the TrackSegment conventions. The mesh contains only the
    // top surface (single-sided) and is suitable for visualization.

    use bevy::render::mesh::{Indices, PrimitiveTopology};

    let angle = angle.abs() * side.sign();
    let segments: usize =
        ((TRACK_CIRCLE_SEGMENTS_PER_PI as f32 * angle.abs() / PI).round() as usize).max(1);
    let delta = angle / segments as f32;
    let offset = match side {
        Side::Left => 0.0,
        Side::Right => PI,
    };

    let r_in = radius - width / 2.0;
    let r_out = radius + width / 2.0;

    // We'll create (segments + 1) pairs of vertices (inner, outer) along the arc
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((segments + 1) * 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity((segments + 1) * 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity((segments + 1) * 2);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 12); // *2 for double-sided

    for i in 0..=segments {
        let theta = (i as f32) * delta + offset;
        let inner = [r_in * theta.cos(), r_in * theta.sin(), 0.0];
        let outer = [r_out * theta.cos(), r_out * theta.sin(), 0.0];

        // push inner then outer to make indexing predictable
        positions.push(inner);
        positions.push(outer);
        normals.push([0.0, 0.0, 1.0]);
        normals.push([0.0, 0.0, 1.0]);

        // UV: u across the arc, v across the width (inner=0, outer=1)
        let u = (i as f32) / (segments as f32);
        uvs.push([u, 0.0]);
        uvs.push([u, 1.0]);
    }

    // build triangles between consecutive pairs for the top face
    for i in 0..segments {
        let base = (i * 2) as u32; // inner_i = base, outer_i = base+1
        // triangle 1: inner_i, outer_i, outer_i1
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 3);
        // triangle 2: inner_i, outer_i1, inner_i1
        indices.push(base);
        indices.push(base + 3);
        indices.push(base + 2);
    }

    // To make the mesh double-sided, duplicate the vertices for the bottom
    // face with flipped normals, and add triangles with reversed winding.
    let top_vertex_count = positions.len() as u32;

    // duplicate positions, normals (flipped), uvs
    let positions_bottom = positions.clone();
    let mut normals_bottom = normals.clone();
    let uvs_bottom = uvs.clone();
    for n in normals_bottom.iter_mut() {
        n[2] = -n[2];
    }

    // append bottom vertex data
    positions.extend(positions_bottom);
    normals.extend(normals_bottom);
    uvs.extend(uvs_bottom);

    // add reversed-winding triangles for the bottom face
    for i in 0..segments {
        let base = (i * 2) as u32 + top_vertex_count; // inner_i = base, outer_i = base+1
        // reversed triangles: outer_i1, outer_i, inner_i  (reverse of top)
        indices.push(base + 3);
        indices.push(base + 1);
        indices.push(base);
        // reversed triangle 2
        indices.push(base + 2);
        indices.push(base + 3);
        indices.push(base);
    }

    use bevy::render::render_asset::RenderAssetUsages;

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub fn quad_mesh(width: f32, height: f32) -> Mesh {
    let half_x = width * 0.5;
    let half_y = height * 0.5;
    // top face positions
    let mut positions: Vec<[f32; 3]> = vec![
        [-half_x, -half_y, 0.0], // bottom-left
        [half_x, -half_y, 0.0],  // bottom-right
        [half_x, half_y, 0.0],   // top-right
        [-half_x, half_y, 0.0],  // top-left
    ];

    let mut normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; 4];
    let mut uvs: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    // top face (CCW): two triangles covering the quad
    let mut indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];

    // Duplicate vertices for bottom face with flipped normals
    let top_count = positions.len() as u32;
    let positions_bottom = positions.clone();
    let mut normals_bottom = normals.clone();
    let uvs_bottom = uvs.clone();
    for n in normals_bottom.iter_mut() {
        n[2] = -n[2];
    }

    positions.extend(positions_bottom);
    normals.extend(normals_bottom);
    uvs.extend(uvs_bottom);

    // bottom face indices (reversed winding)
    // For the quad (4 vertices) add reversed-winding triangles for the bottom face
    // bottom face: reversed winding of the top face
    indices.extend_from_slice(&[
        top_count + 2,
        top_count + 1,
        top_count + 0,
        top_count + 3,
        top_count + 2,
        top_count + 0,
    ]);

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

#[derive(Debug, Clone, Copy)]
pub struct SegmentTransform {
    position: Vec2,
    direction: Angle,
}

impl std::fmt::Display for SegmentTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SEG x {} y {} ang {}",
            self.position.x,
            self.position.y,
            self.direction.to_degrees()
        )
    }
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

    pub fn mesh(&self) -> Mesh {
        match *self {
            TrackSegment::Start | TrackSegment::End => {
                // Collider::cuboid(TRACK_HALF_WIDTH, TRACK_TIPS_LENGTH / 2.0, TRACK_HALF_HEIGHT)
                quad_mesh(0.1, 0.1)
            }
            TrackSegment::Straight(data) => {
                // Collider::cuboid(TRACK_HALF_WIDTH, data.length / 2.0, TRACK_HALF_HEIGHT)
                quad_mesh(0.1, 0.1)
            }
            TrackSegment::NinetyDegTurn(data) => {
                // let hl: f32 = (data.line_half_length + TRACK_HALF_WIDTH) / 2.0;
                // let ht = (data.line_half_length - TRACK_HALF_WIDTH) / 2.0;
                // // Collider::cuboid(hl, hl, TRACK_HALF_HEIGHT);
                // Collider::compound(vec![
                //     (
                //         Vec3::ZERO,
                //         Quat::IDENTITY,
                //         Collider::cuboid(TRACK_HALF_WIDTH, hl, TRACK_HALF_HEIGHT),
                //     ),
                //     (
                //         Vec3::new(ht * -data.side.sign(), ht, 0.0),
                //         Quat::from_rotation_z(FRAC_PI_2),
                //         Collider::cuboid(TRACK_HALF_WIDTH, hl, TRACK_HALF_HEIGHT),
                //     ),
                // ])
                quad_mesh(0.1, 0.1)
            }
            TrackSegment::CyrcleTurn(data) => arc_mesh(
                data.radius,
                TRACK_HALF_WIDTH * 2.0,
                data.angle.to_radians(),
                data.side,
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

    pub fn spawn(
        &self,
        parent: Entity,
        origin: SegmentTransform,
        features: EntityFeatures,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        let entity = commands
            .spawn((*self, ChildOf(parent), self.transform(origin)))
            .id();
        if features.has_physics() {
            commands.entity(entity).insert((
                self.collider(),
                RigidBody::Fixed,
                Friction {
                    coefficient: 0.5,
                    combine_rule: CoefficientCombineRule::Average,
                },
            ));
        }
        if features.has_visualization() {
            commands.entity(entity).insert((
                Mesh3d(meshes.add(self.mesh())),
                MeshMaterial3d(materials.add(Color::srgba(0.0, 0.0, 0.0, 1.0))),
            ));
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

    pub fn spawn_bundles(
        &self,
        parent: Entity,
        features: EntityFeatures,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let mut segment_origin = self.origin;

        for segment in &self.segments {
            segment.spawn(
                parent,
                segment_origin,
                features,
                &mut commands,
                &mut meshes,
                &mut materials,
            );
            segment_origin = segment.compute_next_origin(segment_origin);
        }
    }
}

fn setup_track(
    mut commands: Commands,
    track_parent: Entity,
    features: EntityFeatures,
    track: Res<Track>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    if features.has_physics() {
        // Static floor
        commands.spawn((
            Collider::cuboid(FLOOR_SIZE / 2.0, FLOOR_SIZE / 2.0, FLOOR_HEIGHT / 2.0),
            RigidBody::Fixed,
            Friction::new(0.5),
            Transform::from_xyz(0.0, 0.0, -FLOOR_HEIGHT / 2.0),
        ));
    }

    track.spawn_bundles(track_parent, features, commands, meshes, materials);
}

pub struct TrackPlugin {
    features: EntityFeatures,
}

impl TrackPlugin {
    pub fn new(features: EntityFeatures) -> Self {
        Self { features }
    }
}

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        let features = self.features;
        app.insert_resource(Track::new(vec![
            TrackSegment::start(),
            TrackSegment::straight(2.0),
            TrackSegment::ninety_deg_turn(0.5, Side::Right),
            TrackSegment::cyrcle_turn(1.0, Angle::from_degrees(120.0), Side::Left),
            TrackSegment::ninety_deg_turn(1.0, Side::Left),
            TrackSegment::cyrcle_turn(2.0, Angle::from_degrees(60.0), Side::Right),
            TrackSegment::end(),
        ]))
        .add_systems(
            Startup,
            move |mut commands: Commands,
                  track: Res<Track>,
                  meshes: ResMut<Assets<Mesh>>,
                  materials: ResMut<Assets<StandardMaterial>>| {
                let track_parent = commands.spawn(Transform::default()).id();
                setup_track(commands, track_parent, features, track, meshes, materials)
            },
        );
    }
}
