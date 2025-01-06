mod parameters;
mod periodic_reporter;
mod population_loader;
mod person;

use std::{
  fmt::Display,
  path::PathBuf
};
use bevy_ecs::{
  component::Component,
  prelude::IntoSystemConfigs
};
use serde::{Deserialize, Serialize};

use ecs_disease_models::{
  model::{ExecutionPhase, Model},
  report::ReporterConfiguration
};

use crate::{
  parameters::Parameters,
  periodic_reporter::PeriodicReporter
};

const PARAMETERS_PATH: &str = "./examples/epi-isolation/input/input.json";
const OUTPUT_DIRECTORY: &str = "./examples/epi-isolation/output";
const OUTPUT_FILE_PREFIX: &str = "epi-isolation";
const OUTPUT_FILE_NAME: &str = "incidence";


fn main() -> Result<(), Box<dyn std::error::Error>> {
  let parameters = Parameters::from_file(&PathBuf::from(PARAMETERS_PATH))?;

  // `Model`'s constructor automatically adds the `Random` and `Timeline` modules.
  let mut model = Model::with_random_seed(parameters.seed);

  model.add_module(parameters);

  // A more thought-through API would make this less awkward.
  let report_config = ReporterConfiguration::new(
    OUTPUT_FILE_PREFIX.to_string(),
    PathBuf::from(OUTPUT_DIRECTORY),
    true
  );
  model.add_module(report_config);

  model.add_module(PeriodicReporter::new("incidence".to_string()));
  // ToDo: Having to add this separately is an awkward pattern.
  model.add_systems(periodic_reporter::track_status_changes.in_set(ExecutionPhase::Normal));

  model.run();

  Ok(())
}
