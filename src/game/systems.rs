use bevy::{math::vec2, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_ggrs::{ggrs::PlayerType, *};
use bevy_matchbox::{prelude::SingleChannel, MatchboxSocket};
use bevy_turborand::{GlobalRng, RngComponent};
use std::time::Duration;

use crate::{game::components::LocalPlayerHandle, FontAssets, GameState, ImageAssets};

use super::{
    components::{AnimationIndices, AnimationTimer, ExampleGameText, GgrsConfig, Player, Pos, Vel},
    effects::Flick,
    input::direction,
    levels,
};

pub fn camera_follow(
    player_handle: Option<Res<LocalPlayerHandle>>,
    player_query: Query<(&Player, &Transform)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_handle = match player_handle {
        Some(player_handle) => player_handle.0,
        None => return, // no local player yet
    };

    for (player, player_transform) in player_query.iter() {
        if player.handle != player_handle {
            continue;
        }

        let pos = player_transform.translation;
        for mut camera_transform in camera_query.iter_mut() {
            camera_transform.translation.x = pos.x;
            camera_transform.translation.y = pos.y;
        }
    }
}

pub fn move_players(
    inputs: Res<PlayerInputs<GgrsConfig>>,
    mut player: Query<(
        &Player,
        &mut Transform,
        &mut AnimationIndices,
        &mut TextureAtlasSprite,
        &mut AnimationTimer,
    )>,
) {
    for (player, mut transform, mut indices, mut sprite, mut timer) in player.iter_mut() {
        let (input, _) = inputs[player.handle];
        let direction = direction(input);
        let move_speed = 2.0;
        let move_delta = (direction * move_speed).extend(0.0);

        if direction == Vec2::ZERO {
            // update animation
            indices.first = 0;
            indices.last = 1;
            sprite.index = usize::clamp(sprite.index, indices.first, indices.last);
            timer.0.set_duration(Duration::from_millis(250));
            continue;
        }

        transform.translation += move_delta;

        // update animatio
        indices.first = 2;
        indices.last = 3;
        sprite.index = usize::clamp(sprite.index, indices.first, indices.last);
        if move_delta.x < 0.0 {
            sprite.flip_x = true;
        } else if move_delta.x > 0.0 {
            sprite.flip_x = false;
        }
        timer.0.set_duration(Duration::from_millis(150));
    }
}

pub fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if socket.get_channel(0).is_err() {
        return; // we've already started
    }

    // check for new connections
    socket.update_peers();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(num_players)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        if player == PlayerType::Local {
            commands.insert_resource(LocalPlayerHandle(i));
        }
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the channel out of the socket (required because GGRS takes ownership of it)
    let channel = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2PSession(ggrs_session));

    // transition to the state InGame
    next_state.set(GameState::InGame);
}

pub fn spawn_player(
    mut commands: Commands,
    images: Res<ImageAssets>,
    mut rip: ResMut<RollbackIdProvider>,
) {
    let anim_indices = AnimationIndices { first: 0, last: 1 };

    // Spawn player 1
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: images.char_idle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform {
                translation: Vec3::new(-16., 0., 0.0),
                scale: Vec3::new(3.0, 3.0, 3.0),
                ..default()
            },
            ..default()
        },
        anim_indices.clone(),
        AnimationTimer(Timer::from_seconds(0.25, TimerMode::Repeating)),
        Player { handle: 0 },
        rip.next(),
    ));

    // Spawn player 2
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: images.char_idle.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                color: Color::rgb(0.5, 0.5, 1.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(16., 0., 0.0),
                scale: Vec3::new(3.0, 3.0, 3.0),
                ..default()
            },
            ..default()
        },
        anim_indices,
        AnimationTimer(Timer::from_seconds(0.25, TimerMode::Repeating)),
        Player { handle: 1 },
        rip.next(),
    ));
}

pub fn setup_level(
    mut commands: Commands,
    images: Res<ImageAssets>,
    mut global_rng: ResMut<GlobalRng>,
) {
    // Size of the tile map in tiles.
    let map_size = TilemapSize { x: 64, y: 64 };

    // To create a map we use the TileStorage component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a Tilemap2dStorage
    // component per layer.
    // Layer 1
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    // fill_tilemap(
    //     TileTextureIndex(10),
    //     map_size,
    //     TilemapId(tilemap_entity),
    //     &mut commands,
    //     &mut tile_storage,
    // );

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    let mut rng = RngComponent::from(&mut global_rng);
    levels::cave(
        &mut rng,
        &mut commands,
        &tilemap_entity,
        &map_size,
        &mut tile_storage,
    );

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(images.atlas.clone()),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

pub fn teardown(mut commands: Commands, texts: Query<(Entity, With<ExampleGameText>)>) {
    for (entity, _) in texts.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}
