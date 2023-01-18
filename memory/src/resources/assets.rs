use bevy::prelude::*;
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;

/// Material of a `Sprite` with a texture and color
#[cfg_attr(feature = "dev", derive(bevy_inspector_egui::Inspectable))]
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
            background_color: self.color.into(),
            // image: self.texture.clone().into(),
        }
    }
    #[autodefault::autodefault]
    pub fn button(&self, style: Style) -> ButtonBundle {
        ButtonBundle {
            style,
            background_color: self.color.into(),
            image: self.texture.clone().into(),
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
#[cfg_attr(feature = "dev", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Resource)]
pub struct MemoryGAssts {
    pub board: SpriteMaterial,
    pub card: [(SpriteMaterial, SpriteMaterial); 2],
    pub back_ground: SpriteMaterial,
    pub score_font: Handle<Font>,
    pub card_font: Handle<Font>,
}
impl FromWorld for MemoryGAssts {
    #[autodefault::autodefault(except(MemoryGAssts))]
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        MemoryGAssts {
            back_ground: SpriteMaterial {
                color: Color::rgba(0., 0., 0., 0.),
            },
            board: SpriteMaterial {
                color: Color::rgb_u8(112, 112, 255),
            },
            card: [
                (
                    SpriteMaterial {
                        color: Color::rgb_u8(130, 87, 38),
                    },
                    SpriteMaterial {
                        color: Color::rgb_u8(255, 200, 155),
                    },
                ),
                (
                    SpriteMaterial {
                        color: Color::rgb_u8(4, 86, 46),
                    },
                    SpriteMaterial {
                        color: Color::rgb_u8(205, 245, 218),
                    },
                ),
            ],
            score_font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            card_font: asset_server.load("fonts/Dicier-Cards.ttf"), //card_font: asset_server.load("fonts/pixeled.ttf")
        }
    }
}
impl MemoryGAssts {
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
                    value: std::char::from_u32(33 + val as u32 % 56)
                        .unwrap()
                        .to_string(),
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
    pub fn flip_card_color(&self, mut color: &mut BackgroundColor, visibility: bool) {
        color.0 = match visibility {
            true => {
                if color.0 == self.card[0].0.color {
                    self.card[0].1.color
                } else if color.0 == self.card[1].0.color {
                    self.card[1].1.color
                } else {
                    color.0
                }
            }
            false => {
                if color.0 == self.card[0].1.color {
                    self.card[0].0.color
                } else if color.0 == self.card[1].1.color {
                    self.card[1].0.color
                } else {
                    color.0
                }
            }
        };
    }
}
