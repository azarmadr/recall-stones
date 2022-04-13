use bevy::{log::{Level, LogSettings},prelude::*};
use bevy_tweening::{lens::*, *};
mod menu;
use menu::*;
use paper_plugin::{
    events::DeckCompletedEvent, Board, BoardAssets, BoardOptions, BoardPosition, Collection,
    PaperPlugin, SpriteMaterial,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    InGame,
    Menu,
    Splash,
}
/// Timer to help start another game after completing one
#[derive(Component)]
pub struct RestartTimer(Timer);
/// A lens to manipulate the [`color`] field of a [`UiColor`] asset.
/// [`color`]: https://docs.rs/bevy/0.6.1/bevy/sprite/struct.Sprite.html#structfield.color
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UiColorLens {
    /// Start color.
    pub start: Color,
    /// End color.
    pub end: Color,
}
impl Lens<UiColor> for UiColorLens {
    fn lerp(&mut self, target: &mut UiColor, ratio: f32) {
        // Note: Add<f32> for Color affects alpha, but not Mul<f32>. So use Vec4 for consistency.
        let start: Vec4 = self.start.into();
        let end: Vec4 = self.end.into();
        let value = start.lerp(end, ratio);
        target.0 = value.into();
    }
}
fn main() {
    let mut app = App::new();
    // Window setup
    app.insert_resource(WindowDescriptor {
        title: "Recall Stones! - A Concentration Game".to_string(),
        width: 480.,
        height: 720.,
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
        //app.register_inspectable::<ButtonAction>();
        //app.register_inspectable::<Mode>();
    }
    // Board plugin
    app.add_plugin(PaperPlugin {
        running_state: AppState::InGame,
    })
    .add_plugin(MenuPlugin)
    .init_resource::<MenuMaterials>() //to be removed
    .add_state(AppState::Splash)
    .add_startup_system(startup)
    .add_system(button_system)
    .add_system(on_completion)
    .add_system(restart_game_on_timer)
    .add_system(component_animator_system::<UiColor>)
    // Run the app
    .run();
}
/// Pre launch setup of assets and options
fn startup(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    commands.insert_resource(BoardOptions {
        card_padding: 2.,
        position: BoardPosition::Centered {
            offset: Vec3::new(0., 25., 0.),
        },
        ..Default::default()
    });
    commands.insert_resource(BoardAssets {
        board_material: SpriteMaterial {
            color: Color::WHITE,
            ..Default::default()
        },
        card_material: SpriteMaterial {
            color: Color::DARK_GRAY,
            ..Default::default()
        },
        score_font: asset_server.load("fonts/pixeled.ttf"),
        card_color: BoardAssets::default_colors(),
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
    state.set(AppState::InGame).unwrap();
}
/// Display Menu for 3 seconds before applying the set options
fn restart_game_on_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut RestartTimer)>,
    mut state: ResMut<State<AppState>>,
    buttons: Query<(Entity,&ButtonAction)>
) {
    for (entity, mut timer) in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            if state.current() != &AppState::InGame {
                state.replace(AppState::InGame).unwrap();
            }
            commands.entity(entity).despawn_recursive();
        }
        if timer.0.percent() < 0.027 {
        for (entity, &button) in buttons.iter(){
            if button == ButtonAction::Apply {
        commands.entity(entity)
            .insert(Animator::new(Tween::new(EaseFunction::QuadraticIn,TweeningType::Once,std::time::Duration::from_secs(2),UiColorLens{start:Color::RED,end:Color::GREEN})));
            }}}
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
                board_options.level_up();
            }
        }
        commands
            .spawn()
            .insert(RestartTimer(Timer::from_seconds(3., false)));
    }
}
