use crate::components::Collection;
use bevy::{
    prelude::*,
    render::{render_resource::*, texture::DEFAULT_IMAGE_HANDLE},
};
use std::collections::HashMap;

/// Material of a `Sprite` with a texture and color
#[derive(Debug, Clone)]
pub struct SpriteMaterial {
    pub color: Color,
    pub texture: Handle<Image>,
}
impl SpriteMaterial {
    #[autodefault::autodefault]
    pub fn sprite(&self, custom_size: Vec2, transform: Transform) -> SpriteBundle {
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(custom_size),
                color: self.color,
            },
            texture: self.texture.clone(),
            transform,
        }
    }
    #[autodefault::autodefault]
    pub fn node(&self, size: Vec2, transform: Transform) -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Px(size.x), Val::Px(size.y)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            },
            transform,
            color: self.color.into(),
            image: self.texture.clone().into(),
        }
    }
    //pub fn button(&self,
}
fn to_image(color: Color) -> Image {
    Image::new_fill(
        Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[
            (color.r() * u8::MAX as f32) as u8,
            (color.g() * u8::MAX as f32) as u8,
            (color.b() * u8::MAX as f32) as u8,
            (color.a() * u8::MAX as f32) as u8,
        ],
        TextureFormat::Rgba8Unorm,
    )
}
impl Default for SpriteMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            texture: DEFAULT_IMAGE_HANDLE.typed(),
        }
    }
}

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST;
                }
            }
            _ => (),
        }
    }
}
/// Assets for the board. Must be used as a resource.
///
/// Use the loader for partial setup
#[derive(Debug, Clone)]
pub struct BoardAssets {
    pub board: SpriteMaterial,
    pub card: SpriteMaterial,
    pub score_font: Handle<Font>,
    pub card_color: Vec<Color>,
    pub col_map: HashMap<Collection, HandleUntyped>,
}
impl FromWorld for BoardAssets {
    #[autodefault::autodefault(except(BoardAssets))]
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();
        BoardAssets {
            board: SpriteMaterial {
                color: Color::WHITE,
                texture: asset_server.load("tiles.png"), //images.add(to_image(Color::WHITE))
            },
            card: SpriteMaterial {
                color: Color::DARK_GRAY,
                texture: images.add(to_image(Color::DARK_GRAY)),
            },
            score_font: asset_server.load("fonts/pixeled.ttf"),
            card_color: vec![
                Color::WHITE,
                Color::GREEN,
                Color::YELLOW,
                Color::ORANGE,
                Color::PURPLE,
            ],
            col_map: HashMap::from([
                (
                    Collection::Eng,
                    asset_server.load_untyped("fonts/pixeled.ttf"),
                ),
                /*(
                    Collection::Dice,
                    asset_server.load_untyped("fonts/Dicier-Block-Heavy.ttf"),
                ),*/
                (
                    Collection::Clubs,
                    asset_server.load_untyped("fonts/clubs.ttf"),
                ),
                (
                    Collection::Hearts,
                    asset_server.load_untyped("fonts/hearts.ttf"),
                ),
                (
                    Collection::Spades,
                    asset_server.load_untyped("fonts/spades.ttf"),
                ),
                (
                    Collection::Diamonds,
                    asset_server.load_untyped("fonts/diamonds.ttf"),
                ),
                (
                    Collection::Tel,
                    asset_server.load_untyped("fonts/RaviPrakash.ttf"),
                ),
            ]),
        }
    }
}
impl BoardAssets {
    /// Safely retrieves the color matching a value
    pub fn card_color(&self, val: u16, max: u16) -> Color {
        let value = (val * self.card_color.len() as u16 / max).saturating_sub(1) as usize;
        match self.card_color.get(value) {
            Some(c) => *c,
            None => match self.card_color.last() {
                None => Color::WHITE,
                Some(c) => *c,
            },
        }
    }

    pub fn count_color(&self, val: u16) -> Color {
        match val {
            1 => Color::GREEN,
            2 => Color::WHITE,
            3 => Color::YELLOW,
            4 => Color::ORANGE,
            _ => Color::RED,
        }
    }
}
