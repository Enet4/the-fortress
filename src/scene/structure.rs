//! Static structures

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
    PickableBundle,
};

use crate::live::{callback_on_click, collision::Collidable};

fn new_wall(
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    dim: Vec2,
    translation: Vec3,
    normal: Dir3,
) -> impl Bundle {
    (
        PbrBundle {
            transform: Transform::from_translation(translation),
            mesh: meshes
                .add(Plane3d {
                    half_size: dim / 2.,
                    normal,
                })
                .into(),
            material,
            ..Default::default()
        },
        Collidable::from_dimensions(Vec3::new(0.1, dim.x, dim.y)),
        PickableBundle::default(),
        On::<Pointer<Click>>::run(callback_on_click),
    )
}

/// Marker component for corridor
#[derive(Debug, Component)]
pub struct Corridor;

/// spawn walls, floor, and ceiling
/// according to the given properties
pub fn spawn_corridor<'a>(
    cmd: &'a mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    floor_material_handle: Handle<StandardMaterial>,
    ceil_material_handle: Handle<StandardMaterial>,
    wall_material_handle: Handle<StandardMaterial>,
    pos: Vec3,
    dim: Vec3,
) -> EntityCommands<'a> {
    let corridor_half_dim = dim / 2.;
    let mut corridor = cmd.spawn((
        Corridor,
        TransformBundle {
            local: Transform::from_translation(pos),
            ..Default::default()
        },
        VisibilityBundle {
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::VISIBLE,
            ..default()
        },
    ));
    corridor.with_children(|cmd| {
        // add floor
        cmd.spawn((
            PbrBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                mesh: meshes
                    .add(Plane3d {
                        half_size: Vec2::new(corridor_half_dim.x, corridor_half_dim.z),
                        normal: Dir3::Y,
                    })
                    .into(),
                material: floor_material_handle,
                ..Default::default()
            },
            Collidable::from_dimensions(Vec3::new(dim.x, 0.25, dim.z)),
            PickableBundle::default(),
            On::<Pointer<Click>>::run(callback_on_click),
        ));

        // add ceiling
        cmd.spawn((
            PbrBundle {
                transform: Transform::from_translation(Vec3::new(0., dim.y, 0.)),
                mesh: meshes
                    .add(Plane3d {
                        half_size: Vec2::new(corridor_half_dim.x, corridor_half_dim.z),
                        normal: -Dir3::Y,
                    })
                    .into(),
                material: ceil_material_handle,
                ..Default::default()
            },
            Collidable::from_dimensions(Vec3::new(dim.x, 0.25, dim.z)),
            PickableBundle::default(),
            On::<Pointer<Click>>::run(callback_on_click),
        ));

        // add some walls around the floor
        cmd.spawn(new_wall(
            &mut *meshes,
            wall_material_handle.clone(),
            Vec2::new(dim[0], dim[2]),
            Vec3::new(-corridor_half_dim.x, corridor_half_dim.y, 0.),
            Dir3::X,
        ));
        cmd.spawn(new_wall(
            &mut *meshes,
            wall_material_handle.clone(),
            Vec2::new(dim[0], dim[2]),
            Vec3::new(corridor_half_dim.x, corridor_half_dim.y, 0.),
            -Dir3::X,
        ));
    });
    corridor
}
