use bevy::{core_pipeline::bloom::BloomSettings, prelude::*, render::camera::Exposure};

use crate::{
    assets::TextureHandles,
    effect::{Glimmers, Wobbles},
    live::obstacle::SimpleTargetBundle,
    postprocess::PostProcessSettings,
    CameraMarker,
};

use crate::structure;

use super::{
    interlude::InterludeSpec,
    phase::PhaseTrigger,
    player::spawn_player,
    spawn_target_icon,
    weapon::{spawn_weapon_cube, WeaponCubeAssets},
};

/// set up the main 3D scene
pub fn setup_scene(
    mut cmd: Commands,
    texture_handles: Res<TextureHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    weapon_cube_assets: Res<WeaponCubeAssets>,
) {
    let wall_texture_handle = texture_handles.wall.clone();
    let floor_texture_handle = texture_handles.floor.clone();
    let ceil_texture_handle = texture_handles.ceil.clone();

    let corridor_length = 72.;
    let corridor_dim = Vec3::from_array([12., 8., corridor_length]);

    let floor_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(floor_texture_handle.clone()),
        uv_transform: bevy::math::Affine2 {
            matrix2: Mat2::from_cols_array(&[corridor_dim.x / 4., 0., 0., corridor_dim.z / 4.]),
            ..Default::default()
        },
        ..Default::default()
    });

    let ceil_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(ceil_texture_handle.clone()),
        uv_transform: bevy::math::Affine2 {
            matrix2: Mat2::from_cols_array(&[corridor_dim.x / 4., 0., 0., corridor_dim.z / 4.]),
            ..Default::default()
        },
        ..Default::default()
    });

    let wall_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(255, 255, 255),
        base_color_texture: Some(wall_texture_handle.clone()),
        uv_transform: bevy::math::Affine2 {
            matrix2: Mat2::from_cols_array(&[0., corridor_dim.y / 4., corridor_dim.z / 4., 0.]),
            ..Default::default()
        },
        perceptual_roughness: 0.85,
        ..Default::default()
    });

    // add corridor
    structure::spawn_corridor(
        &mut cmd,
        &mut meshes,
        floor_material_handle,
        ceil_material_handle,
        wall_material_handle,
        Vec3::ZERO,
        corridor_dim,
    );

    let fork_dim = Vec3::from_array([12., 8., 8.]);

    // create new materials for the fork
    let floor_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(floor_texture_handle),
        uv_transform: bevy::math::Affine2 {
            matrix2: Mat2::from_cols_array(&[corridor_dim.x / 2., 0., 0., fork_dim.z / 4.]),
            ..Default::default()
        },
        ..Default::default()
    });

    let ceil_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(ceil_texture_handle),
        uv_transform: bevy::math::Affine2 {
            matrix2: Mat2::from_cols_array(&[fork_dim.x / 2., 0., 0., fork_dim.z / 4.]),
            ..default()
        },
        ..default()
    });

    let wall_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(255, 255, 255),
        base_color_texture: Some(wall_texture_handle),
        uv_transform: bevy::math::Affine2 {
            matrix2: Mat2::from_cols_array(&[fork_dim.x / 2., 0., 0., fork_dim.y / 4.]),
            ..default()
        },
        ..default()
    });

    // add fork at the end of the corridor
    structure::spawn_fork(
        &mut cmd,
        &mut meshes,
        floor_material_handle,
        ceil_material_handle,
        wall_material_handle,
        Vec3::new(0., 0., corridor_dim.z),
        fork_dim,
    );

    // add the player, attach a camera to it, then add a light to the camera
    spawn_player(&mut cmd, Vec3::new(0., 2.5, -5.0)).with_children(|cmd| {
        // wobbly pivot point for the camera and light
        cmd.spawn((
            TransformBundle::default(),
            VisibilityBundle::default(),
            Wobbles::default(),
        ))
        .with_children(|cmd| {
            // camera
            cmd.spawn((
                CameraMarker,
                IsDefaultUiCamera,
                Camera3dBundle {
                    camera: Camera {
                        clear_color: ClearColorConfig::Custom(Color::BLACK),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0., 0.5, 0.5))
                        .looking_to(Dir3::Z, Dir3::Y),
                    exposure: Exposure::default(),
                    ..default()
                },
                InheritedVisibility::HIDDEN,
                FogSettings {
                    color: Color::BLACK,
                    falloff: FogFalloff::Linear {
                        start: 64.,
                        end: 72.,
                    },
                    ..default()
                },
                BloomSettings::NATURAL,
                PostProcessSettings {
                    oscillate: 0.,
                    ..default()
                },
            ))
            .with_children(|cmd| {
                // light
                cmd.spawn((
                    PointLightBundle {
                        point_light: PointLight {
                            color: Color::srgba_u8(255, 255, 224, 255),
                            shadows_enabled: true,
                            intensity: 4_400_000.,
                            range: 62.,
                            shadow_depth_bias: 0.1,
                            ..default()
                        },
                        transform: Transform::from_xyz(0., 1., 4.0),
                        ..default()
                    },
                    Glimmers {
                        amplitude_min: 48.,
                        amplitude_max: 64.,
                    },
                ));
            });
        });
    });

    // add phase triggers

    // test: add a target cube
    let test_cube_dim = Vec3::from_array([2., 4., 2.]);
    let test_cube_entity = cmd
        .spawn(SimpleTargetBundle::new_test_cube(
            Vec3::new(2., 2., 12.),
            test_cube_dim,
            meshes.add(Cuboid::from_size(test_cube_dim)).into(),
            materials.add(StandardMaterial {
                base_color: Color::srgba_u8(255, 0, 0, 255),
                ..default()
            }),
        ))
        .id();

    spawn_target_icon(&mut cmd, test_cube_entity, 1.into());

    // test: add a weapon cube

    spawn_weapon_cube(
        &mut cmd,
        &weapon_cube_assets,
        materials,
        Vec3::new(0., 3., 22.),
        2.into(),
    );

    // test: add an interlude just before the fork

    cmd.spawn((
        PhaseTrigger::new_by_corridor(corridor_length, 0.85),
        InterludeSpec::from_sequence([
            (
                "At the end of the corridor, you see two possible paths.\n\nThere appear to be no distinct visual cues between the two.",
                None,
            ),
            ("Reluctantly, you follow your instinct and choose.", None),
        ]),
    ));
}
