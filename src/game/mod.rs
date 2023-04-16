use self::{
    components::{GgrsConfig, PhysicsSet},
    effects::flick_system,
    input::ggrs_input,
    systems::{
        animate_sprite, example_setup, example_update, move_players, spawn_player, teardown,
        wait_for_players,
    },
};
use crate::AppState;
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

        app.add_systems((example_setup, spawn_player).in_schedule(OnEnter(AppState::InGame)))
            .add_systems(
                (move_players, animate_sprite, example_update, flick_system)
                    .chain()
                    .in_set(OnUpdate(AppState::InGame))
                    .in_schedule(GGRSSchedule),
            )
            .add_system(wait_for_players)
            .configure_set(PhysicsSet::Movement.before(PhysicsSet::CollisionDetection))
            .add_system(teardown.in_schedule(OnExit(AppState::InGame)));
    }
}
