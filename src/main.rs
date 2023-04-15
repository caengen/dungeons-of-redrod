use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_toggle_active,
    log::{Level, LogPlugin},
    prelude::*,
    window::PresentMode,
    DefaultPlugins,
};
use bevy_ggrs::GGRSPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_matchbox::MatchboxSocket;
use config::Debug;
use game::GamePlugin;
use main_menu::*;
use rand::Rng;
use random::{Random, RandomPlugin};
use std::{env, process};

mod config;
mod game;
mod main_menu;
mod random;

pub const SCREEN: Vec2 = Vec2::from_array([512.0, 512.0]);
pub const DARK: Color = Color::rgb(0.191, 0.184, 0.156);
pub const LIGHT: Color = Color::rgb(0.852, 0.844, 0.816);

#[derive(States, Hash, Clone, PartialEq, Eq, Debug, Default)]
pub enum AppState {
    MainMenu,
    #[default]
    InGame,
}

/**
 * The configuration for the game loop. For cleanliness
 */
fn main() {
    // Possibility for program args
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = config::ProgramConfig::build(&args).unwrap_or_else(|err| {
        println!("A problem occured when parsing args: {err}");
        process::exit(1);
    });

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Dungeons of Redrod".into(),
                    resolution: (SCREEN.x, SCREEN.y).into(),
                    present_mode: PresentMode::AutoNoVsync,
                    // Tells wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                level: Level::DEBUG,
                filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_state::<AppState>()
    .insert_resource(Debug(cfg.debug))
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_plugin(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)))
    .add_plugin(RandomPlugin)
    .add_plugin(MainMenuPlugin)
    .add_plugin(GamePlugin)
    .add_startup_systems((setup, start_matchbox_socket));

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(DARK),
        },
        ..default()
    });
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/dungeons_of_redrod?next=2";
    info!("connecting to matchbox server: {:?}", room_url);
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
}
