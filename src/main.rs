use bevy::{
    log::{Level, LogSettings},
    prelude::*,
};
mod menu;
use menu::*;
use paper_plugin::{tween::*, *};
//mod xp;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    InGame,
    Menu,
    Splash,
}
/// Timer to help start another game after completing one
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
    .add_plugin(TweeningPlugin);
    // Debug hierarchy inspector
    #[cfg(feature = "debug")]
    {
        app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
        //app.register_inspectable::<ButtonAction>();
        //app.register_inspectable::<Mode>();
    }
    // Board plugin
    app.add_plugin(PaperPlugin(AppState::InGame))
        .add_plugin(MenuPlugin)
        .init_resource::<MenuMaterials>() //to be removed
        .add_state(AppState::Splash)
        .add_startup_system(startup)
        .add_system(on_completion)
        .add_system(restart_game_on_timer)
        //.add_startup_system(xp::setup_menu)
        .add_system(component_animator_system::<UiColor>)
        // Run the app
        .run();
}
/// Pre launch setup of assets and options
fn startup(mut commands: Commands, mut state: ResMut<State<AppState>>) {
    // Camera
    commands.spawn_bundle(UiCameraBundle::default());
    state.set(AppState::Menu).unwrap();
}
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct RestartTimer(Timer);
/// Display Menu for 3 seconds before applying the set options
fn restart_game_on_timer(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut RestartTimer)>,
    buttons: Query<(Entity, &ButtonAction)>,
) {
    for (entity, mut timer) in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            if state.current() != &AppState::InGame {
                state.replace(AppState::InGame).unwrap();
            }
            commands.entity(entity).despawn_recursive();
        }
        if timer.0.percent() < 0.027 {
            for (entity, &button) in buttons.iter() {
                if button == ButtonAction::Apply {
                    commands
                        .entity(entity)
                        .insert(Animator::new(Tween::<UiColor>::new(
                            EaseFunction::QuadraticIn,
                            TweeningType::Once,
                            std::time::Duration::from_secs(2),
                            BeTween::with_lerp(|t: &mut UiColor, _, r| {
                                t.0 = Vec4::from(Color::RED)
                                    .lerp(Vec4::from(Color::GREEN), r)
                                    .into()
                            }),
                        )));
                }
            }
        }
    }
}
fn on_completion(
    mut state: ResMut<State<AppState>>,
    mut commands: Commands,
    mut board_complete_evr: EventReader<DeckCompletedEvent>,
) {
    for _ev in board_complete_evr.iter() {
        state.push(AppState::Menu).unwrap();
        commands
            .spawn()
            .insert(RestartTimer(Timer::from_seconds(3., false)));
    }
}
