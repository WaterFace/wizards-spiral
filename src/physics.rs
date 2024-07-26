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

fn toggle_physics_debug_render(mut debug_render_context: ResMut<DebugRenderContext>) {
    let enabled = debug_render_context.enabled;
    debug_render_context.enabled = !enabled;
}
