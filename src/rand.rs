use bevy::prelude::*;
use rand::SeedableRng;

#[derive(Debug, Default)]
pub struct RandPlugin;

impl Plugin for RandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalRng>()
            .add_systems(Update, update_rng_seed);
    }
}

#[derive(Deref, DerefMut, Resource)]
pub struct GlobalRng(rand::rngs::StdRng);

impl FromWorld for GlobalRng {
    fn from_world(world: &mut World) -> Self {
        let seed = world.get_resource::<RngSeed>();

        match seed {
            None => GlobalRng(
                rand::rngs::StdRng::from_rng(rand::thread_rng())
                    .expect("Failed to initialize StdRng"),
            ),
            Some(RngSeed(seed)) => GlobalRng(rand::rngs::StdRng::seed_from_u64(*seed)),
        }
    }
}

#[derive(Debug, Resource)]
pub struct RngSeed(u64);

fn update_rng_seed(seed: Option<Res<RngSeed>>, mut commands: Commands) {
    if let Some(seed) = seed {
        if seed.is_changed() {
            commands.insert_resource(GlobalRng(rand::rngs::StdRng::seed_from_u64(seed.0)))
        }
    }
}
