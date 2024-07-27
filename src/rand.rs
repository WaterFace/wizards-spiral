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

#[derive(Resource)]
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

impl rand::RngCore for GlobalRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.0.try_fill_bytes(dest)
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
