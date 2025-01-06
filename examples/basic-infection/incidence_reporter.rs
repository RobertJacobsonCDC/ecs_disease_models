/*!

The `IncidenceReporter` module listens for status changes and records them to a CSV file.

*/

use std::fmt::{Display, Formatter};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use ecs_disease_models::{
  timeline::Timeline,
  report::Reporter
};
use crate::InfectionStatus;

pub struct IncidenceReporterMarker;
pub type IncidenceReporter = Reporter<IncidenceReporterMarker>;

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

/// A system that monitors for infection transitions to write rows to the incidence report.
pub fn track_status_changes(
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
