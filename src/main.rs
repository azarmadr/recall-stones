mod buttons;

use crate::buttons::{ButtonAction, ButtonColors};
use autodefault::autodefault;
use bevy::log;
use bevy::log::{Level, LogSettings};
use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy_inspector_egui::RegisterInspectable;
use paper_plugin::{
    events::DeckCompletedEvent, Board, BoardAssets, BoardOptions, BoardPosition, Collection,
    PaperPlugin, SpriteMaterial,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    InGame,
    Menu,
    Out,
}

#[derive(Component)]
pub struct RestartTimer(Timer);

fn main() {
    let mut app = App::new();
    // Window setup
    app.insert_resource(WindowDescriptor {
        title: "Recall the Stones!".to_string(),
        width: 720.,
        height: 1080.,
        ..Default::default()
    })
    // Log setup
    .insert_resource(LogSettings {
        level: Level::INFO,
        ..Default::default()
    })
    // Bevy default plugins
    .add_plugins(DefaultPlugins)
    .add_plugin(bevy_tweening::TweeningPlugin);
    // Debug hierarchy inspector
    #[cfg(feature = "debug")]
    {
        app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
        app.register_inspectable::<ButtonAction>();
    }
    // Board plugin
    app.add_plugin(PaperPlugin {
        running_state: AppState::InGame,
    })
    .add_state(AppState::Out)
    .add_startup_system(setup_board)
    // Startup system (cameras)
    .add_startup_system(setup_camera)
    // UI
    .add_startup_system(setup_ui)
    // State handling
    .add_system(input_handler)
    .add_system(on_completion)
    .add_system(restart_game_on_timer)
    // Run the app
    .run();
}

fn setup_board(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    // Board plugin options
    commands.insert_resource(BoardOptions {
        deck_params: (2, 4, 2),
        card_padding: 2.,
        position: BoardPosition::Centered {
            offset: Vec3::new(0., 25., 0.),
        },
        ..Default::default()
    });
    // Board assets
    commands.insert_resource(BoardAssets {
        label: "Default".to_string(),
        board_material: SpriteMaterial {
            color: Color::WHITE,
            ..Default::default()
        },
        card_material: SpriteMaterial {
            color: Color::DARK_GRAY,
            ..Default::default()
        },
        covered_card_material: SpriteMaterial {
            color: Color::GRAY,
            ..Default::default()
        },
        counter_font: asset_server.load("fonts/pixeled.ttf"),
        card_color: BoardAssets::default_colors(),
        material: SpriteMaterial {
            texture: asset_server.load("sprites/bomb.png"),
            color: Color::WHITE,
        },
        col_map: HashMap::from([
            (
                Collection::Eng,
                asset_server.load_untyped("fonts/pixeled.ttf"),
            ),
            /*(
                Collection::Dice,
                asset_server.load_untyped("fonts/Dicier-Block-Heavy.ttf"),
            ),*/
            (
                Collection::Clubs,
                asset_server.load_untyped("fonts/clubs.ttf"),
            ),
            (
                Collection::Hearts,
                asset_server.load_untyped("fonts/hearts.ttf"),
            ),
            (
                Collection::Spades,
                asset_server.load_untyped("fonts/spades.ttf"),
            ),
            (
                Collection::Diamonds,
                asset_server.load_untyped("fonts/diamonds.ttf"),
            ),
            (
                Collection::Tel,
                asset_server.load_untyped("fonts/RaviPrakash.ttf"),
            ),
        ]),
    });
    // Launch game
    state.set(AppState::InGame).unwrap();
}

fn setup_camera(mut commands: Commands) {
    // 2D orthographic camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // UI Camera
    commands.spawn_bundle(UiCameraBundle::default());
}

#[allow(clippy::type_complexity)]
fn input_handler(
    button_colors: Res<ButtonColors>,
    mut interaction_query: Query<
        (&Interaction, &ButtonAction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut board_options: ResMut<BoardOptions>,
    mut state: ResMut<State<AppState>>,
) {
    for (interaction, action, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = button_colors.pressed.into();
                match action {
                    /*
                    ButtonAction::Clear => {
                    log::debug!("clearing detected");
                    log::info!("clearing game");
                    match state.current() {
                    AppState::InGame => state.set(AppState::Out).unwrap(),
                    AppState::Menu => state.replace(AppState::Out).unwrap(),
                    _ => (),
                    }
                    }
                    */
                    ButtonAction::LevelUp => {
                        log::debug!("LevelUp");
                        log::info!("LevelUp");
                        board_options.deck_params.0 += 1;
                        board_options.deck_params.1 += 1;
                    }
                    ButtonAction::LevelDown => {
                        log::debug!("LevelDown");
                        log::info!("LevelDown");
                        if board_options.deck_params.0 > 2 {
                            board_options.deck_params.0 -= 1;
                            board_options.deck_params.1 -= 1;
                        }
                    }
                    ButtonAction::CoupletUp => {
                        log::debug!("CoupletUp");
                        log::info!("CoupletUp");
                        if board_options.deck_params.2 < 5 {
                            board_options.deck_params.2 += 1;
                        }
                    }
                    ButtonAction::CoupletDown => {
                        log::debug!("CoupletDown");
                        log::info!("CoupletDown");
                        if board_options.deck_params.2 > 2 {
                            board_options.deck_params.2 -= 1;
                        }
                    }
                    ButtonAction::Generate => {
                        log::debug!("loading detected");
                        log::info!("loading game");
                        match state.current() {
                            AppState::Out => state.set(AppState::InGame).unwrap(),
                            AppState::Menu => state.replace(AppState::InGame).unwrap(),
                            _ => {
                                //state.push(AppState::Out).unwrap();
                                //state.replace(AppState::InGame).unwrap();
                            }
                        }
                    } //_ => ()
                }
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

fn restart_game_on_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut RestartTimer)>,
    mut state: ResMut<State<AppState>>,
) {
    for (entity, mut timer) in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            if state.current() != &AppState::InGame {
                state.replace(AppState::InGame).unwrap();
            }
            commands.entity(entity).despawn_recursive();
        }
    }
}
fn on_completion(
    mut state: ResMut<State<AppState>>,
    board: Option<Res<Board>>,
    mut commands: Commands,
    mut board_options: ResMut<BoardOptions>,
    mut board_complete_evr: EventReader<DeckCompletedEvent>,
) {
    for _ev in board_complete_evr.iter() {
        state.push(AppState::Menu).unwrap();
        if let Some(b) = &board {
            if b.turns < 2 * b.deck.len() as u32 {
                board_options.deck_params.0 += 1;
                board_options.deck_params.1 += 2;
                if board_options.col_is_suites() {
                    if board_options.deck_params.0 > 14 {
                        board_options.deck_params.0 = 14;
                    }
                    if board_options.deck_params.1 > 14 {
                        board_options.deck_params.1 = 14;
                    }
                }
            }
        }
        commands
            .spawn()
            .insert(RestartTimer(Timer::from_seconds(3., false)));
    }
}

#[autodefault]
fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_materials = ButtonColors {
        normal: Color::GRAY,
        hovered: Color::DARK_GRAY,
        pressed: Color::BLACK,
    };
    let font = asset_server.load("fonts/pixeled.ttf");
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Px(150.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                position: Rect { top: Val::Px(0.) },
            },
            color: Color::BLACK.into(),
        })
    .insert(Name::new("Instructions"))
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![
                        write_strings("Instructions:",27.,Color::WHITE,font.clone()),
                        write_strings("1. Match a card with its couplets",23.,Color::WHITE,font.clone()),
                        write_strings("if couplet is two, then two cards have same value which can be matched",17.,Color::WHITE,font.clone()),
                    ],
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Left,
                    },
                },
            });
        });
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Px(50.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            },
            color: Color::WHITE.into(),
        })
        .insert(Name::new("UI"))
        .with_children(|parent| {
            /*
            setup_single_menu(
            parent,
            "CLEAR",
            button_materials.normal.into(),
            font.clone(),
            ButtonAction::Clear,
            );
            */
            setup_single_menu(
                parent,
                "LevelUp",
                button_materials.normal.into(),
                font.clone(),
                ButtonAction::LevelUp,
            );
            setup_single_menu(
                parent,
                "LevelDown",
                button_materials.normal.into(),
                font.clone(),
                ButtonAction::LevelDown,
            );
            setup_single_menu(
                parent,
                "CoupletUp",
                button_materials.normal.into(),
                font.clone(),
                ButtonAction::CoupletUp,
            );
            setup_single_menu(
                parent,
                "CoupletDown",
                button_materials.normal.into(),
                font.clone(),
                ButtonAction::CoupletDown,
            );
            setup_single_menu(
                parent,
                "GENERATE",
                button_materials.normal.into(),
                font,
                ButtonAction::Generate,
            );
        });
    commands.insert_resource(button_materials);
}

#[autodefault]
fn setup_single_menu(
    parent: &mut ChildBuilder,
    text: &str,
    color: UiColor,
    font: Handle<Font>,
    action: ButtonAction,
) {
    parent
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(95.), Val::Auto),
                margin: Rect::all(Val::Px(10.)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            },
            color,
        })
        .insert(action)
        .insert(Name::new(text.to_string()))
        .with_children(|builder| {
            builder.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: text.to_string(),
                        style: TextStyle {
                            font,
                            font_size: 27.,
                            color: Color::WHITE,
                        },
                    }],
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                },
            });
        });
}
fn write_strings(text: &str, font_size: f32, color: Color, font: Handle<Font>) -> TextSection {
    TextSection {
        value: (text.to_owned() + "\n").to_string(),
        style: TextStyle {
            font,
            font_size,
            color,
        },
    }
}
