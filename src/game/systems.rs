use bevy::{math::vec2, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_ggrs::*;
use bevy_matchbox::{prelude::SingleChannel, MatchboxSocket};
use rand::Rng;
use std::time::Duration;

use crate::random::Random;

use super::{
    data::{AnimationIndices, AnimationTimer, ExampleGameText, GgrsConfig, Player, Pos, Vel},
    effects::Flick,
    session_data::{INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT, INPUT_UP},
};

pub fn ggrs_input(_: In<ggrs::PlayerHandle>, keys: Res<Input<KeyCode>>) -> u8 {
    let mut input = 0u8;

    if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
        input |= INPUT_LEFT;
    }
    if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
        input |= INPUT_RIGHT;
    }
    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        input |= INPUT_UP;
    }
    if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
        input |= INPUT_DOWN;
    }

    input
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
        let mut direction = Vec2::ZERO;

        let (input, _) = inputs[player.handle];

        if input & INPUT_LEFT != 0 {
            direction.x -= 1.0;
        }
        if input & INPUT_RIGHT != 0 {
            direction.x += 1.0;
        }
        if input & INPUT_UP != 0 {
            direction.y += 1.0;
        }
        if input & INPUT_DOWN != 0 {
            direction.y -= 1.0;
        }

        let move_speed = 1.0;
        let move_delta = (direction * move_speed).extend(0.0);

        if direction == Vec2::ZERO {
            // update animation
            indices.first = 0;
            indices.last = 1;
            sprite.index = usize::clamp(sprite.index, indices.first, indices.last);
            timer.0.set_duration(Duration::from_millis(500));
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
        timer.0.set_duration(Duration::from_millis(200));
    }
}

pub fn wait_for_players(mut commands: Commands, mut socket: ResMut<MatchboxSocket<SingleChannel>>) {
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
}

pub fn example_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: Local<Random>,
) {
    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([TextSection::new(
            "~In Game~",
            TextStyle {
                font: asset_server.load("fonts/visitor.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        )])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
        Vel(vec2(rng.gen_range(1.0..1.5), rng.gen_range(1.0..1.5))),
        Pos(vec2(5.0, 15.0)),
        ExampleGameText,
        Flick {
            duration: Timer::from_seconds(60.0, TimerMode::Once),
            switch_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        },
    ));
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut rip: ResMut<RollbackIdProvider>,
) {
    let idle_handle = asset_server.load("textures/chars/char_atlas.png");
    let idle_atlas =
        TextureAtlas::from_grid(idle_handle, Vec2 { x: 16.0, y: 16.0 }, 4, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(idle_atlas);
    let anim_indices = AnimationIndices { first: 0, last: 1 };

    // Spawn player 1
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform {
                translation: Vec3::new(-16., 0., 0.0),
                scale: Vec3::new(3.0, 3.0, 3.0),
                ..default()
            },
            ..default()
        },
        anim_indices.clone(),
        AnimationTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
        Player { handle: 0 },
        rip.next(),
    ));

    // Spawn player 2
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
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
        AnimationTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
        Player { handle: 1 },
        rip.next(),
    ));
}

pub fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Size of the tile map in tiles.
    let map_size = TilemapSize { x: 32, y: 32 };

    // To create a map we use the TileStorage component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a Tilemap2dStorage
    // component per layer.
    let mut tile_storage = TileStorage::empty(map_size);

    // For the purposes of this example, we consider a tilemap with rectangular tiles.
    let map_type = TilemapType::Square;

    let tilemap_entity = commands.spawn_empty().id();

    // Spawn a 32 by 32 tilemap.
    // Alternatively, you can use helpers::fill_tilemap.
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
        }
    }
}

pub fn teardown(mut commands: Commands, texts: Query<(Entity, With<ExampleGameText>)>) {
    for (entity, _) in texts.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn example_update(
    window: Query<&Window>,
    mut texts: Query<(
        &mut Style,
        &CalculatedSize,
        &mut Pos,
        &mut Vel,
        With<ExampleGameText>,
    )>,
) {
    let window = window.get_single().unwrap();
    for (mut style, calculated_size, mut pos, mut vel, _) in texts.iter_mut() {
        pos.0.y += vel.0.y;
        pos.0.x += vel.0.x;

        if pos.0.y + calculated_size.size.y > window.height() {
            pos.0.y = window.height() - calculated_size.size.y;
            vel.0.y *= -1.0;
        } else if pos.0.y < 0.0 {
            pos.0.y = 0.0;
            vel.0.y *= -1.0;
        }
        if pos.0.x + calculated_size.size.x > window.width() {
            pos.0.x = window.width() - calculated_size.size.x;
            vel.0.x *= -1.0;
        } else if pos.0.x < 0.0 {
            pos.0.x = 0.0;
            vel.0.x *= -1.0;
        }

        style.position = UiRect {
            top: Val::Px(pos.0.y),
            left: Val::Px(pos.0.x),
            ..default()
        };
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
