use bevy::ecs::resource::Resource;
use bevy::math::{EulerRot, Vec2, Vec3};
use bevy::transform::components::GlobalTransform;
use execution_data::MotorDriversDutyCycles;
use rand::{Rng, SeedableRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn sign(&self) -> f32 {
        match self {
            Side::Left => 1.0,
            Side::Right => -1.0,
        }
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Left => write!(f, "L"),
            Side::Right => write!(f, "R"),
        }
    }
}

pub trait SetBySide<T: Copy> {
    fn set_by_side(&mut self, side: Side, value: T);
}

pub trait GetBySide<T: Copy> {
    fn get_by_side(&self, side: Side) -> T;
}

impl GetBySide<i16> for MotorDriversDutyCycles {
    fn get_by_side(&self, side: Side) -> i16 {
        match side {
            Side::Left => self.left,
            Side::Right => self.right,
        }
    }
}

/// An angle, stored internally in radians.
///
/// Replaces the `Angle` type that used to be re-exported by `bevy::text::cosmic_text`,
/// which Bevy no longer re-exports.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Angle(f32);

impl Angle {
    pub fn from_degrees(degrees: f32) -> Self {
        Self(degrees.to_radians())
    }

    pub fn from_radians(radians: f32) -> Self {
        Self(radians)
    }

    pub fn to_degrees(self) -> f32 {
        self.0.to_degrees()
    }

    pub fn to_radians(self) -> f32 {
        self.0
    }
}

/// Helper to rotate a Vec2 by angle in radians
/// # Arguments
/// * `v`     - The vector to rotate
/// * `angle` - The angle in radians
pub fn rotate_vec2(v: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

pub fn point_to_new_origin(point: Vec3, transform: &GlobalTransform) -> Vec2 {
    rotate_vec2(
        (point - transform.translation()).truncate(),
        -transform.rotation().to_euler(EulerRot::ZYX).0,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EntityFeatures {
    Physics,
    Visualization,
    PhysicsAndVisualization,
}

impl std::fmt::Display for EntityFeatures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityFeatures::Physics => write!(f, "Physics"),
            EntityFeatures::Visualization => write!(f, "Visualization"),
            EntityFeatures::PhysicsAndVisualization => write!(f, "PhysicsAndVisualization"),
        }
    }
}

impl EntityFeatures {
    pub fn has_physics(&self) -> bool {
        match self {
            EntityFeatures::Physics => true,
            EntityFeatures::Visualization => false,
            EntityFeatures::PhysicsAndVisualization => true,
        }
    }

    pub fn has_visualization(&self) -> bool {
        match self {
            EntityFeatures::Physics => false,
            EntityFeatures::Visualization => true,
            EntityFeatures::PhysicsAndVisualization => true,
        }
    }
}

/// A fast, deterministic generator of random numbers with normal distribution.
#[derive(Resource)]
pub struct NormalRandom {
    rng: rand::rngs::SmallRng,
}

impl NormalRandom {
    pub fn new() -> Self {
        NormalRandom {
            rng: rand::rngs::SmallRng::seed_from_u64(42),
        }
    }

    pub fn sample(&mut self) -> f32 {
        self.rng
            .sample::<f32, rand_distr::StandardNormal>(rand_distr::StandardNormal)
    }

    pub fn noisy_value(&mut self, value: f32, noise: f32) -> f32 {
        (self.sample() * noise) + value
    }
}
