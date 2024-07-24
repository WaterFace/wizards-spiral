use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct RandPlugin;

impl Plugin for RandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalRng>()
            .add_systems(Update, update_rng_seed);
    }
}

#[derive(Deref, DerefMut, Resource)]
pub struct GlobalRng(fastrand::Rng);

impl FromWorld for GlobalRng {
    fn from_world(world: &mut World) -> Self {
        let seed = world.get_resource::<RngSeed>();

        match seed {
            None => GlobalRng(fastrand::Rng::new()),
            Some(RngSeed(seed)) => GlobalRng(fastrand::Rng::with_seed(*seed)),
        }
    }
}

#[derive(Debug, Resource)]
pub struct RngSeed(u64);

fn update_rng_seed(seed: Option<Res<RngSeed>>, mut commands: Commands) {
    if let Some(seed) = seed {
        if seed.is_changed() {
            commands.insert_resource(GlobalRng(fastrand::Rng::with_seed(seed.0)))
        }
    }
}
