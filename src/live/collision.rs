//! Simple collision components
use bevy::{math::bounding::Aabb3d, prelude::*};

#[derive(Debug, Component)]
pub struct CollidableBox {
    /// the bounding box dimensions
    pub dim: Vec3,
}

impl CollidableBox {
    pub fn new(dim: Vec3) -> Self {
        Self { dim }
    }

    pub fn to_bound(&self, pos: Vec3) -> Aabb3d {
        Aabb3d::new(pos, self.dim / 2.)
    }
}

impl Default for CollidableBox {
    fn default() -> Self {
        Self::new(Vec3::new(1., 1., 1.))
    }
}
