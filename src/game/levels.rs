use bevy::{
    math::vec2,
    prelude::{Commands, Component, Entity, Local, ResMut, Vec2},
};
use bevy_ecs_tilemap::{
    helpers::square_grid::neighbors::{self, Neighbors},
    prelude::{TilemapId, TilemapSize},
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
    TilemapBundle,
};
use bevy_turborand::{DelegatedRng, GlobalRng, RngComponent};
use std::ops::Add;
use std::ops::Range;

// rad 1 er 0 .. 22
// rad 2 er 23 .. 45
// rad 3 er 46 .. 68
// rad 4 er 69 .. 91
// rad 5 er 92 .. 114
// rad 6 er 115 .. 137
// rad 7 er 138 .. 160
// rad 8 er 161 .. 183
// rad 9 er 184 .. 206
// rad 10 er 207 .. 229
// rad 11 er 230 .. 252
// rad 12 er 253 .. 275
// rad 13 er 276 .. 298
// rad 14 er 299 .. 321
// rad 15 er 322 .. 344
// rad 16 er 345 .. 367
// rad 17 er 368 .. 390
// rad 18 er 391 .. 413
// rad 19 er 414 .. 436
// rad 20 er 437 .. 459
// rad 21 er 460 .. 482
// rad 22 er 483 .. 505
// rad 23 er 506 .. 528
// rad 24 er 529 .. 551
// rad 25 er 552 .. 574
// rad 26 er 575 .. 597
// rad 27 er 598 .. 620
// rad 28 er 621 .. 643
// rad 29 er 644 .. 666
// rad 30 er 667 .. 689
// rad 31 er 690 .. 712
// rad 32 er 713 .. 735
// rad 33 er 736 .. 758

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum CoarseTileType {
    Wall,
    Floor,
    Door,
    Dirt,
}
#[derive(Component, Debug, Clone, PartialEq, Eq, derive_more::From)]
pub enum CaveAtlasIndices {
    Wall1RightBottomLeft = 0,
    Wall1TopBottomLeft = 1,
    Wall1RightBottom = 2,
    Wall1RightLeft = 3,
    Wall1BottomLeft = 4,
    Wall1TopRightBottomLeft = 5,
    Wall1TopRightBottom = 23,
    Wall1TopRightLeft = 24,
    Wall1TopRight = 25,
    Wall1Left = 26,
    Wall1TopBottom = 27,
    Wall1Top = 46,
    Wall1Bottom = 47,
    Wall1Right = 48,
    Wall1TopLeft = 50,
    Wall1Gate = 51,
    Wall1Rubble1 = 28,
    Wall1Rubble2 = 29,
    Floor1_1 = 8,
    Floor2_1 = 9,
    Floor2_2 = 31,
    Floor1_2 = 32,
    Floor1_3 = 54,
    Floor2_3 = 55,
}

const ROOM_GENERATION_ATTEMPTS: i32 = 50;
const CORRIDOR_MAX_LENGTH: usize = 20;
pub static ROOM_SIZES: [TilePos; 2] = [TilePos { x: 5, y: 5 }, TilePos { x: 5, y: 7 }];
pub struct Room {
    pub pos: TilePos,
    pub size: TilePos,
}
impl Room {
    pub fn new(pos: TilePos, size: TilePos) -> Room {
        Room { pos, size }
    }
    pub fn intersects(&self, other: &Room) -> bool {
        let left = u32::max(self.pos.x, other.pos.x);
        let right = u32::min(self.pos.x + self.size.x, other.pos.x + other.size.x);
        let top = u32::max(self.pos.y, other.pos.y);
        let bottom = u32::min(self.pos.y + self.size.y, other.pos.y + other.size.y);

        left < right && top < bottom
    }
}

pub struct Map {
    pub size: TilemapSize,
    pub tiles: Vec<CoarseTileType>,
}

impl Map {
    pub fn idx_to_vec2(&self, idx: usize) -> Vec2 {
        Vec2 {
            x: idx as f32 % self.size.x as f32,
            y: (idx as f32 / self.size.x as f32).floor(),
        }
    }

    pub fn tile_at_pos(&self, pos: TilePos) -> Option<&CoarseTileType> {
        let idx = pos.to_index(&self.size);
        if pos.x < 0 || pos.y < 0 || idx >= self.tiles.len() {
            return None;
        }

        Some(&self.tiles[idx])
    }
}

pub fn is_floor(tile: &CoarseTileType) -> bool {
    match tile {
        CoarseTileType::Floor => true,
        _ => false,
    }
}

pub fn is_room(tile: &CoarseTileType) -> bool {
    match tile {
        CoarseTileType::Floor | CoarseTileType::Wall => true,
        _ => false,
    }
}

pub fn get_tile_at_pos(map: &Map, pos: TilePos) -> Option<&CoarseTileType> {
    let idx = pos.to_index(&map.size);
    map.tiles.get(idx)
}

pub fn surrounding_tiles(map: &Map, idx: usize) -> Vec<Option<&CoarseTileType>> {
    let adjecent_vecs = vec![
        vec2(-1.0, -1.0), // north west
        vec2(0.0, -1.0),  // north
        vec2(1.0, -1.0),  // north east
        vec2(-1.0, 0.0),  // west
        vec2(1.0, 0.0),   // east
        vec2(-1.0, 1.0),  // south east
        vec2(0.0, 1.0),   // south
        vec2(1.0, 1.0),   // south west
    ];

    adjecent_vecs
        .iter()
        .map(|v| {
            let pos = map.idx_to_vec2(idx).add(*v);
            map.tile_at_pos(TilePos {
                x: pos.x as u32,
                y: pos.y as u32,
            })
        })
        .collect()
}
pub fn surrounding_idxs(map: &Map, idx: usize) -> Vec<usize> {
    let adjecent_vecs = vec![
        vec2(-1.0, -1.0), // top left corner
        vec2(0.0, -1.0),  // top center
        vec2(1.0, -1.0),  // top right corner
        vec2(-1.0, 0.0),  // left
        vec2(1.0, 0.0),   // right
        vec2(-1.0, 1.0),  // bottom left corner
        vec2(0.0, 1.0),   // bottom center
        vec2(1.0, 1.0),   // bottom right corner
    ];

    adjecent_vecs
        .iter()
        .map(|v| {
            let pos = map.idx_to_vec2(idx).add(*v);
            TilePos {
                x: pos.x as u32,
                y: pos.y as u32,
            }
            .to_index(&map.size)
        })
        .filter(|u| u < &map.tiles.len())
        .collect()
}

pub fn adjecent_idxs(map: &Map, idx: usize) -> Vec<usize> {
    let adjecent_vecs = vec![
        vec2(0.0, -1.0), // north
        vec2(-1.0, 0.0), // west
        vec2(1.0, 0.0),  // east
        vec2(0.0, 1.0),  // south
    ];

    adjecent_vecs
        .iter()
        .map(|v| {
            let res = map.idx_to_vec2(idx).add(*v);

            if res.x < 0.0
                || res.y < 0.0
                || res.x >= map.size.x as f32
                || res.y >= map.size.y as f32
            {
                return None;
            }

            let res = TilePos {
                x: res.x as u32,
                y: res.y as u32,
            };
            Some(res.to_index(&map.size))
        })
        .filter(|t| t.is_some())
        .map(|t| t.unwrap())
        .collect()
}

pub fn get_wall_atlas_pos(
    tiles: &Vec<CoarseTileType>,
    surrounding: &Vec<usize>,
) -> CaveAtlasIndices {
    let matches = surrounding
        .iter()
        .map(|idx| tiles.get(*idx))
        .map(|t| match t {
            Some(CoarseTileType::Wall) => true,
            _ => false,
        })
        .collect::<Vec<bool>>();

    match matches[..] {
        // end pieces
        [_, false, _, true, false, _, false, _] => CaveAtlasIndices::Wall1Left,
        [_, false, _, false, true, _, false, _] => CaveAtlasIndices::Wall1Right,
        [_, false, _, false, false, _, true, _] => CaveAtlasIndices::Wall1Bottom,
        [_, true, _, false, false, _, false, _] => CaveAtlasIndices::Wall1Top,
        // connectors
        [_, true, _, true, true, _, true, _] => CaveAtlasIndices::Wall1TopRightBottomLeft,
        [_, true, _, false, false, _, true, _] => CaveAtlasIndices::Wall1TopBottom,
        [_, false, _, true, true, _, false, _] => CaveAtlasIndices::Wall1RightLeft,
        [_, false, _, true, true, _, true, _] => CaveAtlasIndices::Wall1RightBottomLeft,
        [_, true, _, true, true, _, false, _] => CaveAtlasIndices::Wall1TopRightLeft,
        [_, true, _, true, false, _, true, _] => CaveAtlasIndices::Wall1TopBottomLeft,
        [_, true, _, false, true, _, true, _] => CaveAtlasIndices::Wall1TopRightBottom,
        // corners
        [_, false, _, false, true, _, true, _] => CaveAtlasIndices::Wall1RightBottom,
        [_, false, _, true, false, _, true, _] => CaveAtlasIndices::Wall1BottomLeft,
        [_, true, _, false, true, _, false, _] => CaveAtlasIndices::Wall1TopRight,
        [_, true, _, true, false, _, false, _] => CaveAtlasIndices::Wall1TopLeft,
        _ => CaveAtlasIndices::Floor1_1,
    }
}

pub fn is_adjecent_to_room(map: &Map, idx: usize) -> bool {
    let surrounding_tiles = surrounding_tiles(map, idx);

    surrounding_tiles.iter().any(|t| match t {
        Some(t) => is_room(t),
        _ => false,
    })
}

pub fn is_neighbourless_idx(map: &Map, idx: usize) -> bool {
    !surrounding_tiles(&map, idx).iter().any(|t| match t {
        Some(t) => is_room(t),
        _ => false,
    })
}

pub fn neighbourless_idxs(map: &Map) -> Vec<usize> {
    let mut starting_points = Vec::new();
    for (idx, tile) in map.tiles.iter().enumerate() {
        if is_room(tile) {
            continue;
        }

        let is_potential_start = is_neighbourless_idx(map, idx);

        if is_potential_start {
            starting_points.push(idx);
        }
    }

    starting_points
}

pub fn generate_rooms(
    mut rng: &mut RngComponent,
    amount: usize,
    bounds: &TilemapSize,
) -> Vec<Room> {
    let mut placed_rooms: Vec<Room> = Vec::new();

    while placed_rooms.len() < amount {
        let room_size = ROOM_SIZES[rng.usize(0..ROOM_SIZES.len())];
        let mut found_empty_spot = false;
        let mut attemps = 0;
        while !found_empty_spot && attemps < ROOM_GENERATION_ATTEMPTS {
            let pos = TilePos {
                x: rng.u32(0..bounds.x),
                y: rng.u32(0..bounds.y),
            };
            if pos.x + room_size.x >= bounds.x || pos.y + room_size.y >= bounds.y {
                attemps += 1;
                continue;
            }

            let room = Room::new(pos, room_size);
            found_empty_spot = !placed_rooms.iter().any(|r| r.intersects(&room));

            if found_empty_spot {
                placed_rooms.push(room);
            }
        }
    }

    placed_rooms
}

/**
 * Takes a room and a range of x and y values. For each x and y value it checks
 *  if the tile at that the door_pos is "empty" and the tile at the other_room_pos
 * is floor. If so it stores the index of the door_pos tile. Finally it returns at
 * random one of the stored indices. If no doors were found it returns None.
 */
fn generate_doors(
    rng: &mut RngComponent,
    map: &mut Map,
    room: &Room,
    x_max: Range<u32>,
    y_max: Range<u32>,
    door_pos: Vec2,
    other_room_pos: Vec2,
) -> Option<usize> {
    let mut group = Vec::new();
    for x in x_max {
        for y in y_max.clone() {
            let door_lookup_pos = TilePos {
                x: ((room.pos.x + x) as f32 + door_pos.x) as u32,
                y: ((room.pos.y + y) as f32 + door_pos.y) as u32,
            };
            let maybe_door_tile = get_tile_at_pos(map, door_lookup_pos);
            let maybe_other_room_tile = get_tile_at_pos(
                map,
                TilePos {
                    x: ((room.pos.x + x) as f32 + other_room_pos.x) as u32,
                    y: ((room.pos.y + y) as f32 + other_room_pos.y) as u32,
                },
            );
            if maybe_door_tile.is_some() && maybe_other_room_tile.is_some() {
                let tile_space = maybe_door_tile.unwrap();
                let tile_maybe_connection = maybe_other_room_tile.unwrap();
                if !is_room(tile_space) && is_room(tile_maybe_connection) {
                    group.push(door_lookup_pos.to_index(&map.size));
                }
            }
        }
    }

    if group.len() > 0 {
        let chosen = rng.usize(0..group.len());
        map.tiles[group[chosen]] = if rng.u32(0..10) >= 5 {
            CoarseTileType::Door
        } else {
            CoarseTileType::Floor
        };
        Some(group[chosen])
    } else {
        None
    }
}

fn dfs(rng: &mut RngComponent, map: &mut Map, visited: &mut Vec<usize>, idx: usize) {
    if visited.len() > CORRIDOR_MAX_LENGTH {
        return;
    }
    let mut adjecent = adjecent_idxs(map, idx);
    if is_room(&map.tiles[idx]) || is_adjecent_to_room(map, idx) {
        return;
    } else {
        visited.push(idx);
    }

    // shuffle the adjecent tiles so we don't always go in the same direction.
    rng.shuffle(adjecent.as_mut_slice());
    // adjecent.
    for adj in adjecent.iter() {
        if adj >= &map.tiles.len() || visited.contains(adj) {
            continue;
        }

        let adjecent_to_any_visited = adjecent_idxs(map, *adj)
            .iter()
            .filter(|i| **i != idx)
            .any(|i| visited.contains(i));
        if !adjecent_to_any_visited && !is_adjecent_to_room(map, *adj) {
            dfs(rng, map, visited, *adj)
        }
    }
}

pub fn cave(
    mut rng: &mut RngComponent,
    mut commands: &mut Commands,
    tilemap_entity: &Entity,
    map_size: &TilemapSize,
    tile_storage: &TileStorage,
) {
    let mut map = Map {
        size: map_size.clone(),
        tiles: vec![CoarseTileType::Dirt; map_size.x as usize * map_size.y as usize],
    };

    // place rooms
    let amount = rng.usize(8..12);
    let rooms = generate_rooms(&mut rng, amount, map_size);
    rooms.iter().for_each(|r| {
        let w = r.size.x;
        let h = r.size.y;
        for x in 0..=w {
            for y in 0..=h {
                let coarse_type = match (x, y) {
                    _ if x == 0 || x == w || y == 0 || y == h => CoarseTileType::Floor,
                    _ => CoarseTileType::Floor,
                };

                let position = TilePos::new(r.pos.x + x, r.pos.y + y);
                let idx = position.to_index(map_size);
                commands
                    .spawn(TileBundle {
                        position,
                        tilemap_id: TilemapId(*tilemap_entity),
                        texture_index: TileTextureIndex(0),
                        ..Default::default()
                    })
                    .insert(coarse_type.clone());
                map.tiles[idx] = coarse_type;
            }
        }
    });

    // place corridors
    let starting_points = neighbourless_idxs(&map);
    let mut corridors = Vec::new();
    for start in starting_points.iter() {
        let mut visited: Vec<usize> = Vec::new();
        dfs(rng, &mut map, &mut visited, *start);
        // println!("Visited {:?}", visited);
        visited.iter().for_each(|v| {
            map.tiles[*v] = CoarseTileType::Floor;
        });
        corridors.push(visited);
    }

    let mut doors = Vec::new();
    // group possible doors by room edge and pick one for each edge of each room
    rooms.iter().for_each(|r| {
        let w = r.size.x as u32;
        let h = r.size.y as u32;

        // traverse bottom
        doors.push(generate_doors(
            rng,
            &mut map,
            r,
            0..w,
            0..1,
            vec2(0.0, -1.0),
            vec2(0.0, -2.0),
        ));
        // traverse top
        doors.push(generate_doors(
            rng,
            &mut map,
            r,
            0..w,
            h..(h + 1),
            vec2(0.0, 1.0),
            vec2(0.0, 2.0),
        ));
        // traverse left
        doors.push(generate_doors(
            rng,
            &mut map,
            r,
            0..1,
            0..h,
            vec2(-1.0, 0.0),
            vec2(-2.0, 0.0),
        ));
        // traverse right
        doors.push(generate_doors(
            rng,
            &mut map,
            r,
            w..(w + 1),
            0..h,
            vec2(1.0, 0.0),
            vec2(2.0, 0.0),
        ));
    });
    let doors = doors
        .iter()
        .filter(|d| d.is_some())
        .map(|d| d.unwrap())
        .collect::<Vec<usize>>();

    // remove dead ends and non-connected corridors
    for corridor in corridors.iter() {
        let adjacents = corridor
            .iter()
            .map(|c| adjecent_idxs(&map, *c))
            .flatten()
            .collect::<Vec<usize>>();
        if adjacents.iter().any(|a| doors.contains(&a)) {
            continue;
        }

        for c in corridor.iter() {
            let position = map.idx_to_vec2(*c);
            let position = TilePos {
                x: position.x as u32,
                y: position.y as u32,
            };
            map.tiles[*c] = CoarseTileType::Dirt;
            commands
                .spawn(TileBundle {
                    position,
                    tilemap_id: TilemapId(*tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                })
                .insert(CoarseTileType::Dirt);
        }
    }

    // add walls
    let mut walls = Vec::new();
    for (idx, tile) in map.tiles.iter().enumerate() {
        if !is_floor(tile) {
            continue;
        }

        let surrounding = surrounding_idxs(&map, idx);
        for s in surrounding.iter() {
            if &map.tiles[*s] == &CoarseTileType::Dirt {
                walls.push(*s);
            }
        }
    }

    walls.iter().for_each(|w| {
        let position = map.idx_to_vec2(*w);
        let position = TilePos {
            x: position.x as u32,
            y: position.y as u32,
        };
        map.tiles[*w] = CoarseTileType::Wall;
        commands
            .spawn(TileBundle {
                position,
                tilemap_id: TilemapId(*tilemap_entity),
                texture_index: TileTextureIndex(0),
                ..Default::default()
            })
            .insert(CoarseTileType::Dirt);
    });

    map.tiles.iter().enumerate().for_each(|(idx, t)| {
        let position = map.idx_to_vec2(idx);
        let position = TilePos {
            x: position.x as u32,
            y: position.y as u32,
        };

        let entity_id = tile_storage.get(&position);
        if let Some(entity_id) = entity_id {
            let mut entity = commands.entity(entity_id);
            match t {
                // .insert(TileTextureIndex(color));
                CoarseTileType::Wall => {
                    let surrounding = surrounding_idxs(&map, idx);
                    let texture_index = get_wall_atlas_pos(&map.tiles, &surrounding);
                    entity.insert(TileTextureIndex(texture_index as u32));
                }
                CoarseTileType::Floor => {
                    entity.insert(TileTextureIndex(CaveAtlasIndices::Floor1_1 as u32));
                }
                CoarseTileType::Door => {
                    entity.insert(TileTextureIndex(CaveAtlasIndices::Wall1Gate as u32));
                }
                CoarseTileType::Dirt => {
                    entity.insert(TileTextureIndex(CaveAtlasIndices::Floor2_1 as u32));
                }
                _ => {}
            }
        }
    });
}
