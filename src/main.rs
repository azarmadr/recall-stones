use bevy::log::{Level, LogSettings};
use bevy::prelude::*;

use bevy::log;
#[cfg(feature = "debug")]
use bevy_inspector_egui::WorldInspectorPlugin;
use paper_plugin::{events::DeckCompletedEvent, PaperPlugin, BoardAssets, BoardOptions, BoardPosition, SpriteMaterial};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    InGame,
    Menu,
    Out,
}

fn main() {
    let mut app = App::new();
    // Window setup
    app.insert_resource(WindowDescriptor {
        title: "Recall the Stones!".to_string(),
        width: 700.,
        height: 800.,
        ..Default::default()
    })
    // Log setup
    .insert_resource(LogSettings {
        level: Level::INFO,
        ..Default::default()
    })
    // Bevy default plugins
    .add_plugins(DefaultPlugins)
    // Board plugin options
    .insert_resource(BoardOptions {
        ..Default::default()
    })
    // Board plugin
    .add_plugin(PaperPlugin {
        running_state: AppState::InGame,
    })
    .add_state(AppState::Out)
    .add_startup_system(setup_board)
    // Startup system (cameras)
    .add_startup_system(setup_camera)
    .add_system(state_handler);
    #[cfg(feature = "debug")]
    // Debug hierarchy inspector
    app.add_plugin(WorldInspectorPlugin::new());
    // Run the app
    app.run();
}

fn setup_board(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    // Board plugin options
    commands.insert_resource(BoardOptions {
        deck_size: (7, 20),
        max_limit: 50,
        card_padding: 2.,
        safe_start: true,
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
        counter_colors: BoardAssets::default_colors(),
        flag_material: SpriteMaterial {
            texture: asset_server.load("sprites/flag.png"),
            color: Color::WHITE,
        },
        material: SpriteMaterial {
            texture: asset_server.load("sprites/bomb.png"),
            color: Color::WHITE,
        },
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

fn state_handler(
    mut state: ResMut<State<AppState>>,
    keys: Res<Input<KeyCode>>,
    mut board_complete_evr: EventReader<DeckCompletedEvent>,
) {
    if keys.just_pressed(KeyCode::C) {
        log::debug!("clearing detected");
        if state.current() == &AppState::InGame {
            log::info!("clearing game");
            state.set(AppState::Out).unwrap();
        }
    }
    if keys.just_pressed(KeyCode::G) {
        log::debug!("loading detected");
        if state.current() == &AppState::Out {
            log::info!("loading game");
            state.set(AppState::InGame).unwrap();
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        log::debug!("toggle menu");
        match state.current() {
            &AppState::InGame => state.push(AppState::Menu).unwrap(),
            &AppState::Menu => state.pop().unwrap(),
            _ => (),
        }
    }
    for _ev in board_complete_evr.iter() {
        state.push(AppState::Menu).unwrap();
    }
}
