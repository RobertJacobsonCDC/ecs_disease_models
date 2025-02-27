/*!

The _transmission manager_ is the business logic related to how new infections occur.

*/

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::SystemConfigs;
use ordered_float::OrderedFloat;
use rand::Rng;
use rand_distr::{Distribution, Exp};

use ecs_disease_models::{
  module::Module,
  random::RngResource,
  timeline::Timeline,
  timeline_event
};
use ecs_disease_models::timeline::Time;
use crate::{
  population_statistics::PopulationStatistics,
  InfectionStatus,
};


/// This free function serves as the system that is stored in the `Timeline`. It just retrieves the
/// `TransmissionManager` from the world and calls its member function.
fn attempt_infection(world: &mut World) {
  // Too noisy
  // #[cfg(feature = "print_messages")]
  // print!("Attempting infection... ");

  // We scope the mutable barrows of `world` so the compiler doesn't complain. Hence, the predeclarations.
  // Alternatively we could have used `world.resource_scope(..)`.

  let mut stats: PopulationStatistics;
  let uniform_sample: f64;
  let exponential_sample: f64;
  let next_attempt_time: OrderedFloat<f64>;
  let this: TransmissionManager;

  {
    this = world.get_resource::<TransmissionManager>().unwrap().clone();
  }

  { // scope of stats
    stats = world.get_resource::<PopulationStatistics>().unwrap().clone();
  }

  let probability_of_infection: f64 = (stats.susceptible as f64) / (stats.size() as f64);

  { // scope of rng_resource
    let mut rng_resource = world.get_resource_mut::<RngResource>().unwrap();
    // Sample uniformly from [0.0, 1.0). This is used to determine if we span an infection.
    uniform_sample =  rng_resource.rng.random::<f64>();
    // While we have the RNG in scope, we sample the exponential distribution for use below.
    exponential_sample = Exp::new(this.foi).unwrap().sample(&mut rng_resource.rng);
  }

  if uniform_sample < probability_of_infection {
    let entity = world.spawn(InfectionStatus::Infected);
    #[cfg(feature = "print_messages")]
    println!("Infection of entity {} succeeded ({:.6} < {:.6})", entity.id(), uniform_sample, probability_of_infection);
    // We use this below instead of pulling out the resource again.
    stats.update_stats(InfectionStatus::Infected);
  } else {
    // Too noisy
    // #[cfg(feature = "print_messages")]
    // println!("infection failed ({} >= {})", uniform_sample, probability_of_infection);
  }

  { // scope of timeline
    let mut timeline  = world.get_resource_mut::<Timeline>().unwrap();
    next_attempt_time = timeline.now() + exponential_sample / (stats.size() as f64);

    // Schedule the next infection attempt if there are time and susceptible people left
    if next_attempt_time <= this.max_time && stats.susceptible > 0 {
      // Too noisy
      // #[cfg(feature = "print_messages")]
      // println!("Scheduling next infection attempt at {}", next_attempt_time);

      let event = timeline_event::Event {
        time: next_attempt_time,
        command: Box::new(attempt_infection),
      };
      timeline.push(event);
    }
  }

}

#[derive(Resource, Copy, Clone, Debug)]
pub struct TransmissionManager{
  max_time: Time,
  foi: f64
}

impl TransmissionManager {
  pub fn new(max_time: Time, foi: f64) -> Self {
    Self {max_time, foi}
  }
}

impl Module for TransmissionManager {
  fn initialize_with_world(self, world: &mut World) -> Option<SystemConfigs>{
    // Insert a new instance into the world
    world.insert_resource(self);

    // Schedule the first infection attempt
    let mut timeline = world.get_resource_mut::<Timeline>().unwrap();
    timeline.push(
      timeline_event::Event {
        time: 0.0.into(),
        command: Box::new(attempt_infection)
      }
    );

    #[cfg(feature = "print_messages")]
    println!("Initialized module TransmissionManager");

    None // No systems
  }
}
