use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Options {
    pub indentation: String,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            indentation: "  ".to_string(), // Two spaces as default
        }
    }
}

impl Options {
    /// Load options from a TOML file, using defaults for missing fields
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut options: Options = toml::from_str(&content)?;

        // If indentation is empty or not set properly, use default
        if options.indentation.is_empty() {
            options.indentation = Options::default().indentation;
        }

        Ok(options)
    }

    /// Load options from a TOML file, or return default if file doesn't exist
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(options) => options,
            Err(_) => Self::default(),
        }
    }

    /// Save options to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Create a default configuration file
    pub fn create_default_config<P: AsRef<Path>>(
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let default_options = Self::default();
        default_options.save_to_file(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_options() {
        let options = Options::default();
        assert_eq!(options.indentation, "  ");
    }

    #[test]
    fn test_load_or_default_with_missing_file() {
        let options = Options::load_or_default("non_existent_file.toml");
        assert_eq!(options.indentation, "  ");
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.toml");

        let original_options = Options {
            indentation: "    ".to_string(), // 4 spaces
        };

        // Save options
        original_options.save_to_file(&file_path).unwrap();

        // Load options
        let loaded_options = Options::load_from_file(&file_path).unwrap();

        assert_eq!(loaded_options.indentation, "    ");
    }

    #[test]
    fn test_partial_toml_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("partial_config.toml");

        // Create a TOML file with missing indentation field
        fs::write(&file_path, "# Config file with no indentation setting").unwrap();

        // This should fail to parse, so load_or_default should return default
        let options = Options::load_or_default(&file_path);
        assert_eq!(options.indentation, "  ");
    }
}
