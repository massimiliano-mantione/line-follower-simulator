// Several of these are used only by the EXERCISE 10.x bodies once implemented.
#![allow(unused_imports)]

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{light::NotShadowCaster, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::utils::{Angle, EntityFeatures, Side, rotate_vec2};

const FLOOR_HEIGHT: f32 = 0.05;
pub const TRACK_HALF_WIDTH: f32 = 0.1;
pub const LINE_HALF_WIDTH: f32 = 0.01;
const TRACK_HALF_HEIGHT: f32 = 0.001;
const TRACK_TIPS_LENGTH: f32 = 0.5;
const TRACK_CIRCLE_SEGMENTS_PER_PI: usize = 40;

const TRACK_ORIGIN_OFFSET: Vec2 = Vec2::new(0.0, -0.25);

/// Generates a curved "track turn" collider (an arc section)
///
/// # Arguments
/// * `radius`  - Inner radius of the arc
/// * `width`   - Thickness (distance between inner and outer edges)
/// * `angle`   - Total arc angle in radians (e.g., PI/2 for 90° turn)
/// * `height`  - Collider height/thickness
/// * `segments` - Number of convex segments for smoothness
#[allow(dead_code)]
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

    let angle = angle.abs();
    let segments: usize =
        ((TRACK_CIRCLE_SEGMENTS_PER_PI as f32 * angle / PI).round() as usize).max(1);
    let delta = angle / segments as f32;
    let offset = match side {
        Side::Left => 0.0,
        Side::Right => PI - angle,
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

    use bevy::asset::RenderAssetUsages;

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
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub fn ninety_deg_mesh(width: f32, half_lenght: f32, side: Side) -> Mesh {
    let half_w = width * 0.5;

    // top face positions
    let mut positions: Vec<[f32; 3]> = match side {
        Side::Left => vec![
            [-half_w, -half_lenght, 0.0], // bottom-left
            [half_w, -half_lenght, 0.0],  // bottom-right
            [half_w, half_w, 0.0],        // top-right
            [-half_lenght, half_w, 0.0],  // top-left
            [-half_lenght, -half_w, 0.0], // mid-left
            [-half_w, -half_w, 0.0],      // mid-right
        ],
        Side::Right => vec![
            [-half_w, -half_lenght, 0.0], // bottom-left
            [half_w, -half_lenght, 0.0],  // bottom-right
            [half_w, -half_w, 0.0],       // mid-left
            [half_lenght, -half_w, 0.0],  // mid-right
            [half_lenght, half_w, 0.0],   // top-right
            [-half_w, half_w, 0.0],       // top-left
        ],
    };

    let mut normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; 6];
    // let mut uvs: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    // top face (CCW): two triangles covering the quad
    let mut indices: Vec<u32> = vec![0, 1, 2, 0, 2, 5, 2, 3, 4, 2, 4, 5];

    // Duplicate vertices for bottom face with flipped normals
    let top_count = positions.len() as u32;
    let positions_bottom = positions.clone();
    let mut normals_bottom = normals.clone();
    // let uvs_bottom = uvs.clone();
    for n in normals_bottom.iter_mut() {
        n[2] = -n[2];
    }

    positions.extend(positions_bottom);
    normals.extend(normals_bottom);
    // uvs.extend(uvs_bottom);

    // bottom face indices (reversed winding)
    // For the quad (4 vertices) add reversed-winding triangles for the bottom face
    // bottom face: reversed winding of the top face
    indices.extend_from_slice(&[
        top_count + 2,
        top_count + 1,
        top_count + 0,
        top_count + 5,
        top_count + 2,
        top_count + 0,
        top_count + 4,
        top_count + 3,
        top_count + 2,
        top_count + 5,
        top_count + 4,
        top_count + 2,
    ]);

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    // .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
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

#[allow(dead_code)]
impl SegmentTransform {
    pub fn new(position: Vec2, direction: Angle) -> Self {
        Self {
            position,
            direction,
        }
    }

    #[allow(unused_variables)]
    pub fn translate_in_direction(&self, translation: Vec2) -> Self {
        // EXERCISE 10.1: move forward, in whatever direction we are currently facing.
        //
        // This is the turtle. `translation` is in *local* coordinates - +Y is
        // "ahead" - so it has to be rotated into world space before being added to
        // the position. `rotate_vec2` (provided) does the rotation.
        //
        // Note this returns a new value rather than mutating: `transform` needs to
        // compute a segment's own placement without disturbing the chain.
        *self
    }

    #[allow(unused_variables)]
    pub fn rotate(&self, rotation: Angle) -> Self {
        // EXERCISE 10.2: turn in place.
        *self
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
    pub fn is_end(&self) -> bool {
        *self == TrackSegment::End
    }
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
        // EXERCISE 10.5: the physics surface for this segment.
        //
        // Note the scale: TRACK_HALF_WIDTH is 0.1, so this is a 200 mm slab of
        // *drivable surface*, not a 20 mm painted line. The line has no collider at
        // all - lesson 05 computes distance to it analytically. One declaration,
        // three different artifacts.
        //
        //  - Start / End: a cuboid TRACK_TIPS_LENGTH long.
        //  - Straight: the same, `data.length` long.
        //  - CyrcleTurn: `arc_collider` (provided) builds it from convex segments.
        //  - NinetyDegTurn: a square corner is *two* overlapping cuboids, not one
        //    rotated one. Build a compound.
        //
        // Every segment is TRACK_HALF_HEIGHT * 2 thick.
        //
        // Right now every segment gets the Start/End shape, which is why the track
        // has no surface to speak of.
        Collider::cuboid(TRACK_HALF_WIDTH, TRACK_TIPS_LENGTH / 2.0, TRACK_HALF_HEIGHT)
    }

    pub fn mesh(&self) -> Mesh {
        match *self {
            TrackSegment::Start | TrackSegment::End => {
                // Collider::cuboid(TRACK_HALF_WIDTH, TRACK_TIPS_LENGTH / 2.0, TRACK_HALF_HEIGHT)
                quad_mesh(LINE_HALF_WIDTH * 2.0, TRACK_TIPS_LENGTH)
            }
            TrackSegment::Straight(data) => {
                // Collider::cuboid(TRACK_HALF_WIDTH, data.length / 2.0, TRACK_HALF_HEIGHT)
                quad_mesh(LINE_HALF_WIDTH * 2.0, data.length)
            }
            TrackSegment::NinetyDegTurn(data) => {
                ninety_deg_mesh(LINE_HALF_WIDTH * 2.0, data.line_half_length, data.side)
            }
            TrackSegment::CyrcleTurn(data) => arc_mesh(
                data.radius,
                LINE_HALF_WIDTH * 2.0,
                data.angle.to_radians(),
                data.side,
            ),
        }
    }

    #[allow(unused_variables)]
    pub fn transform(&self, origin: SegmentTransform) -> Transform {
        // EXERCISE 10.3: where does this segment *sit*?
        //
        // Careful: this is not the same question as 10.4. `origin` is where the
        // segment *starts*, but colliders and meshes are built centred on their own
        // local origin, so each type needs shifting:
        //
        //  - Start / End: forward by half of TRACK_TIPS_LENGTH.
        //  - Straight: forward by half its length.
        //  - NinetyDegTurn: forward by `line_half_length`.
        //  - CyrcleTurn: sideways by `radius`, so the transform lands on the
        //    *centre of the circle*. That is what lets lesson 05 write
        //    `local_point.length() - radius`. Mind `side.sign()`.
        //
        // Then turn that `SegmentTransform` into a Bevy `Transform`: position in the
        // XY plane at z = 0, rotated about Z by the direction.
        //
        // Conflating "where do I sit" with "where does the next one start" is the
        // most common bug here, and it makes the track drift apart segment by
        // segment.
        Transform::default()
    }

    #[allow(unused_variables)]
    pub fn compute_next_origin(&self, origin: SegmentTransform) -> SegmentTransform {
        // EXERCISE 10.4: where does the *next* segment start?
        //
        // Thread the turtle through. Per type:
        //
        //  - Start / End: forward by TRACK_TIPS_LENGTH.
        //  - Straight: forward by its length.
        //  - NinetyDegTurn: a square corner, so the exit is offset on *both* axes by
        //    `line_half_length`, and then rotated 90 degrees. Mind the sign.
        //  - CyrcleTurn: the chord of the arc, then rotate by the arc angle. This is
        //    the only real trigonometry in the workshop - derive it. For an arc of
        //    radius r swept through angle a, starting along +Y with the centre off
        //    to the side, work out the displacement of the end point relative to the
        //    start, then apply `side.sign()`.
        //
        // Right now nothing advances, so the whole track is stacked at the origin.
        origin
    }

    pub fn spawn(
        &self,
        path_parent: Entity,
        line_parent: Entity,
        origin: SegmentTransform,
        features: EntityFeatures,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        let segment = *self;
        let transform = self.transform(origin);
        if features.has_physics() {
            commands.spawn_scene(bsn! {
                template(move |_| Ok(segment))
                ChildOf(path_parent)
                template_value(transform)
                template_value(self.collider())
                template_value(RigidBody::Fixed)
                Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                }
            });
        }
        if features.has_visualization() {
            let mesh = meshes.add(self.mesh());
            let material = materials.add(Color::srgba(0.0, 0.0, 0.0, 1.0));
            commands.spawn_scene(bsn! {
                template(move |_| Ok(segment))
                ChildOf(line_parent)
                template_value(transform)
                Mesh3d(mesh)
                MeshMaterial3d::<StandardMaterial>(material)
            });
        }
    }
}

#[derive(Clone, Resource)]
pub struct Track {
    size: Vec2,
    origin: SegmentTransform,
    segments: Vec<TrackSegment>,
}

impl Track {
    pub fn new(size: Vec2, origin: SegmentTransform, segments: Vec<TrackSegment>) -> Self {
        let origin = SegmentTransform {
            position: origin.position + Vec2::NEG_Y * TRACK_TIPS_LENGTH / 2.0,
            direction: origin.direction,
        };
        Self {
            size,
            origin,
            segments,
        }
    }

    pub fn camera_target(&self) -> Vec3 {
        -self.origin.position.extend(0.0)
    }

    pub fn camera_radius(&self) -> f32 {
        self.size.x.max(self.size.y) * 1.25
    }

    pub fn spawn_bundles(
        &self,
        path_parent: Entity,
        line_parent: Entity,
        features: EntityFeatures,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        let mut segment_origin = self.origin;

        for segment in &self.segments {
            segment.spawn(
                path_parent,
                line_parent,
                segment_origin,
                features,
                commands,
                meshes,
                materials,
            );
            segment_origin = segment.compute_next_origin(segment_origin);
        }
    }
}

pub fn setup_track(
    commands: &mut Commands,
    track_root: Entity,
    features: EntityFeatures,
    track: &Track,
    is_bottom: bool,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let bottom_x = -track.origin.position.x + TRACK_ORIGIN_OFFSET.x;
    let bottom_y = -track.origin.position.y + TRACK_ORIGIN_OFFSET.y;
    //let bottom_rot = Quat::from_rotation_z(track.origin.direction.to_radians());
    let bottom_rot = Quat::default();

    let at_height =
        move |z: f32| Transform::from_xyz(bottom_x, bottom_y, z).with_rotation(bottom_rot);

    let track_path_root = commands
        .spawn_scene(bsn! {
            ChildOf(track_root)
            template_value(at_height(-FLOOR_HEIGHT))
        })
        .id();

    let floor_physics = features.has_physics().then(|| {
        bsn! {
            template_value({
                Collider::cuboid(track.size.x / 2.0, track.size.y / 2.0, FLOOR_HEIGHT / 2.0)
            })
            template_value(RigidBody::Fixed)
            Friction { coefficient: 0.5 }
        }
    });
    commands.spawn_scene(bsn! {
        ChildOf(track_root)
        template_value(at_height(-FLOOR_HEIGHT / 2.0))
        {floor_physics}
    });

    let track_line_root = commands
        .spawn_scene(bsn! {
            ChildOf(track_root)
            template_value(at_height(0.001))
        })
        .id();

    if features.has_visualization() {
        let mesh = meshes.add(quad_mesh(track.size.x, track.size.y));
        let alpha = if is_bottom { 1.0 } else { 0.1 };
        let material = materials.add(Color::srgba(1.0, 1.0, 1.0, alpha));

        commands.spawn_scene(bsn! {
            ChildOf(track_root)
            template_value(at_height(0.0))
            Mesh3d(mesh)
            MeshMaterial3d::<StandardMaterial>(material)
            NotShadowCaster
        });
    }

    if !is_bottom || features.has_physics() {
        track.spawn_bundles(
            track_path_root,
            track_line_root,
            features,
            commands,
            meshes,
            materials,
        );
    }
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
        app.add_systems(
            Startup,
            move |mut commands: Commands,
                  track: Res<Track>,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>| {
                let track_root = commands.spawn_scene(bsn! { Transform }).id();
                setup_track(
                    &mut commands,
                    track_root,
                    features,
                    &track,
                    true,
                    &mut meshes,
                    &mut materials,
                )
            },
        );
    }
}
