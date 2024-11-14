use bevy::prelude::*;
use bevy_mod_picking::events::{Click, Pointer};
use bevy_mod_picking::prelude::*;

use super::{callback_on_click, collision::Collidable, Target};

#[derive(Bundle)]
pub struct SimpleTargetBundle {
    #[bundle()]
    pub pbr: PbrBundle,
    pub pickable: PickableBundle,
    pub collidable: Collidable,
    pub target: Target,
    pub on_click: On<Pointer<Click>>,
}

impl SimpleTargetBundle {
    pub fn new(pbr: PbrBundle, collidable: Collidable, target: Target) -> Self {
        Self {
            pbr,
            pickable: PickableBundle {
                pickable: Pickable {
                    is_hoverable: false,
                    should_block_lower: true,
                },
                ..Default::default()
            },
            collidable,
            target,
            on_click: On::<Pointer<Click>>::run(callback_on_click),
        }
    }

    pub fn new_test_cube(
        position: Vec3,
        dim: Vec3,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
    ) -> Self {

        let pbr = PbrBundle {
            transform: Transform::from_translation(position),
            mesh,
            material,
            ..Default::default()
        };

        let target = Target {
            num: 1.into(),
            ..Default::default()
        };

        Self::new(pbr, Collidable { dim }, target)
    }
}

impl Default for SimpleTargetBundle {
    fn default() -> Self {
        Self::new(default(), default(), default())
    }
}
