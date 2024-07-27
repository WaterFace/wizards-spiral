use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Default)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
            .add_plugins(RapierDebugRenderPlugin {
                enabled: false,
                ..Default::default()
            });
        let mut rapier_config = RapierConfiguration::new(1.0);
        rapier_config.gravity = Vec2::ZERO;
        app.insert_resource(rapier_config);
    }
}

pub const COLLISION_GROUP_OBSTACLE: Group = Group::GROUP_1;
pub const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;
pub const COLLISION_GROUP_ENEMY: Group = Group::GROUP_3;
pub const COLLISION_GROUP_PROJECTILE: Group = Group::GROUP_4;
pub const COLLISION_GROUP_REFLECTED_PROJECTILE: Group = Group::GROUP_4;
