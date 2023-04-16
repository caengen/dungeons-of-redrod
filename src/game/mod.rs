use self::{
    components::{GgrsConfig, PhysicsSet},
    effects::flick_system,
    input::ggrs_input,
    systems::{
        animate_sprite, camera_follow, example_setup, example_update, move_players, spawn_player,
        teardown, wait_for_players,
    },
};
use crate::GameState;
use bevy::prelude::*;
use bevy_ggrs::{GGRSPlugin, GGRSSchedule};

mod collision;
mod components;
mod effects;
mod input;
mod systems;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        GGRSPlugin::<GgrsConfig>::new()
            .with_input_system(ggrs_input)
            .register_rollback_component::<Transform>()
            .build(app);

        app.add_systems((
            example_setup.in_schedule(OnEnter(GameState::InGame)),
            spawn_player.in_schedule(OnEnter(GameState::InGame)),
        ))
        .add_systems((
            wait_for_players.run_if(in_state(GameState::Matchmaking)),
            camera_follow.run_if(in_state(GameState::InGame)),
            animate_sprite.run_if(in_state(GameState::InGame)),
        ))
        .add_systems(
            (move_players, example_update, flick_system)
                .chain()
                .in_schedule(GGRSSchedule),
        )
        .configure_set(PhysicsSet::Movement.before(PhysicsSet::CollisionDetection))
        .add_system(teardown.in_schedule(OnExit(GameState::InGame)));
    }
}
