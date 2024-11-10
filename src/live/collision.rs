//! Simple collision components
use bevy::{math::bounding::Aabb3d, prelude::*};

#[derive(Debug, Component)]
pub struct Collidable {
    /// the bounding box dimensions
    pub dim: Vec3,
}

impl Collidable {
    pub fn from_dimensions(dim: Vec3) -> Self {
        Self { dim }
    }

    pub fn to_bound(&self, pos: Vec3) -> Aabb3d {
        Aabb3d::new(pos, self.dim / 2.)
    }
}

impl Default for Collidable {
    fn default() -> Self {
        Self::from_dimensions(Vec3::new(1., 1., 1.))
    }
}
