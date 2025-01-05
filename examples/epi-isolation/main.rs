pub mod parameters;

use std::path::PathBuf;

use ecs_disease_models::model::Model;
use crate::parameters::Parameters;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let parameters = match Parameters::from_file(&PathBuf::from("./examples/epi-isolation/input/input.json")) {
    Ok(parameters) => {
      parameters.validate_inputs()?;
      parameters
    }
    Err(e) => {
      eprintln!("Failed to load input: {}", e);
      return Err(e.into());
    }
  };

  let mut model = Model::with_random_seed(parameters.seed);

  // `Model`'s constructor automatically adds the `Random` and `Timeline` modules.


  model.add_module(parameters);
  // model.add_module::<PopulationStatistics>();
  // model.add_module::<TransmissionManager>();
  // model.add_module::<InfectionManager>();
  // model.add_module::<IncidenceReporter>();

  model.run()
}
