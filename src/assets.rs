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
    pub equipmentclick01: Handle<AudioSource>,
    pub dread: Handle<AudioSource>,
}

impl FromWorld for AudioHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let zipclick = asset_server.load("audio/zipclick.ogg");
        let equipmentclick01 = asset_server.load("audio/equipmentclick01.ogg");
        let dread = asset_server.load("audio/dread.ogg");

        AudioHandles {
            enabled: true,
            zipclick,
            equipmentclick01,
            dread,
        }
    }
}

impl AudioHandles {
    pub fn play_zipclick<'a>(&self, cmd: &'a mut Commands) -> Option<EntityCommands<'a>> {
        self.play_impl(cmd, &self.zipclick)
    }

    pub fn play_equipmentclick01<'a>(&self, cmd: &'a mut Commands) -> Option<EntityCommands<'a>> {
        self.play_impl(cmd, &self.equipmentclick01)
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
