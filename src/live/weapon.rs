use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
};
use bevy_mod_picking::prelude::*;

use crate::{
    assets::AudioHandles,
    effect::{Rotating, TimeToLive, Velocity},
    logic::Num,
    postprocess::PostProcessSettings,
};

use super::{
    icon::spawn_icon,
    player::Player,
    projectile::{spawn_projectile, ProjectileAssets},
    OnLive, WeaponListNode,
};

/// Component representing a specific weapon in the player's arsenal.
/// (in practice it is always the staff, but the projectiles are different)
#[derive(Debug, Component)]
pub struct PlayerWeapon {
    /// the number representing the attack
    pub num: Num,
    /// projectile speed
    pub projectile_speed: f32,
    /// the amount of cooldown added per use
    pub cooldown: f32,
}

impl PlayerWeapon {
    pub fn new(num: Num) -> Self {
        Self {
            num,
            ..Default::default()
        }
    }
}

impl Default for PlayerWeapon {
    fn default() -> Self {
        Self {
            projectile_speed: 30.,
            num: 0.into(),
            cooldown: 1.,
        }
    }
}

pub fn install_weapon(cmd: &mut Commands, num: Num) {
    cmd.spawn((OnLive, PlayerWeapon::new(num)));
}

/// Marker component representing the weapon currently wielded by the player.
///
/// Can be used both in the weapon pool and the weapon button pool.
#[derive(Debug, Default, Component)]
pub struct WeaponSelected;

/// system that processes the addition of new weapons
pub fn process_new_weapon(
    mut cmd: Commands,
    weapon_q: Query<(Entity, &PlayerWeapon), Added<PlayerWeapon>>,
    mut weapon_list_node_q: Query<(Entity, Option<&Children>), With<WeaponListNode>>,
) {
    for (weapon_entity, weapon) in weapon_q.iter() {
        // add a new weapon to the list
        let (entity, weapon_buttons) = weapon_list_node_q
            .get_single_mut()
            .expect("No weapon list node found! This is likely a bug");

        let mut first = false;
        let shortcut = if let Some(weapon_buttons) = weapon_buttons {
            if weapon_buttons.is_empty() {
                // automatically select the first weapon
                cmd.entity(weapon_entity).insert(WeaponSelected);
                first = true;
            }

            weapon_buttons.len() as u8 + 1
        } else {
            // automatically select the first weapon
            cmd.entity(weapon_entity).insert(WeaponSelected);
            first = true;
            1
        };

        cmd.entity(entity).with_children(|root| {
            spawn_weapon_button(root, weapon.num, shortcut, first);
        });
    }
}

/// Component for implementing a timeout before
/// the next attack can be made by a player or mob.
#[derive(Debug, Component)]
pub struct AttackCooldown {
    /// the time to wait before the next attack, in seconds
    pub value: f32,
    /// the maximum cooldown, usually applied after an attack, in seconds
    pub max: f32,
    /// whether the weapon cannot be used (because it overheated)
    pub locked: bool,
}

impl Default for AttackCooldown {
    fn default() -> Self {
        Self {
            value: 0.,
            max: 2.,
            locked: false,
        }
    }
}

pub fn update_cooldown(time: Res<Time>, mut q: Query<&mut AttackCooldown>) {
    for mut cooldown in q.iter_mut() {
        cooldown.value -= time.delta_seconds();
        if cooldown.value <= 0. {
            cooldown.value = 0.;
            cooldown.locked = false;
        }
    }
}

/// An event fired when the player clicks on something to attack.
#[derive(Debug, Event)]
pub struct TriggerWeapon {
    pub target_pos: Vec3,
}

/// System that reacts to events for triggering the weapon.
pub fn trigger_weapon(
    mut cmd: Commands,
    projectile_assets: Res<ProjectileAssets>,
    audio_handles: Res<AudioHandles>,
    mut trigger_weapon_events: EventReader<TriggerWeapon>,
    mut weapon_q: Query<&PlayerWeapon, With<WeaponSelected>>,
    mut player_q: Query<(&GlobalTransform, &mut AttackCooldown), With<Player>>,
) {
    for trigger_weapon in trigger_weapon_events.read() {
        let Ok(weapon) = weapon_q.get_single_mut() else {
            return;
        };

        let (player_transform, mut cooldown) = player_q.single_mut();

        // if the weapon is locked, we cannot trigger it
        if cooldown.locked {
            continue;
        }

        let player_position = player_transform.translation();

        // play sound effect
        audio_handles.play_fireball(&mut cmd);

        let direction = trigger_weapon.target_pos - player_position;
        let direction = direction.normalize();

        // spawn a projectile
        spawn_projectile(
            &mut cmd,
            player_position,
            direction,
            weapon,
            &projectile_assets,
        );

        // apply cooldown
        cooldown.value = cooldown.value + weapon.cooldown;
        if cooldown.value >= cooldown.max {
            cooldown.value = cooldown.max;
            cooldown.locked = true;
        }
    }
}

/// An event fired when a player projectile hits a target.
#[derive(Debug, Event)]
pub struct PlayerAttack {
    /// the target entity hit by the attack
    pub entity: Entity,
    /// the number of the attack
    pub num: Num,
}

/// create a new button
pub fn spawn_weapon_button(
    cmd: &mut ChildBuilder<'_>,
    attack_num: Num,
    shortcut: u8,
    selected: bool,
) {
    let (back_color, front_color) = if selected {
        (Color::WHITE, Color::BLACK)
    } else {
        (Color::BLACK, Color::WHITE)
    };

    let bundle = (
        OnLive,
        WeaponButton {
            num: attack_num,
            shortcut,
        },
        Pickable {
            should_block_lower: true,
            is_hoverable: false,
        },
        ButtonBundle {
            background_color: BackgroundColor(back_color),
            border_color: BorderColor(front_color),
            style: Style {
                border: UiRect::all(Val::Px(1.)),
                display: Display::Flex,
                align_self: AlignSelf::Center,
                column_gap: Val::Px(10.),
                width: Val::Px(64.),
                height: Val::Px(64.),
                margin: UiRect::all(Val::Px(10.)),
                ..default()
            },
            ..default()
        },
    );

    if selected {
        cmd.spawn((bundle, WeaponSelected))
    } else {
        cmd.spawn(bundle)
    }
    // insert button
    .with_children(|parent| {
        // shortcut
        parent.spawn(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(1.),
                top: Val::Px(1.),
                ..default()
            },
            text: Text::from_section(
                shortcut.to_string(),
                TextStyle {
                    font_size: 14.,
                    color: front_color,
                    ..default()
                },
            ),
            ..Default::default()
        });

        // the actual number of the attack
        parent.spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::Center,
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            text: Text::from_section(
                attack_num.to_string(),
                TextStyle {
                    font_size: 36.,
                    color: front_color,
                    ..default()
                },
            ),
            ..default()
        });
    });
}

/// Component for a new weapon number to portrayed as a cube on the screen.
#[derive(Debug, Default, Component)]
pub struct WeaponCube {
    pub num: Num,
}

#[derive(Debug, Resource)]
pub struct WeaponCubeAssets {
    pub mesh: Handle<Mesh>,
}

impl FromWorld for WeaponCubeAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mesh = meshes.add(Mesh::from(Cuboid::from_length(0.75)));
        Self { mesh }
    }
}

pub fn spawn_weapon_cube(
    cmd: &mut Commands,
    assets: &WeaponCubeAssets,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    num: Num,
) -> Entity {
    let entity = cmd
        .spawn((
            OnLive,
            WeaponCube { num },
            Rotating(0.5),
            PbrBundle {
                transform: Transform::from_translation(position),
                mesh: assets.mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgba(1., 1., 1., 0.875),
                    ..default()
                }),
                ..default()
            },
        ))
        .id();

    // add an icon for it
    spawn_icon(cmd, entity, num, Color::srgb(0., 1., 1.));

    entity
}

pub fn process_approach_weapon_cube(
    mut cmd: Commands,
    player_q: Query<&Transform, With<Player>>,
    mut postprocess_settings_q: Query<&mut PostProcessSettings>,
    mut weapon_cube_q: Query<(Entity, &Transform, &WeaponCube, &mut Rotating)>,
) {
    let Ok(player_transform) = player_q.get_single() else {
        return;
    };
    let player_corridor_pos = player_transform.translation.z;

    for (entity, weapon_transform, weapon_cube, mut rotating) in weapon_cube_q.iter_mut() {
        let weapon_corridor_pos = weapon_transform.translation.z;
        let distance = (player_corridor_pos - weapon_corridor_pos).abs();

        if distance < 9.5 {
            // make an effect
            cmd.entity(entity).insert(Velocity(Vec3::new(0., 1., 0.)));
            // increase rotation speed
            rotating.0 = rotating.0 * 4.;
            // add time to live
            cmd.entity(entity).insert(TimeToLive(0.6));
            // remove weapon cube marker
            cmd.entity(entity).remove::<WeaponCube>();
            install_weapon(&mut cmd, weapon_cube.num);

            // add a visual effect
            if let Ok(mut settings) = postprocess_settings_q.get_single_mut() {
                settings.add_intensity(0.05);
            }
        }
    }
}

/// system to check keypresses for weapon shortcuts
pub fn weapon_keyboard_input(
    mut cmd: Commands,
    mut keyboard_input: EventReader<KeyboardInput>,
    weapon_button_q: Query<(Entity, &WeaponButton, Has<WeaponSelected>)>,
    mut change_weapon: EventWriter<ChangeWeapon>,
    audio_handles: Res<AudioHandles>,
) {
    for ev in keyboard_input.read() {
        let KeyboardInput {
            logical_key, state, ..
        } = ev;

        if let (Key::Character(c), ButtonState::Pressed) = (logical_key, state) {
            let Some(c) = c.chars().next() else { continue };
            if ('1'..='9').contains(&c) {
                let shortcut = (c as u8 - b'0') as u8;

                // look up each weapon button and update selection
                for (entity, weapon_button, is_selected) in &weapon_button_q {
                    if weapon_button.shortcut == shortcut {
                        if is_selected {
                            // no change is needed, stop here
                            break;
                        }
                        let num = weapon_button.num;
                        cmd.entity(entity).insert(WeaponSelected);
                        // perform weapon selection
                        change_weapon.send(ChangeWeapon { num });

                        // play sound
                        audio_handles.play_equipmentclick1(&mut cmd);
                        break;
                    } else {
                        cmd.entity(entity).remove::<WeaponSelected>();
                    }
                }
            }
        }
    }
}

/// Component for a weapon button
#[derive(Debug, Default, Component)]
pub struct WeaponButton {
    /// the attack number
    num: Num,
    /// an integer between 1-9, representing the shorcut button
    shortcut: u8,
}

/// system callback for when the player clicks on a weapon button
/// (as an alternative to using the shortcut keys)
pub fn weapon_button_action(
    mut cmd: Commands,
    mut interaction_query: Query<
        (Entity, &Interaction, &WeaponButton, Has<WeaponSelected>),
        Changed<Interaction>,
    >,
    mut weapon_button_q: Query<Entity, With<WeaponButton>>,
    mut events: EventWriter<ChangeWeapon>,
    audio_handles: Res<AudioHandles>,
) {
    for (entity, interaction, weapon_button, is_selected) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if is_selected {
            // already selected, do nothing
            continue;
        }

        // play sounds
        audio_handles.play_equipmentclick1(&mut cmd);

        // traverse all buttons to update the selected weapon
        for e in &mut weapon_button_q {
            if e == entity {
                // mark this one as selected
                cmd.entity(e).insert(WeaponSelected);
            } else {
                // unmark any other
                cmd.entity(e).remove::<WeaponSelected>();
            }
        }

        // change weapon
        events.send(ChangeWeapon {
            num: weapon_button.num,
        });
    }
}

/// Event which requests for a change in the selected weapon
#[derive(Debug, Event)]
pub struct ChangeWeapon {
    pub num: Num,
}

pub fn process_weapon_change(
    mut events: EventReader<ChangeWeapon>,
    mut weapon_q: Query<&mut PlayerWeapon, With<WeaponSelected>>,
) {
    for ChangeWeapon { num, .. } in events.read() {
        // update the weapon characteristics
        let Ok(mut weapon) = weapon_q.get_single_mut() else {
            return;
        };
        weapon.num = *num;
    }
}

/// system that updates the style of the selected button
pub fn process_weapon_button_selected(
    mut weapon_button_q: Query<
        (&mut BackgroundColor, &Children),
        (With<WeaponButton>, Added<WeaponSelected>),
    >,
    mut weapon_button_text_q: Query<&mut Text>,
) {
    for (mut background_color, children) in &mut weapon_button_q {
        background_color.0 = Color::WHITE;

        for child in children {
            let Ok(mut text) = weapon_button_text_q.get_mut(*child) else {
                continue;
            };
            for section in &mut text.sections {
                section.style.color = Color::BLACK;
            }
        }
    }
}

/// system that updates the style of the selected button
pub fn process_weapon_button_deselected(
    mut removals: RemovedComponents<WeaponSelected>,
    mut weapon_button_q: Query<(&mut BackgroundColor, &Children), With<WeaponButton>>,
    mut weapon_button_text_q: Query<&mut Text>,
) {
    for entity in removals.read() {
        // see if entity still exists
        let Ok((mut background_color, children)) = weapon_button_q.get_mut(entity) else {
            continue;
        };

        background_color.0 = Color::BLACK;

        for child in children {
            let Ok(mut text) = weapon_button_text_q.get_mut(*child) else {
                continue;
            };
            for section in &mut text.sections {
                section.style.color = Color::WHITE;
            }
        }
    }
}
