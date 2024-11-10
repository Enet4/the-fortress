use bevy::{
    prelude::*,
    render::{
        camera::Exposure,
        texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    },
};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::{On, Pickable},
    PickableBundle,
};

use crate::{
    effect::{Glimmers, Wobbles},
    live::{callback_on_click, collision::Collidable, spawn_player, Target},
    postprocess::PostProcessSettings,
};

mod structure;

fn repeat_texture(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..Default::default()
    });
}

/// set up the 3D scene
pub fn setup_scene(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wall_texture_handle =
        asset_server.load_with_settings("Brick 23 - 128x128.png", repeat_texture);
    let floor_texture_handle =
        asset_server.load_with_settings("Tile 9 - 128x128.png", repeat_texture);
    let ceil_texture_handle =
        asset_server.load_with_settings("Wood 16 - 128x128.png", repeat_texture);

    let corridor_dim = Vec3::from_array([10., 10., 64.]);

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
        ..Default::default()
    });

    structure::spawn_corridor(
        &mut cmd,
        &mut meshes,
        floor_material_handle,
        ceil_material_handle,
        wall_material_handle,
        Vec3::default(),
        corridor_dim,
    );

    // test: add a cube

    let test_cube_dim = Vec3::from_array([2., 4., 2.]);

    cmd.spawn((
        PbrBundle {
            transform: Transform::from_translation(Vec3::new(2., 2., -16.)),
            mesh: meshes.add(Cuboid::from_size(test_cube_dim)).into(),
            material: materials.add(StandardMaterial {
                base_color: Color::srgba_u8(255, 0, 0, 255),
                ..Default::default()
            }),
            ..Default::default()
        },
        PickableBundle {
            pickable: Pickable {
                is_hoverable: false,
                should_block_lower: true,
            },
            ..default()
        },
        Collidable { dim: test_cube_dim },
        Target {
            num: 1.into(),
            ..default()
        },
        On::<Pointer<Click>>::run(callback_on_click),
    ));

    // add the player, attach a camera to it, then add a light to the camera
    spawn_player(&mut cmd, Vec3::new(0., 2.5, -corridor_dim.z / 2. + 4.)).with_children(|cmd| {
        // wobbly pivot point for the camera and light
        cmd.spawn((
            TransformBundle::default(),
            VisibilityBundle::default(),
            Wobbles::default(),
        ))
        .with_children(|cmd| {
            // camera
            cmd.spawn((
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
                        start: 56.,
                        end: 64.,
                    },
                    ..default()
                },
                PostProcessSettings::default(),
            ))
            .with_children(|cmd| {
                // light
                cmd.spawn((
                    PointLightBundle {
                        point_light: PointLight {
                            color: Color::WHITE,
                            shadows_enabled: true,
                            intensity: 4_500_000.,
                            range: 64.,
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
}
