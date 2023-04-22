use bevy::{
    math::vec2,
    prelude::{Commands, Component, Entity, Local, ResMut, Vec2},
};
use bevy_ecs_tilemap::{
    prelude::{TilemapId, TilemapSize},
    tiles::{TileBundle, TilePos, TileTextureIndex},
    TilemapBundle,
};
use bevy_turborand::{DelegatedRng, GlobalRng, RngComponent};

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
enum CoarseTileType {
    Wall,
    Floor,
    Door,
}
enum CaveAtlasIndices {
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

pub fn cave(
    mut rng: &mut RngComponent,
    mut commands: &mut Commands,
    tilemap_entity: &Entity,
    map_size: &TilemapSize,
) {
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
                let tile_entity = commands
                    .spawn(TileBundle {
                        position,
                        tilemap_id: TilemapId(*tilemap_entity),
                        texture_index: TileTextureIndex(0),
                        ..Default::default()
                    })
                    .insert(coarse_type);
            }
        }
    })
}
