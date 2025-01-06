/*!

These are the components that make up an entity, a person.

The population loader should be kept in sync with this so that all the components are loaded for each person.

*/

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// All people have exactly one of these states.
/// These states refer to the person's infectiousness at a given time
/// and are not related to the person's health status. How long an agent
/// spends in the infectious compartment is determined entirely from their
/// number of infection attempts and draws from the generation interval.
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

// The components of our entities, people.
#[derive(Component, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Age(u8);

#[derive(Component, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct HomeId(u64);

#[derive(Component, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct CensusTract(u64);

#[derive(Component, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Alive(bool);

impl Default for Alive {
  fn default() -> Self {
    Alive(true)
  }
}
