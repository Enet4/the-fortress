//! Global asset handles

use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    render::texture::{
        ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
    },
    text::Font,
};

#[derive(Debug, Clone, Resource)]
pub struct DefaultFont(pub Handle<Font>);

impl FromWorld for DefaultFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let font = asset_server.load("font/LibreCaslonTextRegular.ttf");

        DefaultFont(font)
    }
}

#[derive(Debug, Resource)]
pub struct TextureHandles {
    pub wall: Handle<Image>,
    pub floor: Handle<Image>,
    pub ceil: Handle<Image>,
}

impl FromWorld for TextureHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let wall = asset_server.load_with_settings("Brick 23 - 128x128.png", repeat_texture);
        let floor = asset_server.load_with_settings("Tile 9 - 128x128.png", repeat_texture);
        let ceil = asset_server.load_with_settings("Wood 16 - 128x128.png", repeat_texture);

        TextureHandles { wall, floor, ceil }
    }
}

fn repeat_texture(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..Default::default()
    });
}

/// Global resource for audio handles
#[derive(Debug, Resource)]
pub struct AudioHandles {
    pub enabled: bool,
    pub zipclick: Handle<AudioSource>,
    pub pickup: Handle<AudioSource>,
    pub equipmentclick1: Handle<AudioSource>,
    pub fireball: Handle<AudioSource>,
    pub hit02: Handle<AudioSource>,
    pub hit37: Handle<AudioSource>,
    pub dread: Handle<AudioSource>,
}

impl FromWorld for AudioHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let zipclick = asset_server.load("audio/zipclick.ogg");
        let pickup = asset_server.load("audio/Picked Coin Echo.ogg");
        let equipmentclick1 = asset_server.load("audio/equipmentclick1.ogg");
        let fireball = asset_server.load("audio/fireball.ogg");
        let hit02 = asset_server.load("audio/hit02.ogg");
        let hit37 = asset_server.load("audio/hit37.ogg");
        let dread = asset_server.load("audio/dread.ogg");

        AudioHandles {
            enabled: true,
            zipclick,
            pickup,
            equipmentclick1,
            fireball,
            hit02,
            hit37,
            dread,
        }
    }
}

impl AudioHandles {
    pub fn play_zipclick<'a>(&self, cmd: &'a mut Commands) -> Option<EntityCommands<'a>> {
        self.play_impl(cmd, &self.zipclick)
    }

    pub fn play_pickup<'a>(&self, cmd: &'a mut Commands) -> Option<EntityCommands<'a>> {
        self.play_impl(cmd, &self.pickup)
    }

    pub fn play_equipmentclick1<'a>(&self, cmd: &'a mut Commands) -> Option<EntityCommands<'a>> {
        self.play_impl(cmd, &self.equipmentclick1)
    }

    pub fn play_fireball<'a>(&self, cmd: &'a mut Commands) -> Option<EntityCommands<'a>> {
        self.play_impl(cmd, &self.fireball)
    }

    pub fn play_hit02<'a>(&self, cmd: &'a mut Commands) -> Option<EntityCommands<'a>> {
        self.play_impl(cmd, &self.hit02)
    }

    pub fn play_hit37<'a>(&self, cmd: &'a mut Commands) -> Option<EntityCommands<'a>> {
        self.play_impl(cmd, &self.hit37)
    }

    pub fn play_dread<'a>(&self, cmd: &'a mut Commands) -> Option<EntityCommands<'a>> {
        self.play_impl(cmd, &self.dread)
    }

    fn play_impl<'a>(
        &self,
        cmd: &'a mut Commands,
        handle: &Handle<AudioSource>,
    ) -> Option<EntityCommands<'a>> {
        if !self.enabled {
            return None;
        }
        Some(cmd.spawn(AudioBundle {
            source: handle.clone(),
            ..default()
        }))
    }
}
