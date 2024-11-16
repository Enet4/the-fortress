//! Global asset handles

use bevy::{
    prelude::*,
    render::texture::{
        ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
    },
};

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
