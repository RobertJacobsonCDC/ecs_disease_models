/*!

Contrary to the version in Ixa, this example doesn't use any special general infrastructure (outside of Bevy ECS primitives.

*/

use std::{
  io::Read,
  fs::File,
  path::PathBuf
};
use serde::{Deserialize, Serialize};

use bevy_ecs::{
  prelude::*,
  schedule::SystemConfigs
};

use ecs_disease_models::{
  module::Module,
  errors::IxaError
};

#[derive(Resource, Serialize, Deserialize, Debug, Clone)]
pub struct Parameters{
  pub max_time: f64,
  pub seed: u64,
  pub r_0: f64,
  pub infection_duration: f64,
  pub generation_interval: f64,
  pub report_period: f64,
  pub synth_population_file: PathBuf,
}

impl Parameters {
  pub fn from_file(path: &PathBuf) -> Result<Parameters, IxaError>{
    let mut file = File::open(path)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;

    let json_data: serde_json::Value = serde_json::from_str(&file_contents)?;
    let parameters: Parameters = serde_json::from_value(json_data["epi_isolation.Parameters"].clone())?;

    parameters.validate_inputs()?;

    Ok(parameters)
  }

  pub fn validate_inputs(&self) -> Result<(), IxaError> {
    if self.r_0 < 0.0 {
      return Err(IxaError::IxaError(
        "r_0 must be a non-negative number.".to_string(),
      ));
    }
    if self.generation_interval <= 0.0 {
      return Err(IxaError::IxaError(
        "The generation interval must be positive.".to_string(),
      ));
    }
    Ok(())
  }

}

impl Module for Parameters {
  fn initialize_with_world(mut self, world: &mut World) -> Option<SystemConfigs>{
    // Insert a new instance into the world
    world.insert_resource(self);

    #[cfg(feature = "print_messages")]
    println!("Initialized module Parameters");

    // `Parameters` schedules no systems
    None
  }
}
