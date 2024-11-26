use bevy::prelude::*;
use bevy_mod_picking::events::{Click, Pointer};
use bevy_mod_picking::prelude::*;

use crate::effect::ScalesUp;

use super::OnLive;
use super::{callback_on_click, collision::CollidableBox, Target};

#[derive(Bundle)]
pub struct SimpleTargetBundle {
    #[bundle()]
    pub pbr: PbrBundle,
    pub pickable: PickableBundle,
    pub collidable: CollidableBox,
    pub target: Target,
    pub on_click: On<Pointer<Click>>,
    pub scales_up: ScalesUp,
    pub on_live: OnLive,
}

impl SimpleTargetBundle {
    pub fn new(pbr: PbrBundle, collidable: CollidableBox, target: Target) -> Self {
        Self {
            pbr,
            pickable: PickableBundle {
                pickable: Pickable {
                    is_hoverable: false,
                    should_block_lower: true,
                },
                ..default()
            },
            collidable,
            target,
            on_click: On::<Pointer<Click>>::run(callback_on_click),
            scales_up: default(),
            on_live: default(),
        }
    }

    pub fn new_test_cube(
        position: Vec3,
        dim: Vec3,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
    ) -> Self {
        let pbr = PbrBundle {
            transform: Transform::from_translation(position).with_scale(Vec3::splat(1e-3)),
            mesh,
            material,
            ..default()
        };

        let target = Target {
            num: 2.into(),
            ..default()
        };

        Self::new(pbr, CollidableBox { dim }, target)
    }
}
