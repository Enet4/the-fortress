use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::prelude::*;
use bevy_ui_anchor::{AnchorTarget, AnchorUiNode, HorizontalAnchor, VerticalAnchor};

use crate::{effect::TimeToLive, logic::Num};

use super::{callback_on_click, player::Player, OnLive};

/// Marker component for the UI node showing a number
#[derive(Debug, Component)]
pub struct IconNode;

/// Reverse entity reference for entities with an icon attached
#[derive(Debug, Component)]
pub struct HasIcon(pub Entity);

/// System to despawn things when they are marked to be deleted.
/// This can be used for collapsed targets
/// and for collected weapon cubes.
pub fn clear_icons_of_destroyed_things(
    mut cmd: Commands,
    weapon_cube_q: Query<&HasIcon, Added<TimeToLive>>,
    icon_q: Query<Entity, With<IconNode>>,
) {
    for has_icon in weapon_cube_q.iter() {
        if let Ok(icon_entity) = icon_q.get(has_icon.0) {
            cmd.entity(icon_entity).despawn_recursive();
        }
    }
}

/// system to adjust opacity of icon nodes
/// based on how far they are from the player
pub fn update_icon_opacity(
    player_q: Query<&Transform, With<Player>>,
    item_q: Query<(&Transform, &HasIcon)>,
    mut icon_q: Query<(&mut BackgroundColor, &Children), With<IconNode>>,
    mut icon_text_q: Query<&mut Text>,
) {
    let Ok(player_transform) = player_q.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (item_transform, has_icon) in &item_q {
        let item_pos = item_transform.translation;
        let distance = (player_pos.z - item_pos.z).abs();

        // the formula for the opacity
        let opacity_1_distance = 18.;
        let opacity_0_distance = 26.;
        let opacity =
            1. - (distance - opacity_1_distance) / (opacity_0_distance - opacity_1_distance);

        // get the icon node
        let icon_e = has_icon.0;
        if let Ok((mut bg_color, children)) = icon_q.get_mut(icon_e) {
            bg_color.0.set_alpha(opacity);

            // get the text node
            if let Ok(mut text) = icon_text_q.get_mut(children[0]) {
                text.sections[0].style.color.set_alpha(opacity);
            }
        }
    }
}

/// Spawn a node that shows the target number on top of the target
pub fn spawn_icon(cmd: &mut Commands, entity: Entity, num: Num, color: Color) -> Entity {
    // draw a circle
    let (icon_size, font_size) = if *num.denom() >= 10 {
        (54., 26.)
    } else if *num.denom() > 1 || num >= 100.into() {
        (48., 28.)
    } else {
        (42., 34.)
    };
    let icon = cmd
        .spawn((
            OnLive,
            IconNode,
            Pickable::IGNORE,
            NodeBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    margin: UiRect::all(Val::Auto),
                    width: Val::Px(icon_size),
                    height: Val::Px(icon_size),
                    ..default()
                },
                background_color: BackgroundColor(Color::BLACK),
                border_radius: BorderRadius::MAX,
                focus_policy: FocusPolicy::Pass,
                z_index: ZIndex::Global(-2),
                ..default()
            },
            AnchorUiNode {
                anchorwidth: HorizontalAnchor::Mid,
                anchorheight: VerticalAnchor::Mid,
                target: AnchorTarget::Entity(entity),
            },
            On::<Pointer<Click>>::run(callback_on_click),
        ))
        .with_children(|cmd| {
            // and draw the number in the circle
            cmd.spawn((
                TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        margin: UiRect::all(Val::Auto),
                        ..default()
                    },
                    text: Text::from_section(
                        num.to_string(),
                        TextStyle {
                            color,
                            font_size,
                            ..default()
                        },
                    ),
                    ..default()
                },
                Pickable::IGNORE,
            ));
        })
        .id();

    // attach the icon to the entity
    cmd.entity(entity).insert(HasIcon(icon));
    icon
}

/// Spawn a node that shows the target number on top of the target
pub fn spawn_target_icon(cmd: &mut Commands, entity: Entity, num: Num) -> Entity {
    spawn_icon(cmd, entity, num, Color::WHITE)
}
