/*!

The `IncidenceReporter` module listens for status changes and records them to a CSV file.

*/

use std::{
  fs::File,
  path::PathBuf,
  fmt::{Display, Formatter}
};
use bevy_ecs::{
  prelude::*,
  schedule::SystemConfigs
};
use serde::{Deserialize, Serialize};
use csv::Writer;

use ecs_disease_models::{
  module::Module,
  model::ExecutionPhase,
  timeline::Timeline
};
use crate::InfectionStatus;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub(crate) struct IncidenceReportItem {
  time: f64,
  person_id: u32,
  infection_status: InfectionStatus,
}

impl Display for IncidenceReportItem {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{{time: {}, person_id: {}, infection_status: {}}}", self.time, self.person_id, self.infection_status)
  }
}

#[derive(Resource)]
pub struct IncidenceReporter {
  file_name: PathBuf,
  writer: Option<Writer<File>>,
}

impl IncidenceReporter {
  // Create a new IncidenceReporter with the given file name
  pub fn new(file_name: &str) -> Self {
    let mut new_reporter = IncidenceReporter {
      file_name: PathBuf::from(file_name),
      writer: None,
    };
    new_reporter.init_writer().expect("Failed to init file writer");
    // new_reporter.write_headers().expect("Failed to write headers");

    new_reporter
  }

  // Initialize the writer (creating or opening the CSV file)
  pub fn init_writer(&mut self) -> std::io::Result<()> {
    let file = File::create(&self.file_name)?;
    let writer = Writer::from_writer(file);
    self.writer = Some(writer);
    Ok(())
  }

  // Write the headers to the CSV (based on the fields of IncidenceReportItem)
  // pub fn write_headers(&mut self) -> std::io::Result<()> {
  //     if let Some(ref mut writer) = self.writer {
  //         writer.write_record(&["name", "incidence_rate", "date"])?;
  //     }
  //     Ok(())
  // }

  // Write a row of data from an IncidenceReportItem instance to the CSV
  pub fn write_row(&mut self, item: IncidenceReportItem) -> std::io::Result<()> {
    if let Some(ref mut writer) = self.writer {
      writer.serialize(item)?;
    }
    Ok(())
  }

  // Close the writer and finalize the CSV file
  pub fn finish(&mut self) -> std::io::Result<()> {
    if let Some(ref mut writer) = self.writer {
      writer.flush()?;
    }
    Ok(())
  }
}

impl Drop for IncidenceReporter {
  fn drop(&mut self) {
    self.finish().expect("Failed to finish");
  }
}


impl Module for IncidenceReporter {
  fn initialize_with_world(self, world: &mut World) -> Option<SystemConfigs> {
    #[cfg(feature = "print_messages")]
    println!("Initialized module PopulationStatistics");

    world.insert_resource(self);

    // Also set up change monitors that keep these statistics up to date.
    Some(track_status_changes.in_set(ExecutionPhase::Normal))
  }
}


/// A system that monitors for infection transitions to write rows to the incidence report.
fn track_status_changes(
  mut incidence_reporter: ResMut<IncidenceReporter>,
  timeline: Res<Timeline>,
  query: Query<(Entity, &InfectionStatus), Changed<InfectionStatus>>,
) {
  // Track the changes in infection status.
  for (entity, new_status) in query.iter() {
    let report_item = IncidenceReportItem{
      time: timeline.now().0,
      person_id: entity.index(),
      infection_status: new_status.clone(),
    };

    #[cfg(feature = "print_messages")]
    println!("Writing change to report {}", report_item);
    incidence_reporter.write_row(report_item).expect("Failed to write row.");

  }

}
