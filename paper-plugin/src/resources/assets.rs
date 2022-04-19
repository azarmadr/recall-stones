use super::{Collection, Collection::*};
use bevy::prelude::*;
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;
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
    pub fn node(&self, style: Style, transform: Transform) -> NodeBundle {
        NodeBundle {
            style,
            transform,
            color: self.color.into(),
            image: self.texture.clone().into(),
        }
    }
    pub fn button(&self, style: Style) -> ButtonBundle {
        ButtonBundle {
            style,
            color: self.color.into(),
            image: self.texture.clone().into(),
            ..Default::default()
        }
    }
}
impl Default for SpriteMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            texture: DEFAULT_IMAGE_HANDLE.typed(),
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
        BoardAssets {
            board: SpriteMaterial {
                color: Color::WHITE,
            },
            card: SpriteMaterial {
                color: Color::DARK_GRAY,
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
                (Eng, asset_server.load_untyped("fonts/pixeled.ttf")),
                /*(
                    Dice,
                    asset_server.load_untyped("fonts/Dicier-Block-Heavy.ttf"),
                ),*/
                (Clubs, asset_server.load_untyped("fonts/clubs.ttf")),
                (Hearts, asset_server.load_untyped("fonts/hearts.ttf")),
                (Spades, asset_server.load_untyped("fonts/spades.ttf")),
                (Diamonds, asset_server.load_untyped("fonts/diamonds.ttf")),
                (Tel, asset_server.load_untyped("fonts/RaviPrakash.ttf")),
            ]),
        }
    }
}
impl BoardAssets {
    /// Safely retrieves the color matching a value
    pub fn card_color(&self, val: u8, max: u8) -> Color {
        let value = (val * self.card_color.len() as u8 / max).saturating_sub(1) as usize;
        match self.card_color.get(value) {
            Some(c) => *c,
            None => match self.card_color.last() {
                None => Color::WHITE,
                Some(c) => *c,
            },
        }
    }
    pub fn count_color(&self, val: u8) -> Color {
        match val {
            1 => Color::GREEN,
            2 => Color::WHITE,
            3 => Color::YELLOW,
            4 => Color::ORANGE,
            _ => Color::RED,
        }
    }
}
