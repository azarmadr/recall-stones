use crate::components::*;
use crate::deck::Deck;
use crate::events::{CardFlipEvent, DeckCompletedEvent};
use bevy::ecs::schedule::StateData;
use bevy::log;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::render::view::Visibility;
use bevy::text::Text2dSize;

#[cfg(feature = "debug")]
use bevy_inspector_egui::RegisterInspectable;
pub use bounds::*;
pub use resources::*;
use std::collections::HashMap;

mod bounds;
pub mod components;
pub mod events;
mod resources;
mod systems;

pub struct PaperPlugin<T> {
    pub running_state: T,
}

impl<T: StateData> Plugin for PaperPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(self.running_state.clone()).with_system(Self::create_board),
        )
        .add_system_set(
            SystemSet::on_update(self.running_state.clone())
                .with_system(systems::input::input_handling)
                .with_system(systems::uncover::trigger_event_handler),
        )
        .add_system_set(
            SystemSet::on_in_stack_update(self.running_state.clone())
                .with_system(systems::uncover::flip_cards),
        )
        .add_system_set(
            SystemSet::on_exit(self.running_state.clone()).with_system(Self::cleanup_board),
        )
        .add_event::<CardFlipEvent>()
        .add_event::<DeckCompletedEvent>();
        #[cfg(feature = "debug")]
        {
            app.register_inspectable::<Idx>()
                .register_inspectable::<Coordinates>()
                .register_inspectable::<Open>()
                .register_inspectable::<Revealed>();
        }
        log::info!("Loaded Board Plugin");
    }
}

impl<T> PaperPlugin<T> {
    /// System to generate the complete board
    pub fn create_board(
        mut commands: Commands,
        board_options: Option<Res<BoardOptions>>,
        board_assets: Res<BoardAssets>,
        windows: Res<Windows>,
    ) {
        let options = match board_options {
            None => BoardOptions::default(), // If no options is set we use the default one
            Some(o) => o.clone(),
        };

        // Cardmap generation
        let mut deck = Deck::empty(options.deck_size.0);
        deck.set_cards(options.max_limit);
        #[cfg(feature = "debug")]
        // Cardmap debugging
        log::info!("{}", deck.console_output());

        // Setup

        // We define the size of our cards in world space
        let card_size = match options.card_size {
            CardSize::Fixed(v) => v,
            CardSize::Adaptive { min, max } => Self::adaptative_card_size(
                windows.get_primary().unwrap(),
                (min, max),
                (deck.width(), deck.height()),
            ),
        };
        // We deduce the size of the complete board
        let board_size = Vec2::new(
            deck.width() as f32 * card_size,
            deck.height() as f32 * card_size,
        );
        log::info!("board size: {}", board_size);
        // We define the board anchor position (bottom left)
        let board_position = match options.position {
            BoardPosition::Centered { offset } => {
                Vec3::new(-(board_size.x / 2.), -(board_size.y / 2.), 0.) + offset
            }
            BoardPosition::Custom(p) => p,
        };

        let mut hidden_cards = HashMap::with_capacity((2 * deck.count()).into());
        let opened_count = HashMap::with_capacity((2 * deck.count()).into());
        let board_entity = commands
            .spawn()
            .insert(Name::new("Board"))
            .insert(Transform::from_translation(board_position))
            // This component is required until https://github.com/bevyengine/bevy/pull/2331 is merged
            .insert(GlobalTransform::default())
            .with_children(|parent| {
                // We spawn the board background sprite at the center of the board, since the sprite pivot is centered
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(board_size),
                            color: board_assets.board_material.color,
                            ..Default::default()
                        },
                        texture: board_assets.board_material.texture.clone(),
                        transform: Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                        ..Default::default()
                    })
                    .insert(Name::new("Background"));
                parent
                    .spawn_bundle(Text2dBundle {
                        text: Text::with_section(
                            format!(
                                "Luck: {}\nPerfect Memory: {}",
                                deck.count(),
                                deck.count() * 2 - 1
                            ),
                            TextStyle {
                                font: board_assets.counter_font.clone(),
                                color: Color::WHITE,
                                font_size: 27.0,
                            },
                            Default::default(),
                        ),
                        ..Default::default() //style:
                    })
                    .insert(Name::new("Score"))
                    .insert(Score);
                Self::spawn_cards(
                    parent,
                    &deck,
                    card_size,
                    options.card_padding,
                    &board_assets,
                    &mut hidden_cards,
                );
            })
            .id();
        // We add the main resource of the game, the board
        commands.insert_resource(Board {
            deck,
            bounds: Bounds2 {
                position: board_position.xy(),
                size: board_size,
            },
            score: 0,
            completed: false,
            card_size,
            hidden_cards,
            opened_count,
            entity: board_entity,
        })
    }

    fn spawn_cards(
        parent: &mut ChildBuilder,
        deck: &Deck,
        size: f32,
        padding: f32,
        board_assets: &Res<BoardAssets>,
        hidden_cards: &mut HashMap<Idx, Entity>,
    ) {
        // Cards
        for (i, card) in deck.iter().enumerate() {
            let (x, y) = (i % deck.width() as usize, i / deck.width() as usize);
            let coordinates = Idx(i as u16);
            let mut cmd = parent.spawn();

            // Card sprite
            cmd.insert_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(size - padding)),
                    color: board_assets.card_material.color,
                    ..Default::default()
                },
                texture: board_assets.card_material.texture.clone(),
                transform: Transform::from_xyz(
                    (x as f32 * size) + (size / 2.),
                    (y as f32 * size) + (size / 2.),
                    1.,
                ),
                ..Default::default()
            })
            .insert(Name::new(format!("Card ({}, {})", x, y)));
            cmd.with_children(|parent| {
                let entity = parent
                    .spawn_bundle(Self::card_to_text_bundle(
                        card.val(),
                        deck.max(),
                        board_assets,
                        size - padding,
                    ))
                    .insert(Name::new("Card"))
                    .insert(coordinates)
                    .id();
                hidden_cards.insert(coordinates, entity);
            });
        }
    }

    /// Generates the card value text 2D Bundle
    fn card_to_text_bundle(
        value: u16,
        max: u16,
        board_assets: &Res<BoardAssets>,
        size: f32,
    ) -> Text2dBundle {
        // We retrieve the text and the correct color
        let color = board_assets.card_color(value * board_assets.card_color.len() as u16 / max);
        // We generate a text bundle
        Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: value.to_string(),
                    style: TextStyle {
                        color,
                        font: board_assets.counter_font.clone(),
                        font_size: size,
                    },
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
            text_2d_size: Text2dSize {
                size: Size {
                    width: size,
                    height: size,
                },
            },
            visibility: Visibility { is_visible: false },
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        }
    }

    /// Computes a card size that matches the window according to the card map size
    fn adaptative_card_size(
        window: &Window,
        (min, max): (f32, f32),
        (width, height): (u16, u16),
    ) -> f32 {
        let max_width = window.width() / width as f32;
        let max_heigth = window.height() / height as f32;
        max_width.min(max_heigth).clamp(min, max)
    }

    fn cleanup_board(board: Res<Board>, mut commands: Commands) {
        commands.entity(board.entity).despawn_recursive();
        commands.remove_resource::<Board>();
    }
}
