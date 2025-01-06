/*!

A PopulationLoader loads population information from an input CSV file and adds them as entities to the world. If
a path to an input file has not been provided, the `PopulationLoader` will look for it in the world's global
`Parameters` instance.

This module is a little different from the others in that it adds no resources or systems, only entities.

*/

use std::{
  fs::File,
  path::PathBuf
};
use serde::Deserialize;
use csv::ReaderBuilder;

use bevy_ecs::{
  prelude::*,
  schedule::SystemConfigs
};
use ecs_disease_models::{
  errors::IxaError,
  module::Module
};
use crate::{
  parameters::Parameters,
  person::{Age, CensusTract, HomeId}
};
use crate::person::{Alive, InfectionStatus};

/// A person record as read from the input file. This is immediately parsed into components to become an entity.
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct PeopleRecord<'a> {
  age: u8,
  homeId: &'a [u8],
}

pub struct PopulationLoader{
  input_file: Option<PathBuf>,
}

impl PopulationLoader {
  pub fn new() -> Self {
    PopulationLoader{
      input_file: None,
    }
  }

  /// Parse a person record and insert it into the world.
  fn create_person_entity_from_record(
    world: &mut World,
    person_record: &PeopleRecord,
  ) -> Result<(), IxaError> {
    let tract: String = String::from_utf8(person_record.homeId[..11].to_owned())?;
    let home_id: String = String::from_utf8(person_record.homeId.to_owned())?;

    world.spawn((
      Age(person_record.age),
      HomeId(home_id.parse()?),
      CensusTract(tract.parse()?),
      Alive::default(),
      InfectionStatus::default()
    ));

    Ok(())
  }

  /// Loads the population data from the CSV file into the world.
  fn load_population_data(&self, world: &mut World) -> Result<(), IxaError> {
    let file = // Open a file using either self or the path in the global parameters
        {
          if let Some(input_file_path) = &self.input_file {
            File::open(input_file_path)?
          } else {
            let parameters = world
                .get_resource::<Parameters>()
                .ok_or(IxaError::IxaError("no input file provided or global Parameters object".to_string()))?;
            File::open(&parameters.synth_population_file)?
          }
        };


    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)  // CSV has headers
        .from_reader(file);

    // Deserialize each record into a PeopleRecord
    for result in csv_reader.deserialize() {
      let record: PeopleRecord = result?;
      // Insert into world
      Self::create_person_entity_from_record(world, &record)?;
    }

    Ok(())
  }

}

impl Module for PopulationLoader {
  fn initialize_with_world(self, world: &mut World) -> Option<SystemConfigs> {
    // ToDo: Again, there should be a coherent error handling strategy.
    self.load_population_data(world).expect("failed to load Population");
    None // No systems
  }
}
