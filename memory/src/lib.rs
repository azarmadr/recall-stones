use {
    autodefault::autodefault,
    bevy::{
        ecs::schedule::{ShouldRun, StateData},
        prelude::*,
    },
    menu::MenuPlugin,
    menu_plugin::MenuMaterials,
    rand::seq::SliceRandom,
    std::time::Duration,
    {components::*, deck::Deck, tween::*},
};
pub use {components::Player, events::*, resources::*};

#[cfg(feature = "debug")]
use {bevy::log, bevy_inspector_egui::InspectorPlugin};

pub mod components;
mod events;
mod menu;
mod resources;
mod systems;
pub mod tween;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    InGame,
    Splash,
    Menu,
}
use AppState::*;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Board;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ScoreBoard;

#[derive(Deref)]
pub struct MemoryGamePlugin<T>(pub T);
impl<T: StateData + Copy> Plugin for MemoryGamePlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_state(AppState::Splash)
            .add_plugin(TweeningPlugin)
            .add_system_set(SystemSet::on_enter(InGame).with_system(create_board))
            .add_system_set(
                SystemSet::on_update(InGame)
                    .with_system(systems::deck_complete.exclusive_system().at_end())
                    .with_system(systems::turn)
                    .with_system(systems::score_board),
            )
            .add_system_set(
                SystemSet::on_in_stack_update(InGame)
                    .with_system(systems::uncover)
                    .with_system(systems::card_flip),
            )
            .add_system_set(SystemSet::on_pause(InGame).with_system(hide_board))
            .add_system_set(SystemSet::on_resume(InGame).with_system(show_board))
            .add_system_set(SystemSet::on_exit(InGame).with_system(despawn::<Board>))
            .add_system_set(SystemSet::on_exit(InGame).with_system(despawn::<ScoreBoard>))
            .add_system(component_animator_system::<Visibility>)
            .init_resource::<MemoryGAssts>()
            .add_system(component_animator_system::<UiColor>)
            .add_plugin(MenuPlugin {
                game: InGame,
                menu: Menu,
            })
            .add_system_set(SystemSet::on_enter(**self).with_system(splash_off))
            .add_system_set(SystemSet::on_in_stack_update(**self).with_system(on_completion))
            .add_system_set(SystemSet::on_exit(**self).with_system(splash_on))
            .init_resource::<MemoryGOpts>();
        #[cfg(feature = "debug")]
        {
            app
                .add_plugin(InspectorPlugin::<Deck>::new())
                .add_plugin(InspectorPlugin::<MemoryGOpts>::new())
                // .add_plugin(InspectorPlugin::<MemoryGAssts>::new())
                ;
        }
    }
}

pub fn splash_on(mut state: ResMut<State<AppState>>) {
    state.push(AppState::Splash).unwrap();
}
pub fn splash_off(mut state: ResMut<State<AppState>>) {
    if state.inactives().is_empty() {
        state.replace(AppState::Menu).unwrap();
    } else {
        state.pop().unwrap();
    }
}

pub fn if_deck_not_done(deck: Option<Res<Deck>>) -> ShouldRun {
    if let Some(d) = deck {
        if d.outcome().is_none() {
            return ShouldRun::Yes;
        }
    }
    ShouldRun::No
}
pub fn show_board(mut cmd: Commands, board: Query<Entity, With<Board>>) {
    cmd.entity(board.single()).insert(Animator::new(Tween::new(
        EaseFunction::ElasticInOut,
        TweeningType::Once,
        std::time::Duration::from_millis(81),
        BeTween::with_lerp(|c: &mut Transform, _, r| c.scale = Vec3::ZERO.lerp(Vec3::ONE, r)),
    )));
}
pub fn hide_board(mut cmd: Commands, board: Query<Entity, With<Board>>) {
    cmd.entity(board.single()).insert(Animator::new(Tween::new(
        EaseFunction::ElasticInOut,
        TweeningType::Once,
        std::time::Duration::from_millis(81),
        BeTween::with_lerp(|c: &mut Transform, _, r| c.scale = Vec3::ONE.lerp(Vec3::ZERO, r)),
    )));
}
/// System to generate the complete board
#[autodefault(except(Board, TransformScaleLens, Size, Text, TextAlignment))]
pub fn create_board(
    mut cmd: Commands,
    material: Res<MenuMaterials>,
    opts: Res<MemoryGOpts>,
    mut assets: ResMut<MemoryGAssts>,
) {
    let mut rng = rand::thread_rng();
    assets.card.shuffle(&mut rng);
    let count = opts.deck_params().0;
    let deck_width = (2. * count as f32).sqrt().round();
    let players = opts.create_players();
    let deck = Deck::init(opts.deck_params(), opts.mode, players.len() as u8);
    // let size = opts.card_size(deck_width, deck_width);
    let size = material.size / deck_width.max(2. * (count as f32 / deck_width).ceil()) * 0.77;
    #[cfg(feature = "debug")]
    {
        log::info!("{deck}");
        log::info!("size {size}\ndeck_width {deck_width}");
    }
    let width = (deck_width + 0.3) * (size + 2.);
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
    cmd.spawn_bundle(assets.back_ground.node(Style {
        position_type: PositionType::Absolute,
        size: Size::new(Val::Percent(100.), Val::Percent(100.)),
        flex_direction: FlexDirection::ColumnReverse,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
    }))
    .insert(Name::new("Board"))
    .insert(Board)
    .with_children(|p| {
        let mut card_iter = deck.iter().enumerate();
        for half in 0..2u8 {
            p.spawn_bundle(assets.back_ground.node(Style {
                flex_basis: Val::Px(0.),
                flex_wrap: if half == 0 {
                    FlexWrap::Wrap
                } else {
                    FlexWrap::WrapReverse
                },
                flex_grow: 0.5,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::FlexStart,
                margin: if half == 0 {
                    UiRect {
                        top: Val::Px(width / 2.),
                        bottom: Val::Px(if opts.mode.full_plate { 0. } else { 3. }),
                    }
                } else {
                    UiRect {
                        top: Val::Px(if opts.mode.full_plate { 0. } else { 3. }),
                        bottom: Val::Px(width / 2.),
                    }
                },
                size: Size::new(Val::Px(width), Val::Percent(100.0)),
            }))
            .insert(Name::new(format!("Half: {}", half)))
            .with_children(|p| {
                for j in 0..count {
                    let (i, &card) = card_iter.next().unwrap();
                    p.spawn_bundle(assets.board.node(Style {
                        min_size: Size {
                            width: Val::Px(size),
                            height: Val::Px(size),
                        },
                    }))
                    .insert(Name::new("Board Color"))
                    .with_children(|p| {
                        p.spawn_bundle(assets.card[if card > 55 { 1 } else { 0 }].0.button(
                            Style {
                                margin: UiRect::all(Val::Px(1.0)),
                                min_size: Size {
                                    width: Val::Px(size),
                                    height: Val::Px(size),
                                },
                            },
                        ))
                        .insert(Animator::new(seq(j)))
                        .insert(Name::new(format!("Card {:?}", i)))
                        .insert(Idx(i, 0))
                        .with_children(|p| {
                            p.spawn_bundle(assets.spawn_card(card, size))
                                .insert(Name::new("Card"));
                        });
                    });
                }
            });
        }
    });
    cmd.spawn_bundle(assets.back_ground.node(Style {
        position_type: PositionType::Absolute,
        flex_basis: Val::Px(0.),
        flex_shrink: 0.,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        //align_content: AlignContent::FlexStart,
        align_self: AlignSelf::FlexEnd,
        size: Size::new(Val::Percent(100.), Val::Undefined),
    }))
    .insert(Name::new("Score Panel"))
    .insert(ScoreBoard)
    .with_children(|p| {
        players.iter().enumerate().for_each(|(n, pl)| {
            let text_bundle = |value| TextBundle {
                style: Style {
                    flex_basis: Val::Px(0.),
                    align_self: AlignSelf::Center,
                    margin: UiRect {
                        right: Val::Px(10.),
                        left: Val::Px(10.),
                    },
                },
                text: Text {
                    sections: vec![TextSection {
                        value,
                        style: TextStyle {
                            color: Color::RED,
                            font: assets.score_font.clone(),
                            font_size: material.size / 27. * 0.8,
                        },
                    }],
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                },
            };
            p.spawn_bundle(assets.board.node(Style {
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::FlexStart,
            }))
            .with_children(|p| {
                p.spawn_bundle(text_bundle(format!(
                    "{} {n}\nOpened: 0\nTurns: 0",
                    if pl.is_bot() { "Bot" } else { "Human" },
                )))
                .insert(*pl);
                /*
                p.spawn_bundle(assets.board.node(Style {}))
                .with_children(|p| {
                    for &key in ["Score", "Opened", "Turns"].iter() {
                        p.spawn_bundle(assets.board.node(Style {
                            flex_basis: Val::Px(0.)
                        }))
                            .with_children(|p| {
                                p.spawn_bundle(text_bundle(key.to_string()));
                            });
                    }
                });
                */
            });
        });
    });
    cmd.insert_resource(deck);
}
fn despawn<T: Component>(mut cmd: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}
/// Display Menu for 3 seconds before applying the set opts
fn on_completion(
    mut state: ResMut<State<AppState>>,
    mut opts: ResMut<MemoryGOpts>,
    mut timer: Local<Timer>,
    mut board_complete_evr: EventReader<TweenCompleted>,
    time: Res<Time>,
) {
    if timer.duration() == Duration::ZERO {
        timer.set_duration(Duration::from_secs(5));
    }
    if board_complete_evr
        .iter()
        .any(|&x| x.user_data == std::u64::MAX)
    {
        opts.level = 5.min(opts.level + 1);
        state.push(AppState::Menu).unwrap();
        timer.reset();
    }
    if timer.tick(time.delta()).just_finished() && state.current() != &AppState::InGame {
        state.replace(AppState::InGame).unwrap();
    }
    // a loading item can be added TODO
}
