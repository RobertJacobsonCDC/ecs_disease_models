/*!

We keep track of summary statistics for the population within a `Resource`. Instead of having to remember
to update this every single place the population is mutated, we attach observers to change events so that
the resource is updated automatically regardless of how the mutation happens. In our case, we only have
two situations in which we must monitor for changes:

1. When an entity is spawned. This occurs if and only if a person (not represented in code directly) transitions from susceptible to infected.
2. When an entity is changed. This occurs if and only if an infected person recovers.

*/

use std::fmt::Display;
use bevy_ecs::prelude::*;

use ecs_disease_models::{
  model::{ExecutionPhase, ModelControl},
  module::Module
};

use crate::{InfectionStatus, POPULATION};

/// Tracks summary statistics for the world.
#[derive(Resource, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PopulationStatistics {
  pub susceptible: u32,
  pub infected: u32,
  pub recovered: u32,
}

impl PopulationStatistics {

  /// Returns a total count of people in this population
  pub fn size(&self) -> u32 {
    self.infected + self.recovered + self.susceptible
  }

  /// Updates the population statistics based on the new infection status.
  ///
  /// In this model, the previous status is implicit, but this may not be the case in more sophisticated models.
  pub(crate) fn update_stats(&mut self, new_status: InfectionStatus) {
    match new_status {

      InfectionStatus::Infected => {
        self.infected += 1;
        self.susceptible -= 1;
      }

      InfectionStatus::Recovered => {
        self.infected -= 1;
        self.recovered += 1;
      }

      InfectionStatus::Susceptible => {
        // In this model this is not a transition and shouldn't happen. We panic.
        unreachable!("infection status transitioned to `InfectionStatus::Susceptible`, which is not possible.");
      }

    }
  }
}

impl Display for PopulationStatistics {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{{ susceptible: {}, infected: {}, recovered: {} }}", self.susceptible, self.infected, self.recovered)
  }
}

// The following is unneeded, because for Bevy ECS a newly spawned entity is counted as a change.
/*
/// A system that handles the case when a person transitions from `Susceptible` to `Infected`, which occurs
/// if and only if an entity is spawned.
fn handle_spawned_infected(
  mut population_stats: ResMut<PopulationStatistics>,
  query: Query<(&InfectionStatus, Entity), Added<InfectionStatus>>,
) {
  // New entities should only ever be spawned with the `InfectionStation::Infected` status in this model.
  // It is a good practice to actually check this invariant and emit an error if it is violated.
  for (new_status, _) in query.iter() {
    if *new_status == InfectionStatus::Infected{
      population_stats.update_stats(InfectionStatus::Infected);
      #[cfg(feature = "print_messages")]
      println!("Spawn change detected (Infected). Updating PopulationStatistics: {:?}", population_stats);
    }
  }
}
*/

/// A system that monitors for infection transitions to adjust the stats correctly.
fn track_population_changes(
  mut population_stats: ResMut<PopulationStatistics>,
  mut model_control: ResMut<ModelControl>,
  query: Query<(&InfectionStatus, &InfectionStatus), Changed<InfectionStatus>>,
) {
  // Track the changes in infection status.
  for (new_status, _) in query.iter() {
    // In our model, the only change of status is a transition from infected to recovered. However,
    // Bevy ECS counts spawning an `Entity` as a "change". Oops.
    population_stats.update_stats(*new_status);

    match new_status {

      InfectionStatus::Susceptible => {
        /* This case should never happen in this model. */
      }

      InfectionStatus::Infected => {
        #[cfg(feature = "print_messages")]
        println!("Change to Infected detected. Updated PopulationStatistics: {}", population_stats.as_ref());
      }

      InfectionStatus::Recovered => {
        #[cfg(feature = "print_messages")]
        println!("Change to Recovered detected. Updated PopulationStatistics: {}", population_stats.as_ref());
      }

    }

  }

  // This is a reasonable place to detect if the simulation should stop.
  if population_stats.recovered == population_stats.size() {
    #[cfg(feature = "print_messages")]
    println!("Requesting ModelControl::Finished");
    *model_control = ModelControl::Finished;
  }

}

impl Module for PopulationStatistics {
  fn initialize_with_world(world: &mut World, schedule: &mut Schedule) {
    let stats = PopulationStatistics {
      susceptible: POPULATION,
      infected: 0,
      recovered: 0,
    };

    world.insert_resource(stats);

    // Also set up change monitors that keep these statistics up to date.
    schedule.add_systems(
        // handle_spawned_infected.in_set(ExecutionPhase::Normal),
        track_population_changes.in_set(ExecutionPhase::Normal)
    );

    #[cfg(feature = "print_messages")]
    println!("Initialized module PopulationStatistics");
  }
}
