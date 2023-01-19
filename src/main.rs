use {bevy::prelude::*, memory::*, menu_plugin::MenuMaterials, std::time::Duration};

#[cfg(feature = "dev")]
use bevy_inspector_egui::WorldInspectorPlugin;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Game {
    Memory,
    Menu,
}

/// Timer to help start another game after completing one
#[bevy_main]
pub fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            title: "Recall Stones! - A Concentration Game".to_string(),
            width: 480.,
            height: 720.,
            ..Default::default()
        },
        ..default()
    }))
    .init_resource::<MenuMaterials>();

    #[cfg(target_arch = "wasm32")]
    app.add_system(handle_browser_resize);

    app.add_plugin(MemoryGamePlugin(Game::Memory))
        .add_state(Game::Menu)
        .add_system(game_timer)
        .add_startup_system(startup);

    // Debug hierarchy inspector
    #[cfg(feature = "dev")]
    app.add_plugin(WorldInspectorPlugin::new());

    app.run();
}
/// Pre launch setup of assets and options
fn startup(mut commands: Commands, mut menu: ResMut<MenuMaterials>, mut windows: ResMut<Windows>) {
    commands.spawn(Camera2dBundle::default());
    let window = windows.primary_mut();
    // window.set_maximized(true);
    menu.size = window
        .requested_width()
        .min(0.8 * window.requested_height());
    // menu.size = window.physical_width().min(window.physical_height()) as f32;
}

fn game_timer(mut state: ResMut<State<Game>>, time: Res<Time>, mut timer: Local<Timer>) {
    if timer.duration() == Duration::ZERO {
        timer.set_duration(Duration::from_millis(27));
    }
    if timer.tick(time.delta()).just_finished() {
        state.replace(Game::Memory).unwrap();
    }
}

#[cfg(target_arch = "wasm32")]
fn handle_browser_resize(mut windows: ResMut<Windows>, mut menu: ResMut<MenuMaterials>) {
    let window = windows.get_primary_mut().unwrap();
    let wasm_window = web_sys::window().unwrap();
    let (target_width, target_height) = (
        wasm_window.inner_width().unwrap().as_f64().unwrap() as f32,
        wasm_window.inner_height().unwrap().as_f64().unwrap() as f32,
    );
    if window.width() != target_width || window.height() != target_height {
        window.set_resolution(target_width, target_height);
    }
    menu.size = window
        .requested_width()
        .min(window.requested_height() * 0.8);
}
