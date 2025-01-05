/*!

The `Reporter` module is responsible for writing out data according to its `ReporterConfiguration`.

Ixa uses a global `ReporterConfiguration`.

*/

use std::{
  env,
  path::PathBuf,
  fs::File,
  marker::PhantomData
};

use csv::{Writer as CsvWriter, Writer};

use bevy_ecs::prelude::Resource;
use crate::errors::IxaError;

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
  fn generate_filename(&mut self, short_name: &str) -> PathBuf {
    let basename = format!("{}{}", self.file_prefix, short_name);
    self.output_directory.join( basename).with_extension("csv")
  }
}

impl Default for ReporterConfiguration {
  fn default() -> Self {
    ReporterConfiguration {
      file_prefix: String::new(),
      output_directory: env::current_dir().unwrap(),
      overwrite: false,
    }
  }
}


/// A `Reporter` object for the type `Marker`.
#[derive(Resource)]
struct Reporter<Marker> {
  file_writer: CsvWriter<File>,

  phantom: PhantomData<Marker>
}

impl<Marker> Reporter<Marker> {
  pub fn new(short_name: String, report_configuration: &mut ReporterConfiguration) -> Result<Reporter<Marker>, IxaError> {
    let path = report_configuration.generate_filename(short_name.as_str());
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
    let file_writer = Writer::from_writer(created_file);

    Ok(
      Reporter {
        file_writer,
        phantom: PhantomData
      }
    )
  }


  fn serialize(&self, writer: &mut CsvWriter<File>) {
    writer.serialize(self).unwrap();
  }

}
