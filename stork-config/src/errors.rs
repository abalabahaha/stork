use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ConfigFromFileError {
    #[error("Recieved empty configuration string")]
    EmptyString,

    #[error("File `{0}` not found, or unreadable")]
    UnreadableFile(PathBuf),

    #[error("Cannot parse config. Stork recieved error: `{0}`")]
    UnparseableInput(#[from] toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_toml_error() {
        let expected = "Cannot parse config. Stork recieved error: `expected an equals, found an identifier at line 1 column 6`";
        let computed = toml::from_str::<()>("this is bad toml")
            .map_err(ConfigFromFileError::from)
            .unwrap_err()
            .to_string();
        assert_eq!(expected, computed);
    }
}
