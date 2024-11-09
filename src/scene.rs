use bevy::{
    prelude::*,
    render::texture::{
        ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
    },
};

use crate::{
    effect::{Glimmers, Wobbles},
    live::{Health, Player},
    postprocess::PostProcessSettings,
};

fn spawn_wall(
    cmd: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    dim: Vec2,
    translation: Vec3,
    normal: Dir3,
) {
    cmd.spawn(PbrBundle {
        transform: Transform::from_translation(translation),
        mesh: meshes
            .add(Plane3d {
                half_size: dim / 2.,
                normal,
            })
            .into(),
        material,
        ..Default::default()
    });
}

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
    let corridor_half_dim = corridor_dim / 2.;

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
        fog_enabled: true,
        double_sided: true,
        uv_transform: bevy::math::Affine2 {
            matrix2: Mat2::from_cols_array(&[corridor_dim.x / 4., 0., 0., corridor_dim.z / 4.]),
            ..Default::default()
        },
        ..Default::default()
    });

    let wall_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(255, 255, 255),
        base_color_texture: Some(wall_texture_handle.clone()),
        fog_enabled: true,
        uv_transform: bevy::math::Affine2 {
            matrix2: Mat2::from_cols_array(&[0., corridor_dim.y / 4., corridor_dim.z / 4., 0.]),
            ..Default::default()
        },
        ..Default::default()
    });

    // add some floor
    cmd.spawn(PbrBundle {
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        mesh: meshes
            .add(Plane3d {
                half_size: Vec2::new(corridor_half_dim.x, corridor_half_dim.z),
                normal: Dir3::Y,
            })
            .into(),
        material: floor_material_handle,
        ..Default::default()
    });

    // add ceiling
    cmd.spawn(PbrBundle {
        transform: Transform::from_translation(Vec3::new(0., corridor_dim.y, 0.)),
        mesh: meshes
            .add(Plane3d {
                half_size: Vec2::new(corridor_half_dim.x, corridor_half_dim.z),
                normal: -Dir3::Y,
            })
            .into(),
        material: ceil_material_handle,
        ..Default::default()
    });

    // test: add a cube

    cmd.spawn(PbrBundle {
        transform: Transform::from_translation(Vec3::new(2., 2., -16.)),
        mesh: meshes.add(Cuboid::new(2., 4., 2.)).into(),
        material: materials.add(StandardMaterial {
            base_color: Color::srgba_u8(255, 0, 0, 255),
            ..Default::default()
        }),
        ..Default::default()
    });

    // add some walls around the floor
    spawn_wall(
        &mut cmd,
        &mut meshes,
        wall_material_handle.clone(),
        Vec2::new(corridor_dim[0], corridor_dim[2]),
        Vec3::new(-corridor_half_dim.x, corridor_half_dim.y, 0.),
        Dir3::X,
    );
    spawn_wall(
        &mut cmd,
        &mut meshes,
        wall_material_handle.clone(),
        Vec2::new(corridor_dim[0], corridor_dim[2]),
        Vec3::new(corridor_half_dim.x, corridor_half_dim.y, 0.),
        -Dir3::X,
    );

    // light
    let light = cmd.spawn((
        PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                shadows_enabled: true,
                intensity: 4_000_000.,
                range: 50.,
                shadow_depth_bias: 0.1,
                ..default()
            },
            transform: Transform::from_xyz(0., corridor_dim[1] - 2., 3.0),
            ..default()
        },
        Glimmers {
            amplitude_min: 48.,
            amplitude_max: 64.,
        },
    ));
    let light = light.id();

    // camera (which is also the position of the player's character)
    let mut camera = cmd.spawn((
        Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., 4., 2. - corridor_half_dim[2]))
                .looking_to(Dir3::Z, Dir3::Y),
            ..default()
        },
        InheritedVisibility::HIDDEN,
        FogSettings {
            color: Color::BLACK,
            falloff: FogFalloff::Linear {
                start: 48.,
                end: 60.,
            },
            ..default()
        },
        PostProcessSettings::default(),
        Wobbles::default(),
        Player,
        Health {
            value: 100.,
            max: 100.,
        },
    ));
    // attach light to camera
    camera.add_child(light);
}
