/*!

A `Model` manages the execution loop and abstracts over Bevy ECS specific implementation details.

This is not the best design. It's just a demo. For example, modules have unfettered access to the entire schedule.
I think a little more is needed to enable parallelism, also.

Names are hard. `Context` is used in Ixa to mean wht Bevy ECS calls `World`, and of course `World` is taken. `Model`
plays the role of `App` in full Bevy.

*/

use bevy_ecs::prelude::*;

use crate::{
  random::RngResource,
  module::Module,
  timeline::Timeline
};
// ToDo: `Model` should use the builder pattern.

/// A `Model` has three execution phases that it runs in order within the event loop.
#[derive(SystemSet, PartialEq, Eq, Ord, Clone, Copy, PartialOrd, Debug, Hash)]
pub enum ExecutionPhase {
  First,
  Normal,
  Last,
}

pub struct Model {
  schedule: Schedule,
  world: World
}

/// The `ModelControl` resource is how modules communicate to the `Model` to effect the event loop.
#[derive(Resource, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Debug, Hash)]
pub enum ModelControl {
  #[default]
  Running, // The simulation can run as normal
  Paused,  // Debug mode, perhaps
  Aborted, // Aborted do due error condition or user request
  Finished // The simulation has run to completion
}

fn system_for_first_phase() {
  // println!("Running system in First phase");
}

fn system_for_normal_phase() {
  // println!("Running system in Normal phase");
}

fn system_for_last_phase() {
  // println!("Running system in Last phase");
}

impl Default for Model {
  fn default() -> Self {
    Self::new()
  }
}

impl Model {
  pub fn new() -> Self {
    Self::with_random_seed(42)
  }

  pub fn with_random_seed(seed: u64) -> Self {
    let mut model = Model {
      schedule: Schedule::default(),
      world: World::default(),
    };

    // Insert the system control resource
    model.world.insert_resource(ModelControl::default());

    // Add the phase schedules to the parent schedule with labels
    model.schedule.add_systems(
      (
        system_for_first_phase.in_set(ExecutionPhase::First),
        system_for_normal_phase.in_set(ExecutionPhase::Normal),
        system_for_last_phase.in_set(ExecutionPhase::Last)
      )
    );

    // Enforce execution order of `SystemSets`
    model.schedule.configure_sets(
      (
        ExecutionPhase::First.before(ExecutionPhase::Normal),
        ExecutionPhase::Normal.before(ExecutionPhase::Last),
      )
    );

    // Every `World` gets these modules
    model.add_module(Timeline::default());
    model.add_module(RngResource::with_random_seed(seed));

    model
  }

  /// Adds the module `M` to this model. Notice that `M` is a generic parameter. The model will call the static
  /// constructor of `M` to create a new instance of the model.
  pub fn add_module<M: Module>(&mut self, module: M) {
    if let Some(systems) = module.initialize_with_world(&mut self.world) {
      self.schedule.add_systems(systems);
    }
  }


  /// Runs the simulation
  pub fn run(&mut self) {
    // limit loops for debug purposes
    loop {

      self.schedule.run(&mut self.world);

      // We act on `ModelControl` requests
      match self.world.get_resource::<ModelControl>().unwrap() {
        ModelControl::Paused
        | ModelControl::Aborted
        | ModelControl::Finished => {
          // For this demo these all do the same thing.
          #[cfg(feature = "print_messages")]
          println!("Stopping model");
          break;
        }

        ModelControl::Running => { /* pass */ }
      }

    }
  }
}
