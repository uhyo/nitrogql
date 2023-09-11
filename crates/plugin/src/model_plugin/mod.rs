use std::collections::HashMap;

use nitrogql_ast::{
    type_system::{TypeDefinition, TypeSystemDefinition},
    value::Value,
    TypeSystemDocument,
};
use nitrogql_printer::ts_types::{ts_types_util::ts_union, TSType};

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

                        let type_arg_is_specified = match type_arg {
                            Some((_, value)) => !value.is_null(),
                            None => false,
                        };

                        if !type_arg_is_specified {
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
    fn transform_resolver_output_types<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
        mut base: HashMap<&'src str, TSType>,
    ) -> HashMap<&'src str, TSType> {
        for def in document.definitions.iter() {
            if let TypeSystemDefinition::TypeDefinition(TypeDefinition::Object(def)) = def {
                let model_directive = def
                    .directives
                    .iter()
                    .find(|directive| directive.name.name == "model");
                if let Some(d) = model_directive {
                    let type_arg = d
                        .arguments
                        .iter()
                        .flatten()
                        .find(|(arg, _)| arg.name == "type");
                    let Some((_, value)) = type_arg else {
                        panic!("'type' argument is required");
                    };
                    let Value::StringValue(value) = value else {
                        continue;
                    };
                    // if @model(type: "...") is applied to a whole object,
                    // then we need to replace the type of the object with the specified type
                    *base.get_mut(&def.name.name).expect("object not found") =
                        TSType::Raw(value.value.clone());
                    continue;
                }

                let model_field_names = def.fields.iter().filter_map(|field| {
                    field
                        .directives
                        .iter()
                        .any(|directive| directive.name.name == "model")
                        .then_some(field.name.name)
                });
                let base_type = base.remove(&def.name.name).expect("object not found");
                let obj_type = TSType::TypeFunc(
                    Box::new(TSType::TypeVariable("Pick".into())),
                    vec![
                        base_type,
                        ts_union(model_field_names.map(|n| TSType::StringLiteral(n.into()))),
                    ],
                );
                base.insert(def.name.name, obj_type);
            }
        }
        base
    }
}
