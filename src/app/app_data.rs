use std::path::PathBuf;

pub struct AppData {
  input_file: Option<PathBuf>,
  output_file: Option<PathBuf>,
  text_input: String,
}
