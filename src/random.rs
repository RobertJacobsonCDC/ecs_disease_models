use bevy_ecs::prelude::*;

use rand::{
  rngs::SmallRng,
  SeedableRng
};

use crate::module::Module;

#[derive(Resource)]
pub struct RngResource {
  pub rng: SmallRng
}

impl Default for RngResource {
  fn default() -> Self {
    Self::new()
  }
}

impl RngResource {
  pub fn new() -> Self {
    Self::with_random_seed(42)
  }

  pub fn with_random_seed(seed: u64) -> Self {
    RngResource {
      rng: SmallRng::seed_from_u64(seed)
    }
  }
}

// ToDo: Do something better with the initial seed. There's a half-hearted attempt littered throughout this demo.
impl Module for RngResource {
  fn initialize_with_world(world: &mut World, _schedule: &mut Schedule) {
    world.insert_resource(RngResource::with_random_seed(42));
    #[cfg(feature = "print_messages")]
    println!("Initialized module Random");
  }
}
