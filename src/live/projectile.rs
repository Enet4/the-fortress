use bevy::{prelude::*, window::PrimaryWindow};

use super::Player;

/// Component for things which fly at a fixed speed
#[derive(Debug, Component)]
pub struct Velocity(Vec3);

pub fn apply_velocity(time: Res<Time>, mut q: Query<(&mut Transform, &Velocity)>) {
    let delta = time.delta_seconds();
    for (mut transform, velocity) in q.iter_mut() {
        transform.translation += Vec3::new(velocity.0.x, velocity.0.y, velocity.0.z) * delta;
    }
}

/// Marker for a projectile
#[derive(Debug, Component)]
pub struct Projectile;

pub fn spawn_projectiles_via_mouseclick(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_player: Query<&Transform, With<Player>>,
) {
    let Ok(window) = q_windows.get_single() else {
        return;
    };
    if buttons.just_pressed(MouseButton::Left) {
        let _cursor_position = window.cursor_position();

        // get the player's position
        let Ok(player_transform) = q_player.get_single() else {
            return;
        };

        let pos = player_transform.translation + Vec3::new(0.18, -0.85, 0.5);

        cmd.spawn((
            Projectile,
            PbrBundle {
                transform: Transform::from_translation(pos),
                mesh: meshes.add(Sphere::new(0.25)).into(),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgba_u8(255, 255, 128, 200),
                    emissive: LinearRgba::new(1., 1., 0.825, 0.75),
                    emissive_exposure_weight: 0.85,
                    ..Default::default()
                }),
                ..default()
            },
            Velocity(Vec3::new(0.0, 0.0, 25.0)),
        ));
    }
}
