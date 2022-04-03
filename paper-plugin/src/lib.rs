pub use crate::components::Collection;
use crate::components::*;
use crate::deck::Deck;
use crate::events::{CardFlipEvent, DeckCompletedEvent};
use bevy::ecs::schedule::StateData;
use bevy::log;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_tweening::{lens::*, *};
use rand::seq::index::sample;
use std::time::Duration;
//use bevy::render::view::Visibility;

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

#[derive(Default)]
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
                .with_system(systems::spawn::spawn_cards)
                .with_system(systems::uncover::open_card)
                .with_system(systems::uncover::deck_complete),
        )
        .add_system_set(
            SystemSet::on_in_stack_update(self.running_state.clone())
                .with_system(systems::uncover::reveal_cards)
                .with_system(systems::uncover::score),
        )
        .add_system_set(
            SystemSet::on_exit(self.running_state.clone()).with_system(Self::cleanup_board),
        )
        .add_system(component_animator_system::<Visibility>)
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
        let deck = Deck::init(options.deck_params);
        #[cfg(feature = "debug")]
        // Cardmap debugging
        log::info!("{}", deck.console_output());

        // Setup

        // We define the size of our cards in world space
        let window = windows.get_primary().unwrap();
        let card_size = options.adaptative_card_size(
            (window.width(), window.height()),
            (deck.width(), deck.height()),
        );
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

        let mut hidden_cards = HashMap::with_capacity(deck.len());
        let opened_count = HashMap::with_capacity(deck.len());
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
                                font: board_assets.score_font.clone(),
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
                    options.col_is_suites(),
                    options.collections,
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
            turns: 0,
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
        col_is_suites: bool,
        collections: Vec<Collection>,
        padding: f32,
        board_assets: &Res<BoardAssets>,
        hidden_cards: &mut HashMap<Idx, Entity>,
    ) {
        let mut col_map = HashMap::new();
        // Cards
        let mut start_time_ms = 0;
        for (i, card) in deck.iter().enumerate() {
            let (x, y) = (i % deck.width() as usize, i / deck.width() as usize);
            let id = Idx(i as u16);
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let couplets = deck.couplets() as usize;
            let sample_size = std::cmp::max(couplets, collections.len());
            let rand_bool = rng.gen_bool(0.5) as usize;
            let col = col_map
                .entry(card)
                .or_insert(
                    sample(&mut rng, sample_size, sample_size)
                        .iter()
                        .filter(|&x| !col_is_suites || rand_bool == x % 2)
                        .collect::<Vec<usize>>(),
                )
                .pop()
                .unwrap()
                % collections.len();

            let delay = Delay::new(Duration::from_millis(start_time_ms));
            start_time_ms += 81;
            let tween_scale = Tween::new(
                EaseFunction::BounceOut,
                TweeningType::Once,
                Duration::from_millis(243),
                TransformScaleLens {
                    start: Vec3::splat(0.27),
                    end: Vec3::ONE,
                },
            );
            let seq = delay.then(tween_scale);
            // Card sprite
            parent
                .spawn_bundle(SpriteBundle {
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
                .insert(Animator::new(seq))
                .insert(Name::new(format!("Card ({}, {})", x, y)))
                .with_children(|parent| {
                    let entity = parent
                        .spawn()
                        .insert(Name::new("Card"))
                        .insert(id)
                        .insert(collections[col])
                        .id();
                    hidden_cards.insert(id, entity);
                });
        }
    }

    fn cleanup_board(board: Res<Board>, mut commands: Commands) {
        commands.entity(board.entity).despawn_recursive();
        commands.remove_resource::<Board>();
    }
}
