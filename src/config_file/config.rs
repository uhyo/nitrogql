use std::path::PathBuf;

/// Config file format: https://the-guild.dev/graphql/config/docs
pub struct ConfigFile {
    /// Path(s) to schema definition files.
    pub schema: Option<Vec<String>>,
    /// Path(s) to operation definition files.
    pub documents: Option<Vec<String>>,
    // extensions
    /// Output file path for schema.
    pub schema_output: Option<PathBuf>,
}
