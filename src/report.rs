/*!

The `Reporter` module is responsible for writing out data according to the `ReporterConfiguration`. If no
`ReporterConfiguration` exists, a default `ReporterConfiguration` will be created.

Ixa uses a global `ReporterConfiguration`.

The `Reporter<Marker>` takes a `Marker` type so that multiple `Reporter`'s can exist at once in a single `World`
instance. Instead of `Reporter<Marker>` singletons, we could just have instances of reporter systems with `Local<D>`
data. This local data is not stored in the world. See https://bevy-cheatbook.github.io/programming/local.html


ToDo: This API needs some work. Some questions are recorded in To-Do's below. Questions:
        - Where is the system that triggers a write added to the schedule?
        - Whose responsibility is it to add the `ReporterConfiguration`? What if there is none?
        - Right now it is only possible to produce a CSV file with rows from a single struct.
          This seems overly restrictive.

*/

use std::{
  env,
  path::PathBuf,
  fs::File,
  marker::PhantomData
};
use csv::Writer as CsvWriter;
use serde::Serialize;

use bevy_ecs::{
  prelude::{Resource, World},
  schedule::SystemConfigs
};
use crate::{
  errors::IxaError,
  module::Module
};

#[derive(Resource)]
pub struct ReporterConfiguration {
  /// Precedes the report name in the filename. An example of a potential prefix might be scenario or simulation name.
  /// Defaults to empty string.
  pub file_prefix: String,
  /// Location that the CSVs are written to. An example of this might be "/data/". Defaults to current active directory.
  pub output_directory: PathBuf,
  /// If `true`, will overwrite existing files in the same location. Default is `false`.
  pub overwrite: bool,
}

impl ReporterConfiguration {
  /// Creates a `ReporterConfiguration` with the given parameters.
  #[must_use]
  pub fn new(file_prefix: String, output_directory: PathBuf, overwrite: bool) -> Self {
    ReporterConfiguration {
      file_prefix,
      output_directory,
      overwrite
    }
  }

  /// Builds the filename. Called by `add_report`, `short_name` refers to the
  /// report type. The three main components are `prefix`, `directory`, and
  /// `short_name`.
  fn generate_filename(&self, short_name: &str) -> PathBuf {
    let basename = format!("{}{}", self.file_prefix, short_name);
    self.output_directory.join( basename).with_extension("csv")
  }
}

impl Default for ReporterConfiguration {
  fn default() -> Self {
    ReporterConfiguration {
      file_prefix: String::new(),
      output_directory: env::current_dir().expect("Failed to get current directory"),
      overwrite: false,
    }
  }
}

impl Module for ReporterConfiguration {
  fn initialize_with_world(self, world: &mut World) -> Option<SystemConfigs> {
    world.insert_resource(self);
    None
  }
}

#[derive(Resource)]
pub struct Reporter<Marker: Send + Sync + 'static> {
  short_name: String,
  writer: Option<CsvWriter<File>>,
  marker: PhantomData<Marker>
}

impl<Marker: Send + Sync + 'static> Reporter<Marker> {

  /// Creates a `Reporter` with the provided short name according to the provided `ReporterConfiguration`.
  pub fn new(short_name: String) -> Reporter<Marker> {
    Reporter{
      short_name,
      writer: None,
      marker: PhantomData
    }
  }

  /// This is called from `initialize_with_world`, so it could be private.
  pub fn initialize(&mut self, report_configuration: &ReporterConfiguration) -> Result<(), IxaError> {
    let path = report_configuration.generate_filename(self.short_name.as_str());
    let created_file = match File::create_new(&path) {

      Ok(file) => file,

      Err(e) => {
        match e.kind() {
          std::io::ErrorKind::AlreadyExists => {
            if report_configuration.overwrite {
              File::create(&path)?
            } else {
              println!("File already exists: {}. Please set `overwrite` to true in the file configuration and rerun.", path.display());
              return Err(IxaError::IoError(e));
            }
          }
          _ => {
            return Err(IxaError::IoError(e));
          }
        }
      }

    };

    self.writer = Some(CsvWriter::from_writer(created_file));

    Ok(())
  }

  // ToDo: Have this return an `IxaError`
  /// Write a row of data from an IncidenceReportItem instance to the CSV
  pub fn write_row<ReportItem>(&mut self, item: ReportItem) -> csv::Result<()>
      where ReportItem: Serialize + Send + Sync + Sized
  {
    // ToDo: What if self isn't initialized?
    let writer = self.writer.as_mut().expect("Failed to get writer");
    writer.serialize(item)
  }

}

impl<Marker: Send + Sync + 'static> Drop for Reporter<Marker> {
  fn drop(&mut self) {
    if let Some(mut writer) = self.writer.take() {
      // ToDo: Determine error handling needs here. Right now we ignore it.
      let _ = writer.flush();
    }
  }
}


impl<Marker: Send + Sync + 'static> Module for Reporter<Marker> {
  /// Inserts self into world. The caller needs to schedule the system.
  fn initialize_with_world(mut self, world: &mut World) -> Option<SystemConfigs> {
    #[cfg(feature = "print_messages")]
    println!("Initialized module Reporter");

    let config = // get or insert ReporterConfiguration
        match world.get_resource::<ReporterConfiguration>() {
          Some(config) => config,
          None => {
            world.insert_resource(ReporterConfiguration::default());
            world.get_resource::<ReporterConfiguration>().unwrap()
          }
        };

    self.initialize(config).expect("Failed to initialize Reporter");
    world.insert_resource(self);

    None // No systems?
  }
}
