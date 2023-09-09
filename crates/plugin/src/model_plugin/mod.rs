use nitrogql_ast::{
    type_system::{TypeDefinition, TypeSystemDefinition},
    TypeSystemDocument,
};

use crate::{
    plugin_v1::{PluginCheckResult, PluginV1Beta},
    PluginCheckError,
};

/// Plugin that adds a @model directive to the schema.
#[derive(Debug)]
pub struct ModelPlugin {}

impl PluginV1Beta for ModelPlugin {
    fn name(&self) -> &str {
        "nitrogql:model-plugin"
    }
    fn schema_addition(&self) -> Option<String> {
        Some(
            r#"
directive model(
  # TypeScript type of this object. Only applicable for whole objects.
  type: String
) on OBJECT | FIELD_DEFINITION
"#
            .into(),
        )
    }
    fn check_schema(&self, schema: &TypeSystemDocument) -> PluginCheckResult {
        // Check usage of the model directive
        let mut errors = vec![];
        for def in &schema.definitions {
            let TypeSystemDefinition::TypeDefinition(def) = def else {
                continue;
            };
            match def {
                TypeDefinition::Object(def) => {
                    let model_directive = def
                        .directives
                        .iter()
                        .find(|directive| directive.name.name == "model");

                    if let Some(directive) = model_directive {
                        // Check type argument
                        let type_arg = directive
                            .arguments
                            .iter()
                            .flatten()
                            .find(|(arg, _)| arg.name == "type");

                        if type_arg.is_none() {
                            errors.push(PluginCheckError {
                                position: directive.position,
                                message: "'type' parameter is required".into(),
                                additional_info: vec![],
                            });
                        }
                    }

                    for field in def.fields.iter() {
                        let model_directive = field
                            .directives
                            .iter()
                            .find(|directive| directive.name.name == "model");

                        if let Some(directive) = model_directive {
                            // Check type argument
                            let type_arg = directive
                                .arguments
                                .iter()
                                .flatten()
                                .find(|(arg, _)| arg.name == "type");

                            if type_arg.is_some() {
                                errors.push(PluginCheckError {
                                    position: directive.position,
                                    message: "'type' parameter cannot be used on fields".into(),
                                    additional_info: vec![],
                                });
                            }
                        }
                    }
                }
                TypeDefinition::Interface(def) => {
                    for field in def.fields.iter() {
                        let model_directive = field
                            .directives
                            .iter()
                            .find(|directive| directive.name.name == "model");

                        if let Some(directive) = model_directive {
                            errors.push(PluginCheckError {
                                position: directive.position,
                                message: "model directive cannot be used on interfaces".into(),
                                additional_info: vec![],
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        PluginCheckResult { errors }
    }
}
