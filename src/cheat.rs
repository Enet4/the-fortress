//! Module for the cheat code implementation
//!
use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
};

/// Resource for long-lasting cheat effects
#[derive(Debug, Default, Resource)]
pub struct Cheats {
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
        check_cheat(text_buffer, cheats);
    }
}

fn check_cheat(mut text_buffer: ResMut<TextBuffer>, mut cheats: ResMut<Cheats>) {
    if text_buffer.has_typed("iddqd") {
        cheats.invulnerability = true;
        println!("Cheat code activated: invulnerability");
        text_buffer.clear();
    }
}
