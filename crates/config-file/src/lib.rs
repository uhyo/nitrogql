mod config;
mod error;
#[cfg(feature = "fs")]
mod load_config;
mod parse_config;
#[cfg(test)]
mod tests;

pub use config::{Config, GenerateConfig, GenerateMode};
#[cfg(feature = "fs")]
pub use load_config::load_config;
pub use parse_config::parse_config;
