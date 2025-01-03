/*!

The _infection manager_ is the business logic related to how existing infections evolve.

*/

use bevy_ecs::prelude::*;
use rand::distr::Distribution;
use rand_distr::Exp;

use ecs_disease_models::{
  model::ExecutionPhase,
  module::Module,
  random::RngResource,
  timeline::Timeline,
  timeline_event::Event,
};
use crate::{
  InfectionStatus,
  INFECTION_DURATION,
  transmission_manager::TransmissionManager
};


/// A system that handles the case when a person transitions from `Susceptible` to `Infected`, which occurs
/// if and only if an entity is spawned.
fn schedule_recovery(
  mut timeline: ResMut<Timeline>,
  mut rng: ResMut<RngResource>,
  query: Query<(&InfectionStatus, Entity), Added<InfectionStatus>>,
) {
  // New entities should only ever be spawned with the `InfectionStation::Infected` status in this model.
  // It is a good practice to actually check this invariant and emit an error if it is violated.
  for (new_status, entity) in query.iter() {
    if *new_status == InfectionStatus::Infected{
      // When a new infection occurs, we schedule the person's recovery on the `Timeline`.
      let time = timeline.now() + Exp::new(1.0 / INFECTION_DURATION).unwrap().sample(&mut rng.rng);

      timeline.push(
        Event{
          time,
          command: Box::new(move | world | {
            let mut status = world.get_mut::<InfectionStatus>(entity).expect("An entity was removed before it was recovered.");
            *status = InfectionStatus::Recovered;
            #[cfg(feature = "print_messages")]
            println!("Entity {} recovered at time {:.4}", entity, time);
          }),
        }
      );


      #[cfg(feature = "print_messages")]
      println!("Spawn change detected. Scheduling recovery at {:.4} for Entity {}", time, entity);
    }
  }
}

// Holds no state
#[derive(Resource)]
pub struct InfectionManager;

impl Module for InfectionManager {
  fn initialize_with_world(world: &mut World, schedule: &mut Schedule) {
    // Insert a new instance into the world
    world.insert_resource(TransmissionManager);

    // Schedule the listener for new infections
    schedule.add_systems(schedule_recovery.in_set(ExecutionPhase::Normal));

    #[cfg(feature = "print_messages")]
    println!("Initialized module InfectionManager");
  }
}
