use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;
use derive_more::From;

#[derive(Component)]
pub struct ExampleGameText;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum PhysicsSet {
    Movement,
    CollisionDetection,
}

#[derive(Debug, Component, From)]
pub struct Vel(pub Vec2);

#[derive(Debug, Component, From)]
pub struct Pos(pub Vec2);

#[derive(Debug, Component, From)]
pub struct Bounding(pub f32);

#[derive(Component, Clone)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct Player {
    pub handle: usize,
}

pub struct GgrsConfig;

impl bevy_ggrs::ggrs::Config for GgrsConfig {
    // 4-directions + fire fits easinly in a single byte
    type Input = u8;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are called `PeerId`s
    type Address = PeerId;
}
