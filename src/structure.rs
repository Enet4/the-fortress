//! Static structures

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
    PickableBundle,
};

use crate::live::{callback_on_click, collision::CollidableBox, OnLive};

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
        CollidableBox::new(Vec3::new(0.1, dim.x, dim.y)),
        PickableBundle::default(),
        On::<Pointer<Click>>::run(callback_on_click),
    )
}

/// Component describing the corridor and its dimensions
/// (Z is forward, Y is up, X is right-strafe)
#[derive(Debug, Component)]
pub struct Corridor {
    pub dim: Vec3,
}

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
        OnLive,
        Corridor { dim },
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
                transform: Transform::from_translation(Vec3::new(0., 0., corridor_half_dim.z)),
                mesh: meshes
                    .add(Plane3d {
                        half_size: Vec2::new(corridor_half_dim.x, corridor_half_dim.z),
                        normal: Dir3::Y,
                    })
                    .into(),
                material: floor_material_handle,
                ..Default::default()
            },
            CollidableBox::new(Vec3::new(dim.x, 0.25, dim.z)),
            PickableBundle::default(),
            On::<Pointer<Click>>::run(callback_on_click),
        ));

        // add ceiling
        cmd.spawn((
            PbrBundle {
                transform: Transform::from_translation(Vec3::new(0., dim.y, corridor_half_dim.z)),
                mesh: meshes
                    .add(Plane3d {
                        half_size: Vec2::new(corridor_half_dim.x, corridor_half_dim.z),
                        normal: Dir3::NEG_Y,
                    })
                    .into(),
                material: ceil_material_handle,
                ..Default::default()
            },
            CollidableBox::new(Vec3::new(dim.x, 0.125, dim.z)),
            PickableBundle::default(),
            On::<Pointer<Click>>::run(callback_on_click),
        ));

        // add some walls around the floor
        cmd.spawn(new_wall(
            &mut *meshes,
            wall_material_handle.clone(),
            Vec2::new(dim[1], dim[2]),
            Vec3::new(
                -corridor_half_dim.x,
                corridor_half_dim.y,
                corridor_half_dim.z,
            ),
            Dir3::X,
        ));
        cmd.spawn(new_wall(
            &mut *meshes,
            wall_material_handle.clone(),
            Vec2::new(dim[1], dim[2]),
            Vec3::new(
                corridor_half_dim.x,
                corridor_half_dim.y,
                corridor_half_dim.z,
            ),
            Dir3::NEG_X,
        ));
    });
    corridor
}

/// Marker component for a corridor fork
/// (to go either left of right)
#[derive(Debug, Component)]
pub struct Fork;

/// spawn a front wall, floor, and ceiling
/// according to the given properties
pub fn spawn_fork<'a>(
    cmd: &'a mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    floor_material_handle: Handle<StandardMaterial>,
    ceil_material_handle: Handle<StandardMaterial>,
    wall_material_handle: Handle<StandardMaterial>,
    pos: Vec3,
    dim: Vec3,
) -> EntityCommands<'a> {
    let half_dim = dim / 2.;
    let mut fork = cmd.spawn((
        OnLive,
        Fork,
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
    fork.with_children(|cmd| {
        // add floor
        cmd.spawn((
            PbrBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., half_dim.z)),
                mesh: meshes
                    .add(Plane3d {
                        half_size: Vec2::new(dim.x, half_dim.z),
                        normal: Dir3::Y,
                    })
                    .into(),
                material: floor_material_handle,
                ..Default::default()
            },
            CollidableBox::new(Vec3::new(dim.x * 2., 0.125, dim.z)),
            PickableBundle::default(),
            On::<Pointer<Click>>::run(callback_on_click),
        ));

        // add ceiling
        cmd.spawn((
            PbrBundle {
                transform: Transform::from_translation(Vec3::new(0., dim.y, half_dim.z)),
                mesh: meshes
                    .add(Plane3d {
                        half_size: Vec2::new(dim.x, half_dim.z),
                        normal: Dir3::NEG_Y,
                    })
                    .into(),
                material: ceil_material_handle,
                ..Default::default()
            },
            CollidableBox::new(Vec3::new(dim.x, 0.25, dim.z)),
            PickableBundle::default(),
            On::<Pointer<Click>>::run(callback_on_click),
        ));

        // add front wall
        cmd.spawn(new_wall(
            &mut *meshes,
            wall_material_handle.clone(),
            Vec2::new(dim[0] * 2., dim[1]),
            Vec3::new(0., half_dim.y, dim.z),
            Dir3::NEG_Z,
        ));
    });
    fork
}
