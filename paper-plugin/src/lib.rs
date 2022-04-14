pub use {bounds::*, resources::*, events::*, components::{ScoreBoard,Collection}};
use { components::*, deck::Deck, tween::*, };
use bevy::{ecs::schedule::StateData, log, math::Vec3Swizzles, prelude::*};
use rand::seq::index::sample;
use std::time::Duration;

use autodefault::autodefault;
#[cfg(feature = "debug")]
use bevy_inspector_egui::RegisterInspectable;
use std::collections::HashMap;

mod bounds;
pub mod components;
mod events;
mod resources;
mod systems;
pub mod tween;

#[derive(Component)]
struct InsertDeck;

#[derive(Default)]
pub struct PaperPlugin<T> {
    pub running_state: T,
}
impl<T: StateData> Plugin for PaperPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(self.running_state.clone())
                .with_system(create_board.exclusive_system())
                .with_system(spawn_cards)
                .with_system(systems::spawn::spawn_cards.exclusive_system().at_end()),
        )
        .add_system_set(
            SystemSet::on_update(self.running_state.clone())
                .with_system(systems::input::input_handling)
                .with_system(systems::uncover::open_card)
                .with_system(systems::uncover::deck_complete),
        )
        .add_system_set(
            SystemSet::on_in_stack_update(self.running_state.clone())
                .with_system(systems::uncover::reveal_cards)
                .with_system(systems::uncover::score),
        )
        .add_system_set(SystemSet::on_exit(self.running_state.clone()).with_system(cleanup_board))
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

/// System to generate the complete board
#[autodefault(except(Board))]
pub fn create_board(
    mut commands: Commands,
    board_options: Option<Res<BoardOptions>>,
    board_assets: Res<BoardAssets>,
    windows: Res<Windows>,
) {
    let options = match board_options {
        None => {
            commands.insert_resource(BoardOptions::default());
            BoardOptions::default()
        }
        Some(o) => o.clone(),
    };

    let deck = Deck::init(options.deck_params(), options.mode);
    #[cfg(feature = "debug")]
    log::info!("{}", deck.console_output());

    // Setup
    let window = windows.get_primary().unwrap();
    let card_size = options.adaptative_card_size(
        (window.width(), window.height()),
        (deck.width(), deck.height()),
    );
    log::info!("card size: {}", card_size);
    let board_size = Vec2::new(
        deck.width() as f32 * card_size,
        deck.height() as f32 * card_size,
    );
    log::info!("board size: {}", board_size);
    let board_position = match options.position {
        BoardPosition::Centered { offset } => {
            Vec3::new(-(board_size.x / 2.), -(board_size.y / 2.), 0.) + offset
        }
        BoardPosition::Custom(p) => p,
    };

    let hidden_cards = HashMap::with_capacity(deck.len());
    let opened_count = HashMap::with_capacity(deck.len());
    let board_entity = commands
        .spawn()
        .insert(Name::new("Board"))
        .insert(Transform::from_translation(board_position))
        // This component is required until https://github.com/bevyengine/bevy/pull/2331 is merged
        .insert(GlobalTransform::default())
        .with_children(|parent| {
            // We spawn the board background sprite at the center of the board,
            // since the sprite pivot is centered
            parent
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(board_size),
                        color: board_assets.board_material.color,
                    },
                    texture: board_assets.board_material.texture.clone(),
                    transform: Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                })
                .insert(Name::new("Background"));
        })
        .insert(InsertDeck)
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
#[autodefault(except(TransformScaleLens))]
fn spawn_cards(
    mut commands: Commands,
    mut board: ResMut<Board>,
    board_options: Res<BoardOptions>,
    children: Query<Entity, With<InsertDeck>>,
    board_assets: Res<BoardAssets>,
) {
    commands
        .entity(children.get_single().unwrap())
        .with_children(|parent| {
            let mut col_map = HashMap::new();
            // Cards
            let mut start_time_ms = 0;
            let size = board.card_size;
            let collections = &board_options.collections;
            let padding = board_options.card_padding;
            let deck = board.deck.clone();
            let couplets = deck.couplets() as usize;
            let mut rng = rand::thread_rng();
            let sample_size = std::cmp::max(couplets, collections.len());
            for (i, &card) in deck.iter().enumerate() {
                let (x, y) = (i % deck.width() as usize, i / deck.width() as usize);
                let id = Idx(i as u16);
                let zebra = match board_options.mode {
                    Mode::Zebra | Mode::SameColor => true,
                    _ => false,
                };
                let col = col_map
                    .entry(card)
                    .or_insert(
                        sample(&mut rng, sample_size, sample_size)
                            .iter()
                            .filter(|&x| !zebra || ((card >= deck.max() / 2) == (x < 2)))
                            .collect::<Vec<usize>>(),
                    )
                    .pop()
                    .unwrap()
                    % collections.len();

                let seq = Delay::new(Duration::from_millis(start_time_ms)).then(Tween::new(
                    EaseFunction::BounceOut,
                    TweeningType::Once,
                    Duration::from_millis(243),
                    TransformScaleLens {
                        start: Vec3::splat(0.27),
                        end: Vec3::ONE,
                    },
                ));
                start_time_ms += 81;
                // Card sprite
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(size - padding)),
                            color: board_assets.card_material.color,
                        },
                        texture: board_assets.card_material.texture.clone(),
                        transform: Transform::from_xyz(
                            (x as f32 * size) + (size / 2.),
                            (y as f32 * size) + (size / 2.),
                            1.,
                        ),
                    })
                    .insert(Animator::new(seq))
                    .insert(Name::new(format!("Card {:?}", (x, y))))
                    .with_children(|parent| {
                        let entity = parent
                            .spawn()
                            .insert(Name::new("Card"))
                            .insert(id)
                            .insert(collections[col])
                            .id();
                        board.hidden_cards.insert(id, entity);
                    });
            }
        })
        .remove::<InsertDeck>();
}
fn cleanup_board(board: Res<Board>, mut commands: Commands) {
    commands.entity(board.entity).despawn_recursive();
    commands.remove_resource::<Board>();
}
