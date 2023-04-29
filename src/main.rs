use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_toggle_active,
    log::{Level, LogPlugin},
    prelude::*,
    window::PresentMode,
    DefaultPlugins,
};
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_matchbox::MatchboxSocket;
use bevy_turborand::prelude::*;
use config::Debug;
use game::GamePlugin;
use main_menu::*;
use std::{env, process};

mod config;
mod game;
mod main_menu;

pub const SCREEN: Vec2 = Vec2::from_array([512.0, 512.0]);
pub const DARK: Color = Color::rgb(0.191, 0.184, 0.156);
pub const LIGHT: Color = Color::rgb(0.852, 0.844, 0.816);
// #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 7, rows = 3))]

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 4, rows = 1))]
    #[asset(path = "textures/chars/char_atlas.png")]
    pub char_idle: Handle<TextureAtlas>,
    #[asset(path = "textures/atlas.png")]
    pub atlas: Handle<Image>,
}

#[derive(AssetCollection, Resource)]

pub struct FontAssets {
    #[asset(path = "fonts/visitor.ttf")]
    pub visitor: Handle<Font>,
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum GameState {
    MainMenu,
    #[default]
    AssetLoading,
    Matchmaking,
    InGame,
}

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
                level: Level::ERROR,
                filter: "game=info".to_string(), //wgpu=error,bevy_render=info,bevy_ecs=trace
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_state::<GameState>()
    .add_loading_state(
        LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Matchmaking),
    )
    .add_collection_to_loading_state::<_, ImageAssets>(GameState::AssetLoading)
    .add_collection_to_loading_state::<_, FontAssets>(GameState::AssetLoading)
    .insert_resource(Debug(cfg.debug))
    .add_plugin(TilemapPlugin)
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_plugin(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)))
    .add_plugin(RngPlugin::default())
    .add_plugin(MainMenuPlugin)
    .add_plugin(GamePlugin)
    // todo: consider moving this to a seperate plugin "MatchmakingPlugin"
    .add_systems((setup, start_matchbox_socket).in_schedule(OnEnter(GameState::Matchmaking)));

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Default,
        },
        ..default()
    });
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/dungeons_of_redrod?next=2";
    info!("connecting to matchbox server: {:?}", room_url);
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
}
