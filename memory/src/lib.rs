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
pub use {components::Player, resources::*};

#[cfg(feature = "dev")]
use {bevy::log, bevy_inspector_egui::InspectorPlugin};

pub mod components;
mod menu;
mod resources;
mod systems;
pub mod tween;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Game,
    Splash,
    Menu,
}
use bevy::ui::FocusPolicy;
use GameState::*;

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
        app.add_state(GameState::Splash)
            .add_plugin(TweeningPlugin)
            .add_plugin(MenuPlugin)
            .add_system(component_animator_system::<Visibility>)
            .add_system(component_animator_system::<BackgroundColor>)
            .add_system_set(SystemSet::on_enter(Game).with_system(create_board))
            .add_system_set(
                SystemSet::on_update(Game)
                    .with_run_criteria(resource_exists::<Deck>)
                    .with_system(systems::deck_complete.at_end())
                    .with_system(systems::turn)
                    .with_system(systems::score_board),
            )
            .add_system_set(
                SystemSet::on_in_stack_update(Game)
                    .with_run_criteria(resource_exists::<Deck>)
                    .with_system(systems::uncover),
            )
            .init_resource::<MemoryGAssts>()
            .add_system(board_display)
            .add_system(systems::card_flip)
            .add_system_set(SystemSet::on_exit(Game).with_system(despawn::<Board>))
            .add_system_set(SystemSet::on_exit(Game).with_system(despawn::<ScoreBoard>))
            .add_system_set(SystemSet::on_enter(**self).with_system(splash_off))
            .add_system_set(SystemSet::on_in_stack_update(**self).with_system(on_completion))
            .add_system_set(SystemSet::on_exit(**self).with_system(splash_on))
            .init_resource::<MemoryGOpts>();

        #[cfg(feature = "dev")]
        app.add_plugin(InspectorPlugin::<MemoryGOpts>::new());
    }
}

pub fn splash_on(mut state: ResMut<State<GameState>>) {
    state.push(GameState::Splash).unwrap();
}
pub fn splash_off(mut state: ResMut<State<GameState>>) {
    if state.inactives().is_empty() {
        state.replace(GameState::Menu).unwrap();
    } else {
        state.pop().unwrap();
    }
}

pub fn resource_exists<T: Resource>(res: Option<Res<T>>) -> ShouldRun {
    res.is_some().into()
}

#[allow(clippy::type_complexity)]
pub fn board_display(
    mut board: Query<&mut Style, Or<(With<Board>, With<ScoreBoard>)>>,
    state: Res<State<GameState>>,
) {
    board.for_each_mut(|mut style| {
        style.display = match state.current() {
            GameState::Game => Display::Flex,
            _ => Display::None,
        }
    })
}
/// System to generate the complete board
#[autodefault(except(Board, TransformScaleLens, Size, Text, TextAlignment))]
pub fn create_board(
    mut cmd: Commands,
    material: Res<MenuMaterials>,
    mut opts: ResMut<MemoryGOpts>,
    mut assets: ResMut<MemoryGAssts>,
) {
    let mut rng = rand::thread_rng();
    assets.card.shuffle(&mut rng);
    let count = opts.deck_params().0;
    opts.outcome = None;
    let deck_width = (2. * count as f32).sqrt().round();
    let players = opts.create_players();
    let deck = Deck::init(opts.deck_params(), opts.mode, players.len() as u8);
    let size = material.size / deck_width.max(2. * (count as f32 / deck_width).ceil()) * 0.77;

    #[cfg(feature = "dev")]
    log::info!("{deck}\nsize {size}\ndeck_width {deck_width}");

    let width = (deck_width + 0.3) * (size + 2.);
    let seq = |i| {
        Delay::new(Duration::from_millis(27 + i as u64 * 81)).then(Tween::new(
            EaseFunction::BounceOut,
            Duration::from_millis(243),
            TransformScaleLens {
                start: Vec3::splat(0.27),
                end: Vec3::ONE,
            },
        ))
    };
    cmd.spawn(assets.back_ground.node(Style {
        position_type: PositionType::Absolute,
        size: Size::new(Val::Percent(100.), Val::Percent(100.)),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
    }))
    .insert(FocusPolicy::Pass)
    .insert(Name::new("Board"))
    .insert(Board)
    .with_children(|p| {
        let mut card_iter = deck.iter().enumerate();
        for half in 0..2u8 {
            p.spawn(assets.back_ground.node(Style {
                flex_basis: Val::Px(0.),
                flex_wrap: if half == 0 {
                    FlexWrap::WrapReverse
                } else {
                    FlexWrap::Wrap
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
            .insert(Name::new(format!("Half: {half}")))
            .with_children(|p| {
                for j in 0..count {
                    let (i, &card) = card_iter.next().unwrap();
                    p.spawn(assets.board.node(Style {
                        min_size: Size {
                            width: Val::Px(size),
                            height: Val::Px(size),
                        },
                    }))
                    .insert(Name::new("Board Color"))
                    .with_children(|p| {
                        p.spawn(assets.card[if card > 55 { 1 } else { 0 }].0.button(Style {
                            margin: UiRect::all(Val::Px(1.0)),
                            min_size: Size {
                                width: Val::Px(size),
                                height: Val::Px(size),
                            },
                        }))
                        .insert(Animator::new(seq(j)))
                        .insert(Name::new(format!("Card {i:?}")))
                        .insert(Idx(i, 0))
                        .with_children(|p| {
                            p.spawn(assets.spawn_card(card, size))
                                .insert(Name::new("Card"));
                        });
                    });
                }
            });
        }
    });
    cmd.spawn(assets.back_ground.node(Style {
        position_type: PositionType::Absolute,
        flex_basis: Val::Px(0.),
        flex_shrink: 0.,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        //align_content: AlignContent::FlexStart,
        align_self: AlignSelf::FlexEnd,
        size: Size::new(Val::Percent(100.), Val::Undefined),
    }))
    .insert(FocusPolicy::Pass)
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
            p.spawn(assets.board.node(Style {
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::FlexStart,
            }))
            .with_children(|p| {
                p.spawn(text_bundle(format!(
                    "{} {n}\nOpened: 0\nTurns: 0",
                    if pl.is_bot() { "Bot" } else { "Human" },
                )))
                .insert(*pl);
                /*
                p.spawn(assets.board.node(Style {}))
                .with_children(|p| {
                    for &key in ["Score", "Opened", "Turns"].iter() {
                        p.spawn(assets.board.node(Style {
                            flex_basis: Val::Px(0.)
                        }))
                            .with_children(|p| {
                                p.spawn(text_bundle(key.to_string()));
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
    mut state: ResMut<State<GameState>>,
    mut opts: ResMut<MemoryGOpts>,
    mut timer: Local<Timer>,
    cards: Query<&Visibility, With<Idx>>,
    time: Res<Time>,
) {
    if timer.duration() == Duration::ZERO {
        timer.set_duration(Duration::from_secs(5));
        timer.pause();
    }
    if opts.outcome.is_some() {
        timer.tick(time.delta());
        if cards.iter().all(|x| x.is_visible) {
            timer.unpause();
            if timer.percent() > 0.5 && state.inactives().is_empty() {
                if opts.auto_start {
                    opts.level = 5.min(opts.level + 1);
                }
                state.push(GameState::Menu).unwrap();
            }
        } else if timer.percent() > 0.27 {
            timer.pause()
        } else {
            timer.unpause();
        }
        if timer.just_finished() {
            if state.current() != &GameState::Game && opts.auto_start {
                state.replace(GameState::Game).unwrap();
            }
            timer.reset();
        }
    }
}
