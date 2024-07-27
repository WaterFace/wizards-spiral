use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct CyclePlugin;

impl Plugin for CyclePlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Debug, Default, Resource)]
pub struct CycleCounter {
    pub count: u64,
}
