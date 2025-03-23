use std::collections::HashMap;

use nitrogql_ast::{
    TypeSystemDocument,
    type_system::{FieldDefinition, ObjectTypeDefinition, TypeDefinition, TypeSystemDefinition},
    value::Value,
};
use nitrogql_config_file::TypeTarget;
use nitrogql_printer::{
    ResolverTypePrinterOptions,
    ts_types::{TSType, ts_types_util::ts_union},
};

use crate::{
    PluginCheckError,
    plugin_v1::{PluginCheckResult, PluginV1Beta},
};

mod tests;

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
directive @model(
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

                    let has_object_model_directive = model_directive.is_some();

                    for field in def.fields.iter() {
                        let model_directive = field
                            .directives
                            .iter()
                            .find(|directive| directive.name.name == "model");

                        if let Some(directive) = model_directive {
                            if has_object_model_directive {
                                errors.push(PluginCheckError {
                                    position: directive.position,
                                    message: "model directive cannot be used on fields if it is already used on the object".into(),
                                    additional_info: vec![],
                                });
                            }
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
        options: &ResolverTypePrinterOptions,
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
                let obj_type = TSType::TypeFunc(
                    Box::new(TSType::TypeVariable("Pick".into())),
                    vec![
                        TSType::NamespaceMember3(
                            options.schema_root_namespace.clone(),
                            TypeTarget::ResolverOutput.to_string(),
                            def.name.name.into(),
                        ),
                        ts_union(model_field_names.map(|n| TSType::StringLiteral(n.into()))),
                    ],
                );
                base.insert(def.name.name, obj_type);
            }
        }
        base
    }
    fn transform_document_for_resolvers<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
    ) -> Option<TypeSystemDocument<'src>> {
        let definitions = document.definitions.iter().map(|def| {
            if let TypeSystemDefinition::TypeDefinition(TypeDefinition::Object(def)) = def {
                let model_directive = def
                    .directives
                    .iter()
                    .find(|directive| directive.name.name == "model");
                if model_directive.is_some() {
                    // If whole object is @model-ed, then you need to define
                    // resolvers for all fields.
                    return TypeSystemDefinition::TypeDefinition(TypeDefinition::Object(
                        def.clone(),
                    ));
                }

                let fields = def.fields.iter().filter_map(|field| {
                    let model_directive = field
                        .directives
                        .iter()
                        .find(|directive| directive.name.name == "model");
                    if model_directive.is_none() {
                        return Some(field.clone());
                    }
                    None
                });
                TypeSystemDefinition::TypeDefinition(TypeDefinition::Object(ObjectTypeDefinition {
                    fields: fields.collect(),
                    ..def.clone()
                }))
            } else {
                def.clone()
            }
        });

        Some(TypeSystemDocument {
            definitions: definitions.collect(),
        })
    }

    fn transform_document_for_runtime_server<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
    ) -> Option<TypeSystemDocument<'src>> {
        // removes @model directives
        let definitions = document.definitions.iter().flat_map(|def| {
            if let TypeSystemDefinition::DirectiveDefinition(def) = def {
                if def.name.name == "model" {
                    return None;
                }
            }
            if let TypeSystemDefinition::TypeDefinition(TypeDefinition::Object(def)) = def {
                let directives = def
                    .directives
                    .iter()
                    .filter(|directive| directive.name.name != "model")
                    .cloned()
                    .collect();

                let fields = def
                    .fields
                    .iter()
                    .map(|field| {
                        let directives = field
                            .directives
                            .iter()
                            .filter(|directive| directive.name.name != "model")
                            .cloned()
                            .collect();
                        FieldDefinition {
                            directives,
                            ..field.clone()
                        }
                    })
                    .collect();

                Some(TypeSystemDefinition::TypeDefinition(
                    TypeDefinition::Object(ObjectTypeDefinition {
                        directives,
                        fields,
                        ..def.clone()
                    }),
                ))
            } else {
                Some(def.clone())
            }
        });

        Some(TypeSystemDocument {
            definitions: definitions.collect(),
        })
    }
}
