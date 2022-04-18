use {
    autodefault::autodefault,
    bevy::{
        ecs::schedule::StateData,
        log, prelude::*
    },
    rand::{prelude::*,seq::index::sample},
    std::collections::HashMap,
    std::time::Duration,
    {components::*, deck::Deck, tween::*},
};
pub use {components::ScoreBoard, events::*, resources::*};

//use mat::*;//mat
#[cfg(feature = "debug")]
use bevy_inspector_egui::RegisterInspectable;

pub mod components;
mod events;
mod resources;
mod systems;
mod player;
pub mod tween;

#[derive(Component)]
struct InsertDeck;

#[derive(Deref)]
pub struct PaperPlugin<T>(pub T);
impl<T: StateData+Copy> Plugin for PaperPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(**self)
                .with_system(create_board.exclusive_system())
                .with_system(spawn_cards)
//                .with_system(create_mat),//mat
        )
        .add_system_set(
            SystemSet::on_update(**self)
                .with_system(systems::input::input_handling)
                .with_system(systems::uncover::open_card)
                .with_system(systems::uncover::deck_complete)
                .with_system(systems::uncover::ai),
        )
        .add_system_set(
            SystemSet::on_in_stack_update(**self)
                .with_system(systems::uncover::reveal_cards)
                .with_system(systems::uncover::score),
        )
        .add_system_set(SystemSet::on_exit(**self).with_system(cleanup_board))
        .add_system(component_animator_system::<Visibility>)
        .add_event::<CardFlipEvent>()
        .add_event::<DeckCompletedEvent>()
//        .add_plugin(MatPlugin(**self)) //mat
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
pub fn create_board(mut cmd: Commands, options: Res<BoardOptions>, assets: Res<BoardAssets>) {
    let deck = Deck::init(options.deck_params(), options.mode);
    // Setup
    let card_size = options.card_size(deck.width(), deck.height());
    let board_size = Vec2::new(
        deck.width() as f32 * card_size,
        deck.height() as f32 * card_size,
    );
    let board_position = options.board_position(board_size);

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
        .insert(Transform::from_translation(board_position))
        .insert(GlobalTransform::default())
        .with_children(|parent| {
            parent
                .spawn_bundle(assets.board.sprite(
                    board_size,
                    Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                ))
                .insert(Name::new("Background"));
        })
        .insert(InsertDeck)
        .id();
        let mut rng = thread_rng();
    cmd.insert_resource(Board {
        deck,
        card_size,
        board_position,
        hidden_cards,
        opened_count,
        entity: board_entity,
        current_player: rng.gen_range(0..options.players),
        player_panels: vec![default()]
    })
}
//#[autodefault(except(Board))] //mat
//pub fn create_mat(mut cmd: Commands, board: Res<Board>) { //mat
//    let deck = &board.deck; //mat
//    // Create map entity and component: //mat
//    let settings = LayerSettings::new( //mat
//        MapSize(deck.width().into(), deck.height().into()), //mat
//        ChunkSize(1, 1), //mat
//        TileSize(16.0, 16.0), //mat
//        TextureSize(96.0, 16.0), //mat
//    ); //mat
//    cmd.spawn().insert(settings); //mat
//} //mat
#[autodefault(except(TransformScaleLens))]
fn spawn_cards(
    mut cmd: Commands,
    mut board: ResMut<Board>,
    options: Res<BoardOptions>,
    deck: Query<Entity, With<InsertDeck>>,
    assets: Res<BoardAssets>,
) {
    cmd.entity(deck.get_single().unwrap())
        .with_children(|parent| {
            let mut col_map = HashMap::new();
            // Cards
            let size = board.card_size;
            let collections = &options.collections;
            let padding = options.card_padding;
            let (count, max, couplets) = options.deck_params();
            let mut rng = rand::thread_rng();
            let sample_size = std::cmp::max(couplets as usize, collections.len());
            let seq = |i| {
                Delay::new(Duration::from_millis(i as u64 * 81)).then(Tween::new(
                    EaseFunction::BounceOut,
                    TweeningType::Once,
                    Duration::from_millis(243),
                    TransformScaleLens {
                        start: Vec3::splat(0.27),
                        end: Vec3::ONE,
                    },
                ))
            };
            for i in 0..count * 2 {
                let (x, y) = (i % board.deck.width(), i / board.deck.width());
                let card = board.deck[i as usize];
                let id = Idx(i);
                let zebra = match options.mode {
                    Mode::Zebra | Mode::SameColor => true,
                    _ => false,
                };
                let col = col_map
                    .entry(card)
                    .or_insert(
                        sample(&mut rng, sample_size, sample_size)
                            .iter()
                            .filter(|&x| !zebra || ((card >= max / 2) == (x < 2)))
                            .collect::<Vec<usize>>(),
                    )
                    .pop()
                    .expect(&*format!("card:{:?}", card))
                    % collections.len();
                // Card sprite
                parent
                    .spawn_bundle(assets.card.sprite(
                        Vec2::splat(size - padding),
                        Transform::from_xyz(
                            (x as f32 * size) + (size / 2.),
                            (y as f32 * size) + (size / 2.),
                            1.,
                        ),
                    ))
                    .insert(Animator::new(seq(i)))
                    .insert(Name::new(format!("Card {:?}", (x, y))))
                    .with_children(|parent| {
                        let entity = parent
                            .spawn_bundle(collections[col].spawn_card(
                                card,
                                &assets,
                                max,
                                size - padding,
                                options.mode,
                            ))
                            .insert(Name::new("Card"))
                            .insert(id)
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
