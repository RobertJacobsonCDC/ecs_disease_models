/*!

A re-implementation of the basic-infection example using Bevy ECS.

# Minor difference from original

Random selection as implemented in the original `basic-infection` example does not really
map well to Bevy ECS, because the ECS intentionally obscures the datastore as an indexable
array of entities. It's bad practice to attempt to index into the list of entities, because
entities are garbage collected, and an entity's location may be reused after the entity is
"despawned". Using an index to refer to a particular entity is therefore fragile at best.
Moreover, whatever is doing the random sampling should not also bear the responsibility of
keeping track of the lifecycle of each person, at least not in the philosophy of ECS.
(Still, it _is_ possible to do so using `Entities::resolve_from_id(..)`.)

If we want, we can just keep track of the entities (their handles) in a vector. The
downsides to this approach are:
 1. You have to track the lifecycle of each entity yourself.
 2. You have the additional memory burden.

Neither of these is a big deal, but they are both things the ECS is already doing for you.

This issue boils down to having to work with the data "online", for example via an iterator,
instead of via random access through indexing. There are usually efficient-enough ways
of working around this issue, like using Reservoir Sampling to select a random sample.

We take a different approach, instead considering the probability that a susceptible person
will be added for an infection "attempt." In the original example, a person was selected
uniformly from the population, and only if the chosen person was susceptible would they
then be infected. Thus, the probability that a newly infected person is added at a single
infection attempt is

    #{people who are susceptible} / #{population size}.

We will keep track of the numerator in a summary statistics resource. The denominator is
the constant `POPULATION`.

Strictly speaking, we don't actually have to store the entities at all for an example this
simple. All we need to do is store the count of people within each `InfectionStatus`
category. But more sophisticated models will certainly need to store individual people as
entities, so we do so here for the purpose of illustration.

*/

pub mod transmission_manager;
pub mod population_statistics;
mod infection_manager;
mod incidence_reporter;

use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use bevy_ecs::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use ecs_disease_models::{
  model::Model,
  timeline::Time
};
use ecs_disease_models::model::ExecutionPhase;
use ecs_disease_models::report::ReporterConfiguration;
use crate::{
  population_statistics::PopulationStatistics,
  infection_manager::InfectionManager,
  transmission_manager::TransmissionManager,
  incidence_reporter::IncidenceReporter
};

static POPULATION        : u32  = 1000;
static SEED              : u64  = 123;
static MAX_TIME          : Time = OrderedFloat(303.0);
static FOI               : f64  = 0.1;
static INFECTION_DURATION: f64  = 5.0;
static OUTPUT_DIR        : &'static str = "./examples/basic-infection";

/**
All people have exactly one of these states. In fact, because this is the only property
of a person within this model, an entity in our ECS _is_ an `InfectionStatus`––though
we don't bother creating an entity until a person's `InfectionStatus` changes to
`InfectionStatus::Infected`.
*/
#[derive(Component, Clone, Copy, PartialEq, Eq, Default, Debug, Hash, Serialize, Deserialize)]
pub enum InfectionStatus {
  #[default]
  Susceptible,
  Infected,
  Recovered,
}

impl Display for InfectionStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}


fn main() {
  let mut model = Model::with_random_seed(SEED);
  // `Model`'s constructor automatically adds the `Random` and `Timeline` modules.
  model.add_module(PopulationStatistics::with_size(POPULATION));
  model.add_module(TransmissionManager::new(MAX_TIME, FOI));
  model.add_module(InfectionManager::new(INFECTION_DURATION));

  // A more thought-through API would make this less awkward.
  let report_config = ReporterConfiguration::new(
    "basic_infection_".to_string(),
    PathBuf::from(OUTPUT_DIR),
    true
  );
  model.add_module(report_config);

  model.add_module(IncidenceReporter::new("incidence".to_string()));
  // ToDo: Having to add this separately is an awkward pattern.
  model.add_systems(incidence_reporter::track_status_changes.in_set(ExecutionPhase::Normal));


  model.run()
}
