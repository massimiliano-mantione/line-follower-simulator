use bevy::math::{EulerRot, Vec2, Vec3};
use bevy::transform::components::GlobalTransform;
use execution_data::MotorDriversDutyCycles;

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
