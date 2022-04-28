use {
    autodefault::autodefault,
    bevy::{
        ecs::schedule::{ShouldRun, StateData},
        prelude::*,
    },
    std::time::Duration,
    {components::*, deck::Deck, tween::*},
};
pub use {
    components::{Player, ScoreBoard},
    events::*,
    resources::*,
};

//use mat::*;//mat
#[cfg(feature = "debug")]
use {bevy::log, bevy_inspector_egui::RegisterInspectable};

pub mod components;
mod events;
mod resources;
mod systems;
pub mod tween;

#[derive(Debug, Copy, Clone)]
pub struct DeckCompletedEvent;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Board;

#[derive(Deref)]
pub struct PaperPlugin<T>(pub T);
impl<T: StateData + Copy> Plugin for PaperPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(**self).with_system(create_board))
            .add_system_set(
                SystemSet::on_update(**self)
                    .with_system(systems::deck_complete.exclusive_system().at_end())
                    .with_system(systems::turn)
                    .with_system(systems::score_board),
            )
            .add_system_set(SystemSet::on_in_stack_update(**self).with_system(systems::uncover))
            .add_system_set(SystemSet::on_pause(**self).with_system(hide_board))
            .add_system_set(SystemSet::on_resume(**self).with_system(show_board))
            .add_system_set(SystemSet::on_exit(**self).with_system(despawn::<Board>))
            .add_system(component_animator_system::<Visibility>)
            .add_event::<DeckCompletedEvent>()
            //        .add_plugin(MatPlugin(**self)) //mat
            .init_resource::<BoardAssets>()
            .init_resource::<BoardOptions>();
        #[cfg(feature = "debug")]
        {
            app.register_inspectable::<Idx>()
                .register_inspectable::<Open>()
                .register_inspectable::<Revealed>();
            log::info!("Loaded Board Plugin");
        }
    }
}

pub fn if_deck_not_done(deck: Option<Res<Deck>>) -> ShouldRun {
    if let Some(d) = deck {
        if d.outcome().is_none() {
            return ShouldRun::Yes;
        }
    }
    return ShouldRun::No;
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
#[autodefault(except(Board, TransformScaleLens))]
pub fn create_board(mut cmd: Commands, options: Res<BoardOptions>, assets: Res<BoardAssets>) {
    let count = options.deck_params().0;
    let deck_width = (2. * count as f32).sqrt().round();
    let players = options.create_players();
    let deck = Deck::init(options.deck_params(), options.mode, players.len() as u8);
    let size = options.card_size(deck_width, deck_width);
    #[cfg(feature = "debug")]
    {
        log::info!("{}", deck);
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
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::FlexStart,
                margin: if half == 0 {
                    Rect {
                        top: Val::Px(width / 2.),
                    }
                } else {
                    Rect {
                        bottom: Val::Px(width / 2.),
                    }
                },
                size: Size::new(Val::Px(width), Val::Percent(100.0)),
            }))
            .insert(Name::new(format!("Half: {}", half)))
            .with_children(|p| {
                for j in 0..count {
                    let (i, &card) = card_iter.next().unwrap();
                    let id = Idx(i, 0);
                    p.spawn_bundle(assets.board.node(Style {
                        min_size: Size {
                            width: Val::Px(size),
                            height: Val::Px(size),
                        },
                    }))
                    .insert(Name::new(format!("Card: {}", i)))
                    .with_children(|p| {
                        p.spawn_bundle(assets.card.button(Style {
                            margin: Rect::all(Val::Px(1.0)),
                            min_size: Size {
                                width: Val::Px(size),
                                height: Val::Px(size),
                            },
                        }))
                        .insert(Animator::new(seq(j)))
                        .insert(Name::new(format!("Card {:?}", i)))
                        .insert(id)
                        .with_children(|p| {
                            p.spawn_bundle(
                                assets.spawn_card(card, size),
                            )
                            .insert(Name::new("Card"));
                        });
                    });
                }
            });
        }
        p.spawn_bundle(assets.back_ground.node(Style {
            flex_basis: Val::Px(0.),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::FlexStart,
            size: Size::new(Val::Px(width), Val::Percent(100.0)),
        }))
        .insert(Name::new("Score Panel"))
        .with_children(|p| {
            players.iter().for_each(|pl| {
                p.spawn_bundle(assets.board.node(Style {
                    //flex_basis: Val::Px(0.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::FlexStart,
                    //size: Size::new(Val::Px(width), Val::Percent(100.0)),
                }))
                .with_children(|p| {
                    p.spawn_bundle(TextBundle {
                        style: Style {
                            flex_basis: Val::Px(0.),
                            ..Default::default()
                        },
                        text: Text {
                            sections: vec![TextSection {
                                value: format!(
                                    "{}\nOpened: 0\nTurns: 0\n{:?}",
                                    if pl.is_bot() { "Bot" } else { "Human" },
                                    pl
                                ),
                                style: TextStyle {
                                    color: Color::RED,
                                    font: assets.score_font.clone(),
                                    font_size: size,
                                },
                            }],
                            alignment: TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            },
                        },
                    })
                    .insert(*pl);
                });
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
