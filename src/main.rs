use {
    bevy::{
        log::{Level, LogSettings},
        prelude::*,
    },
    memory::*,
    menu_plugin::MenuMaterials,
};
//mod xp;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Game {
    Memory,
    Menu,
}

/// Timer to help start another game after completing one
#[bevy_main]
fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: "Recall Stones! - A Concentration Game".to_string(),
        width: if cfg!(feature="debug") {1080.}else{480.},
        height: 720.,
        ..Default::default()
    })
    .insert_resource(LogSettings {
        level: Level::INFO,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(MemoryGamePlugin(Game::Memory))
    .init_resource::<MenuMaterials>()
    .add_state(Game::Menu)
    //.add_startup_system(xp::setup_menu)
    .add_startup_system(startup);
    // Debug hierarchy inspector
    #[cfg(feature = "debug")]
    app.add_plugin(bevy_inspector_egui::InspectorPlugin::<MenuMaterials>::new());
    app.run();
}
/// Pre launch setup of assets and options
fn startup(mut commands: Commands, mut state: ResMut<State<Game>>) {
    // Camera
    commands.spawn_bundle(UiCameraBundle::default());
    state.set(Game::Memory).unwrap();
}
