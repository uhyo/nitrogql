mod definition_map;
mod direct_fields_of_output_type;
mod extension_resolver;

pub use definition_map::{generate_definition_map, DefinitionMap};
pub use direct_fields_of_output_type::direct_fields_of_output_type;
pub use extension_resolver::resolve_extensions;
