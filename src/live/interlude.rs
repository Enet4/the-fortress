//! Module for the interlude message mechanism,
//! which is used to display story, explanations, and dialog to the player.

use bevy::prelude::*;

use crate::AppState;

use super::{phase::PhaseTrigger, player::Player, LiveState};

/// Complete specification for an interlude,
/// also serving as a marker for the interlude top UI node.
#[derive(Debug, Clone, Component)]
pub struct InterludeSpec {
    pub message: String,
    pub image: Option<Handle<Image>>,
    pub effect: InterludeEffect,
}

impl InterludeSpec {
    pub fn from_sequence<I, T>(seq: I) -> Self
    where
        I: IntoIterator<Item = (T, Option<Handle<Image>>)>,
        T: Into<String>,
    {
        Self::from_sequence_impl(seq, false).expect("interlude sequence must not be empty")
    }

    pub fn from_sequence_and_exit<I, T>(seq: I) -> Self
    where
        I: IntoIterator<Item = (T, Option<Handle<Image>>)>,
        T: Into<String>,
    {
        Self::from_sequence_impl(seq, true).expect("interlude sequence must not be empty")
    }

    fn from_sequence_impl<I, T>(seq: I, exit: bool) -> Option<Self>
    where
        I: IntoIterator<Item = (T, Option<Handle<Image>>)>,
        T: Into<String>,
    {
        let mut seq = seq.into_iter();
        let (message, image) = seq.next()?;

        match (Self::from_sequence_impl(seq, exit), exit) {
            (None, false) => Some(Self::new_text_single(message)),
            (None, true) => Some(Self {
                message: message.into(),
                image,
                effect: InterludeEffect::Exit,
            }),
            (Some(next), _) => Some(Self {
                message: message.into(),
                image,
                effect: InterludeEffect::Next(Box::new(next)),
            }),
        }
    }

    pub fn new_text(message: impl Into<String>, effect: InterludeEffect) -> Self {
        Self {
            message: message.into(),
            image: None,
            effect,
        }
    }

    pub fn new_text_single(message: impl Into<String>) -> Self {
        Self::new_text(message, InterludeEffect::Resume)
    }
}

/// What happens after an interlude is advanced.
#[derive(Debug, Default, Clone)]
pub enum InterludeEffect {
    /// Show the next interlude
    Next(Box<InterludeSpec>),
    /// Resume the game
    #[default]
    Resume,
    /// Return to the main menu
    Exit,
}

/// Marker component for a sub-node of the interlude UI
#[derive(Debug, Component)]
pub struct InterludePiece;

#[derive(Debug, Component)]
pub struct FadeIn;

#[derive(Debug, Component)]
pub struct FadeOut;

pub fn spawn_interlude(cmd: &mut Commands, spec: InterludeSpec) -> Entity {
    let message = spec.message.clone();
    let image = spec.image.clone();
    cmd.spawn((
        spec,
        NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                padding: UiRect::axes(Val::Px(72.), Val::Px(48.)),
                ..default()
            },
            background_color: Color::BLACK.into(),
            border_color: Color::WHITE.into(),
            border_radius: BorderRadius::all(Val::Px(2.)),
            ..default()
        },
    ))
    .with_children(|cmd| {
        // inner node for the border
        cmd.spawn((
            InterludePiece,
            NodeBundle {
                style: Style {
                    border: UiRect::all(Val::Px(2.)),
                    padding: UiRect {
                        top: Val::Px(40.),
                        bottom: Val::Px(20.),
                        left: Val::Px(20.),
                        right: Val::Px(20.),
                    },
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                border_color: BorderColor(Color::WHITE),
                ..default()
            },
        ))
        .with_children(|cmd| {
            // message node
            cmd.spawn((
                InterludePiece,
                TextBundle {
                    style: Style {
                        height: Val::Percent(100.),
                        ..default()
                    },
                    text: Text {
                        justify: JustifyText::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                        sections: vec![TextSection {
                            value: message.into(),
                            style: TextStyle {
                                font_size: 30.,
                                // start invisible, will move up through a system
                                color: Color::srgba(1., 1., 1., 0.),
                                font: Default::default(),
                            },
                        }],
                    },
                    ..default()
                },
            ));
            // if there is an image, add it
            if let Some(image) = image {
                cmd.spawn((
                    InterludePiece,
                    ImageBundle {
                        style: Style {
                            margin: UiRect {
                                left: Val::Px(10.),
                                right: Val::Px(10.),
                                top: Val::Px(10.),
                                bottom: Val::Px(10.),
                                ..default()
                            },
                            height: Val::Percent(100.),
                            ..default()
                        },
                        image: UiImage {
                            // start invisible, will move up through a system
                            color: Color::srgba(1., 1., 1., 0.),
                            texture: image,
                            ..default()
                        },
                        ..default()
                    },
                ));
            }
        });
    })
    .id()
}

pub fn process_interlude_trigger(
    mut cmd: Commands,
    trigger_q: Query<(Entity, &InterludeSpec, &PhaseTrigger)>,
    player_q: Query<&Transform, With<Player>>,
    mut next_state: ResMut<NextState<LiveState>>,
) {
    let Ok(player_transform) = player_q.get_single() else {
        return;
    };

    for (entity, spec, trigger) in trigger_q.iter() {
        if trigger.should_trigger(&player_transform.translation) {
            // spawn the interlude
            spawn_interlude(&mut cmd, spec.clone());
            // despawn the trigger
            cmd.entity(entity).despawn();
            // issue state transition
            next_state.set(LiveState::ShowingInterlude);
        }
    }
}

/// An event made to advance the interlude.
/// The event carries the entity of the previous interlude
/// and the effect which should be applied next.
#[derive(Debug, Event)]
pub struct AdvanceInterlude(Entity, InterludeEffect);

/// system that detects a click and moves forward in the interlude
pub fn on_click_advance_interlude(
    mut cmd: Commands,
    on_click: Res<ButtonInput<MouseButton>>,
    interlude_q: Query<(Entity, &InterludeSpec)>,
    interlude_pieces_q: Query<(Entity, Has<FadeOut>), With<InterludePiece>>,
    mut advance_event: EventWriter<AdvanceInterlude>,
) {
    if !on_click.just_pressed(MouseButton::Left) {
        return;
    }

    for (entity, has_fadeout) in interlude_pieces_q.iter() {
        // add fade-out if it does not exist yet
        if !has_fadeout {
            cmd.entity(entity).insert(FadeOut);
        } else {
            // fetch interlude spec and advance immediately
            if let Ok((e, spec)) = interlude_q.get_single() {
                // removing the fade-out is important,
                // otherwise it may try to advance the same interlude twice
                cmd.entity(entity).remove::<FadeOut>();
                advance_event.send(AdvanceInterlude(e, spec.effect.clone()));
            }
        }
    }
}

/// make interlude content fade in from black
pub fn fade_in_interlude(
    time: Res<Time>,
    mut cmd: Commands,
    mut text_q: Query<(Entity, &mut Text), Without<FadeIn>>,
    mut image_q: Query<(Entity, &mut UiImage), With<FadeIn>>,
) {
    let delta = time.delta_seconds();
    for (entity, mut text) in text_q.iter_mut() {
        for section in text.sections.iter_mut() {
            let new_alpha = (section.style.color.alpha() + delta * 1.25).min(1.);
            section.style.color.set_alpha(new_alpha);
            if new_alpha == 1. {
                cmd.entity(entity).remove::<FadeIn>();
            }
        }
    }
    for (entity, mut image) in image_q.iter_mut() {
        let new_alpha = (image.color.alpha() + delta * 2.).min(1.);
        image.color.set_alpha(new_alpha);
        if new_alpha == 1. {
            cmd.entity(entity).remove::<FadeIn>();
        }
    }
}

/// system to slowly fade out interlude content before transitioning
pub fn fade_out_interlude(
    time: Res<Time>,
    interlude_q: Query<(Entity, &InterludeSpec)>,
    mut text_q: Query<&mut Text, With<FadeOut>>,
    mut image_q: Query<&mut UiImage, With<FadeOut>>,
    mut advance_event: EventWriter<AdvanceInterlude>,
) {
    let delta = time.delta_seconds();

    if let Ok(mut text) = text_q.get_single_mut() {
        for section in text.sections.iter_mut() {
            let new_alpha = (section.style.color.alpha() - delta * 2.).max(0.);
            section.style.color.set_alpha(new_alpha);

            if new_alpha == 0. {
                // time to transition
                let (e, spec) = interlude_q.single();
                advance_event.send(AdvanceInterlude(e, spec.effect.clone()));
            }
        }
    }
    if let Ok(mut image) = image_q.get_single_mut() {
        let new_alpha = (image.color.alpha() - delta * 2.).max(0.);
        image.color.set_alpha(new_alpha);
    }
}

pub fn process_advance_interlude(
    mut events: EventReader<AdvanceInterlude>,
    mut cmd: Commands,
    mut next_live_state: ResMut<NextState<LiveState>>,
    mut next_root_state: ResMut<NextState<AppState>>,
) {
    for event in events.read() {
        let AdvanceInterlude(entity, effect) = event;
        match effect {
            InterludeEffect::Next(next_spec) => {
                // despawn the current interlude
                // (using `get_entity` because there is a race condition
                // which might trigger this event more than once for the same step)
                if let Some(e_cmd) = cmd.get_entity(*entity) {
                    e_cmd.despawn_recursive();
                    // spawn the next interlude
                    spawn_interlude(&mut cmd, *next_spec.clone());
                }
            }
            InterludeEffect::Resume => {
                // despawn the current interlude
                cmd.entity(*entity).despawn_recursive();
                // issue a state transition back to live
                next_live_state.set(LiveState::Running);
            }
            InterludeEffect::Exit => {
                // despawn the current interlude
                cmd.entity(*entity).despawn_recursive();
                // issue state transition
                next_root_state.set(AppState::Menu);
            }
        }
        break;
    }
}
