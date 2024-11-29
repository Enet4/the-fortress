//! Module for the cheat code implementation
//!
use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
};

use crate::{
    live::{CurrentLevel, Decision, LiveState},
    AppState,
};

/// Resource for long-lasting cheat effects
#[derive(Debug, Default, Resource)]
pub struct Cheats {
    pub used_cheats: bool,
    pub invulnerability: bool,
}

/// Global resource for the things the player types in
#[derive(Debug, Default, Resource)]
pub struct TextBuffer {
    buffer: String,
}

impl TextBuffer {
    const MAX_CAPACITY: usize = 32;

    pub fn has_typed(&self, cheat: &str) -> bool {
        self.buffer.ends_with(cheat)
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn push(&mut self, char: char) {
        self.buffer.push(char);
        if self.buffer.len() > Self::MAX_CAPACITY {
            // this is OK because the string is always ASCII
            self.buffer.remove(0);
        }
    }
}

/// system to accumulate keypresses into the text buffer
/// and check for cheat codes
pub fn cheat_input(
    mut text_buffer: ResMut<TextBuffer>,
    mut keyboard_input: EventReader<KeyboardInput>,
    current_level: ResMut<CurrentLevel>,
    app_state: Res<State<AppState>>,
    next_state: ResMut<NextState<LiveState>>,
    cheats: ResMut<Cheats>,
) {
    let mut has_presses = false;
    for ev in keyboard_input.read() {
        let KeyboardInput {
            logical_key, state, ..
        } = ev;
        if *state == ButtonState::Pressed {
            if let Key::Character(c) = logical_key {
                // only consider ASCII alphabetic characters
                // (cheat codes only have letters)
                if c.len() == 1 {
                    let c = c.to_ascii_lowercase().chars().next().unwrap();
                    if c.is_ascii_alphabetic() {
                        text_buffer.push(c);
                    }
                }
            }
            has_presses = true;
        }
    }
    if has_presses {
        check_cheat(text_buffer, cheats, current_level, app_state, next_state);
    }
}

fn check_cheat(
    mut text_buffer: ResMut<TextBuffer>,
    mut cheats: ResMut<Cheats>,
    mut current_level: ResMut<CurrentLevel>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<LiveState>>,
) {
    if text_buffer.has_typed("iddqd") {
        cheats.invulnerability = !cheats.invulnerability;
        if cheats.invulnerability {
            println!("Cheat code activated: invulnerability");
        } else {
            println!("Cheat code deactivated: invulnerability");
        }
        cheats.used_cheats = true;
        text_buffer.clear();
    } else if text_buffer.has_typed("nothingleftforme") {
        if *app_state.get() == AppState::Live {
            println!("Cheat code activated: next level by going left");
            if current_level.advance(Decision::Left) {
                next_state.set(LiveState::LoadingLevel);
            }
            cheats.used_cheats = true;
            text_buffer.clear();
        }
        text_buffer.clear();
    } else if text_buffer.has_typed("thisisdownrightridiculous") {
        println!("Cheat code activated: next level by going right");
        if *app_state.get() == AppState::Live {
            if current_level.advance(Decision::Right) {
                next_state.set(LiveState::LoadingLevel);
            }
            cheats.used_cheats = true;
            text_buffer.clear();
        }
    }
}
