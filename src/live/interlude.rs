//! Module for the interlude message mechanism,
//! which is used to display story, explanations, and dialog to the player.

use bevy::prelude::*;

use crate::{assets::DefaultFont, ui::Sizes, AppState, GameSettings};

use super::{phase::PhaseTrigger, player::Player, LiveState};

/// Complete specification for an interlude,
/// also serving as a marker for the interlude top UI node.
#[derive(Debug, Clone, Component)]
pub struct InterludeSpec {
    /// the text message in the interlude
    pub message: String,
    /// path to the image to present in this step
    pub image: Option<&'static str>,
    /// what should happen when the player dismisses this interlude
    pub effect: InterludeEffect,
}

impl InterludeSpec {
    pub fn from_sequence<I, T>(seq: I) -> Self
    where
        I: IntoIterator<Item = (T, Option<&'static str>)>,
        T: Into<String>,
    {
        Self::from_sequence_impl(seq, false).expect("interlude sequence must not be empty")
    }

    pub fn from_sequence_and_exit<I, T>(seq: I) -> Self
    where
        I: IntoIterator<Item = (T, Option<&'static str>)>,
        T: Into<String>,
    {
        Self::from_sequence_impl(seq, true).expect("interlude sequence must not be empty")
    }

    fn from_sequence_impl<I, T>(seq: I, exit: bool) -> Option<Self>
    where
        I: IntoIterator<Item = (T, Option<&'static str>)>,
        T: Into<String>,
    {
        let mut seq = seq.into_iter();
        let (message, image) = seq.next()?;

        match (Self::from_sequence_impl(seq, exit), exit) {
            (None, false) => Some(Self::new_single(message, image)),
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

    pub fn new_single(message: impl Into<String>, image: Option<&'static str>) -> Self {
        Self {
            message: message.into(),
            image,
            effect: InterludeEffect::Resume,
        }
    }

    /// whether reaching this interlude also means the end of the game
    pub fn is_exit(&self) -> bool {
        match &self.effect {
            InterludeEffect::Exit => true,
            InterludeEffect::Resume => false,
            InterludeEffect::Next(spec) => spec.is_exit(),
        }
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

pub fn spawn_interlude(
    cmd: &mut Commands,
    spec: InterludeSpec,
    default_font: &DefaultFont,
    sizes: &Sizes,
    asset_server: &AssetServer,
) -> Entity {
    let message = spec.message.clone();

    let image = spec.image.map(|path| asset_server.load(path));

    let font = &default_font.0;

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
                padding: UiRect::axes(
                    Val::Px(sizes.outer_padding_h),
                    Val::Px(sizes.outer_padding_v),
                ),
                ..default()
            },
            background_color: Color::BLACK.into(),
            border_color: Color::WHITE.into(),
            border_radius: BorderRadius::all(Val::Px(2.)),
            z_index: ZIndex::Global(9),
            ..default()
        },
    ))
    .with_children(|cmd| {
        // inner node for the border
        cmd.spawn((
            InterludePiece,
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
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
            // if there is an image, add it
            if let Some(image) = image {
                cmd.spawn((
                    InterludePiece,
                    FadeIn,
                    ImageBundle {
                        style: Style {
                            margin: UiRect {
                                left: Val::Px(10.),
                                right: Val::Px(10.),
                                top: Val::Px(10.),
                                bottom: Val::Px(10.),
                                ..default()
                            },
                            max_height: Val::Percent(100.),
                            max_width: Val::Percent(40.),
                            ..default()
                        },
                        image: UiImage {
                            // start invisible, will move up through a system
                            color: Color::srgba(1., 1., 1., 0.125),
                            texture: image,
                            ..default()
                        },
                        ..default()
                    },
                ));
            }

            // message node
            cmd.spawn((
                InterludePiece,
                FadeIn,
                TextBundle {
                    style: Style {
                        margin: UiRect::all(Val::Auto),
                        ..default()
                    },
                    text: Text {
                        justify: JustifyText::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                        sections: vec![TextSection {
                            value: message.into(),
                            style: TextStyle {
                                font: font.clone(),
                                font_size: sizes.interlude_font_size,
                                // start invisible, will move up through a system
                                color: Color::srgba(1., 1., 1., 0.125),
                            },
                        }],
                    },
                    ..default()
                },
            ));
        });
    })
    .id()
}

pub fn process_interlude_trigger(
    mut cmd: Commands,
    game_settings: Res<GameSettings>,
    trigger_q: Query<(Entity, &InterludeSpec, &PhaseTrigger)>,
    player_q: Query<&Transform, With<Player>>,
    mut next_state: ResMut<NextState<LiveState>>,
    asset_server: Res<AssetServer>,
    sizes: Res<Sizes>,
    default_font: Res<DefaultFont>,
) {
    let Ok(player_transform) = player_q.get_single() else {
        return;
    };

    for (entity, spec, trigger) in trigger_q.iter() {
        if trigger.should_trigger(&player_transform.translation) {
            // do not show interludes which just resume the game afterwards
            if game_settings.skip_interludes && !spec.is_exit() {
                continue;
            }

            // spawn the interlude
            spawn_interlude(&mut cmd, spec.clone(), &default_font, &sizes, &asset_server);
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
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    // should only fetch the interlude being presented,
    // hence `Without<PhaseTrigger>`
    interlude_q: Query<(Entity, &InterludeSpec), Without<PhaseTrigger>>,
    interlude_pieces_q: Query<(Entity, Has<FadeOut>), With<InterludePiece>>,
    mut advance_event: EventWriter<AdvanceInterlude>,
) {
    // advance on left mouse click, Enter, or tap
    if !mouse_button_input.just_pressed(MouseButton::Left)
        && !keyboard_input.just_pressed(KeyCode::Enter)
        && !touches.any_just_pressed()
    {
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
    mut text_q: Query<(Entity, &mut Text), With<FadeIn>>,
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
    // should only fetch the interlude being presented,
    // hence `Without<PhaseTrigger>`
    interlude_q: Query<(Entity, &InterludeSpec), Without<PhaseTrigger>>,
    mut text_q: Query<&mut Text, With<FadeOut>>,
    mut image_q: Query<&mut UiImage, With<FadeOut>>,
    mut advance_event: EventWriter<AdvanceInterlude>,
) {
    let delta = time.delta_seconds();

    let mut should_transition = false;
    if let Ok(mut text) = text_q.get_single_mut() {
        for section in text.sections.iter_mut() {
            let new_alpha = (section.style.color.alpha() - delta * 2.).max(0.);
            section.style.color.set_alpha(new_alpha);

            if new_alpha == 0. {
                // time to transition
                should_transition = true;
            }
        }
    }
    if let Ok(mut image) = image_q.get_single_mut() {
        let new_alpha = (image.color.alpha() - delta * 2.).max(0.);
        image.color.set_alpha(new_alpha);
    }

    if should_transition {
        let (e, spec) = interlude_q.single();
        advance_event.send(AdvanceInterlude(e, spec.effect.clone()));
    }
}

pub fn process_advance_interlude(
    mut events: EventReader<AdvanceInterlude>,
    mut cmd: Commands,
    mut next_live_state: ResMut<NextState<LiveState>>,
    mut next_root_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    sizes: Res<Sizes>,
    default_font: Res<DefaultFont>,
) {
    for event in events.read() {
        let AdvanceInterlude(entity, effect) = event;
        // despawn the current interlude
        // (using `get_entity` because there is a race condition
        // which might trigger this event more than once for the same step)
        if let Some(e_cmd) = cmd.get_entity(*entity) {
            e_cmd.despawn_recursive();

            match effect {
                InterludeEffect::Next(next_spec) => {
                    // spawn the next interlude
                    spawn_interlude(
                        &mut cmd,
                        *next_spec.clone(),
                        &default_font,
                        &sizes,
                        &asset_server,
                    );
                }
                InterludeEffect::Resume => {
                    // issue a state transition back to live
                    next_live_state.set(LiveState::Running);
                }
                InterludeEffect::Exit => {
                    // issue state transition back to menu
                    next_root_state.set(AppState::Menu);
                }
            }
        }
        break;
    }
}
