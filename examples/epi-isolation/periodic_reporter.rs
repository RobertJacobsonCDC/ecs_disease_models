/*!

The `PeriodicReporter` module collects statistics at regular intervals and records them to a CSV file.

The original epi-isolation code created a weird report in which for every sampled time it lists every combination of
`(Age, CensusTract, InfectiousStatus)` values and then _counts_ how many entities there are with that combination.
We don't do this. Instead we just print the time and `Age, CensusTract, InfectiousStatus` for every entity. Even this
information is odd when the number of entities is small (less than a few thousand). It would be much more efficient to
just record the time of each status change. But presumably periodic reports like this are for large populations.

ToDo: Periodic reporting should be generic and built-in, unified with `Reporter<Marker>`.

*/

use std::{
  fmt::{Display, Formatter}
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use ecs_disease_models::{
  timeline::Timeline,
  report::Reporter,
  timeline::Time
};
use crate::person::{Age, CensusTract, InfectionStatus};


pub struct PeriodicReporterMarker;
pub type PeriodicReporter = Reporter<PeriodicReporterMarker>;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub(crate) struct IncidenceReportItem {
  time: Time,
  age: Age,
  census_tract: CensusTract,
  infection_status: InfectionStatus,
}

impl Display for IncidenceReportItem {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{{ Time({:.6}), {:?}, {:?}, {} }}",
      self.time,
      self.age,
      self.census_tract,
      self.infection_status
    )
  }
}

/// The command that writes out a row of the periodic report.
pub fn write_periodic_report(
  mut periodic_reporter: ResMut<PeriodicReporter>,
  timeline: Res<Timeline>,
  query: Query<(Entity, &Age, &CensusTract, &InfectionStatus)>,
) {
  let time = timeline.now();

  // (Age, CensusTract, InfectiousStatus, Count)
  // Track the changes in infection status.
  for (_, age, census_tract, infection_status) in query.iter() {
    let report_item = IncidenceReportItem{
      time,
      age: *age,
      census_tract: *census_tract,
      infection_status: *infection_status,
    };

    #[cfg(feature = "print_messages")]
    println!("Writing change to report {}", report_item);
    periodic_reporter.write_row(report_item).expect("Failed to write row.");
  }

}
