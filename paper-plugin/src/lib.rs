use {
    autodefault::autodefault,
    bevy::{ecs::schedule::StateData, log, math::Vec3Swizzles, prelude::*},
    bevy_ecs_tilemap::*,
    rand::seq::index::sample,
    std::collections::HashMap,
    std::time::Duration,
    {components::*, deck::Deck, tween::*},
};
pub use {
    bounds::*,
    components::{Collection, ScoreBoard},
    events::*,
    resources::*,
};

#[cfg(feature = "debug")]
use bevy_inspector_egui::RegisterInspectable;

mod bounds;
pub mod components;
mod events;
mod resources;
mod systems;
pub mod tween;

#[derive(Component)]
struct InsertDeck;

#[derive(Default)]
pub struct PaperPlugin<T>(pub T);
impl<T: StateData> Plugin for PaperPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin)
            .add_system_set(
            SystemSet::on_enter(self.0.clone()).with_system(create_board.exclusive_system()), //.with_system(spawn_cards)
                                                                                              //.with_system(systems::spawn::spawn_cards.exclusive_system().at_end())
        )
        .add_system_set(
            SystemSet::on_update(self.0.clone())
                .with_system(systems::input::input_handling)
                .with_system(systems::uncover::open_card), //.with_system(systems::uncover::deck_complete)
        )
        .add_system_set(
            SystemSet::on_in_stack_update(self.0.clone())
                .with_system(systems::uncover::reveal_cards)
                .with_system(systems::uncover::score),
        )
        .add_system_set(SystemSet::on_exit(self.0.clone()).with_system(cleanup_board))
        .add_system(component_animator_system::<Visibility>)
        .add_system(resources::set_texture_filters_to_nearest)
        .add_event::<CardFlipEvent>()
        .add_event::<DeckCompletedEvent>()
        .init_resource::<BoardAssets>()
        .init_resource::<BoardOptions>();
        #[cfg(feature = "debug")]
        {
            app.register_inspectable::<Idx>()
                .register_inspectable::<Coordinates>()
                .register_inspectable::<Open>()
                .register_inspectable::<Revealed>();
            log::info!("Loaded Board Plugin");
        }
    }
}

/// System to generate the complete board
#[autodefault(except(Board))]
pub fn create_board(
    mut cmd: Commands,
    board_options: Option<Res<BoardOptions>>,
    assets: Res<BoardAssets>,
    windows: Res<Windows>,
    mut map_query: MapQuery,
) {
    let options = match board_options {
        None => {
            cmd.insert_resource(BoardOptions::default());
            BoardOptions::default()
        }
        Some(o) => o.clone(),
    };
    let deck = Deck::init(options.deck_params(), options.mode);
    // Setup
    let window = windows.get_primary().unwrap();
    let card_size = options.adaptative_card_size(
        (window.width(), window.height()),
        (deck.width(), deck.height()),
    );
    let board_size = Vec2::new(
        deck.width() as f32 * card_size,
        deck.height() as f32 * card_size,
    );
    let board_position = match options.position {
        BoardPosition::Centered { offset } => {
            Vec3::new(-(board_size.x / 2.), -(board_size.y / 2.), 0.) + offset
        }
        BoardPosition::Custom(p) => p,
    };

    #[cfg(feature = "debug")]
    {
        log::info!("{}", deck.console_output());
        log::info!("card size: {}", card_size);
        log::info!("board size: {}", board_size);
    }
    let hidden_cards = HashMap::with_capacity(deck.len());
    let opened_count = HashMap::with_capacity(deck.len());
    let board_entity = cmd
        .spawn()
        .insert(Name::new("Board"))
        // This component is required until https://github.com/bevyengine/bevy/pull/2331 is merged
        /*
        .spawn_bundle(assets.board.node(board_size,Transform::from_translation(board_position)))
        .with_children(|parent| {
            // We spawn the board background sprite at the center of the board,
            // since the sprite pivot is centered
            parent
                .spawn_bundle(assets.board.sprite(
                    Some(board_size),
                    Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                ))
                .insert(Name::new("Background"));
        })
        */
        //.insert(InsertDeck)
        .id();
    let mut map = Map::new(0u16, board_entity);
    let (mut layer_builder, _) = LayerBuilder::new(
        &mut cmd,
        LayerSettings::new(
            MapSize(deck.width() as u32, deck.height() as u32),
            ChunkSize(1, 1),
            TileSize(card_size, card_size),
            TextureSize(96.0, 16.0),
        ),
        0,
        0,
    );
    layer_builder.set_all(TileBundle::default());
    let layer_entity = map_query.build_layer(&mut cmd, layer_builder, assets.board.texture.clone());
    map.add_layer(&mut cmd, 0, layer_entity);
    cmd.entity(board_entity)
        .insert(map)
        .insert(Transform::from_translation(board_position))
        .insert(GlobalTransform::default());
    // We add the main resource of the game, the board
    cmd.insert_resource(Board {
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
    mut cmd: Commands,
    mut board: ResMut<Board>,
    board_options: Res<BoardOptions>,
    deck: Query<Entity, With<InsertDeck>>,
    assets: Res<BoardAssets>,
) {
    cmd.entity(deck.get_single().unwrap())
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
                    .spawn_bundle(assets.card.node(
                        Vec2::splat(size - padding),
                        Transform::from_xyz(
                            (x as f32 * size) + (size / 2.),
                            (y as f32 * size) + (size / 2.),
                            1.,
                        ),
                    ))
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
fn cleanup_board(board: Res<Board>, mut cmd: Commands) {
    cmd.entity(board.entity).despawn_recursive();
    cmd.remove_resource::<Board>();
}
