use bevy::prelude::*;
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;

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
    pub fn node(&self, style: Style) -> NodeBundle {
        NodeBundle {
            style,
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
    pub back_ground: SpriteMaterial,
    pub score_font: Handle<Font>,
    pub card_font: Handle<Font>,
}
impl FromWorld for BoardAssets {
    #[autodefault::autodefault(except(BoardAssets))]
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        BoardAssets {
            back_ground: SpriteMaterial {
                color: Color::rgba(0., 0., 0., 0.),
            },
            board: SpriteMaterial {
                color: Color::WHITE,
            },
            card: SpriteMaterial {
                color: Color::DARK_GRAY,
            },
            score_font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            card_font: asset_server.load("fonts/Dicier-Cards.ttf"), //card_font: asset_server.load("fonts/pixeled.ttf")
        }
    }
}
impl BoardAssets {
    pub fn count_color(&self, val: u8) -> Color {
        match val {
            1 => Color::GREEN,
            2 => Color::WHITE,
            3 => Color::YELLOW,
            4 => Color::ORANGE,
            _ => Color::RED,
        }
    }
    pub fn spawn_card(&self, val: u16, size: f32) -> TextBundle {
        let color = if val / 14 % 2 == 0 {
            Color::BLACK
        } else {
            Color::RED
        };
        TextBundle {
            style: Style {
                flex_basis: Val::Px(0.),
                ..Default::default()
            },
            text: Text {
                sections: vec![TextSection {
                    value: std::char::from_u32(33 + val as u32).unwrap().to_string(),
                    style: TextStyle {
                        color,
                        font: self.card_font.clone(),
                        font_size: size,
                    },
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        }
    }
}
