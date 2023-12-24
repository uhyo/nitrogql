#![cfg(test)]

use graphql_builtins::generate_builtins;
use nitrogql_ast::TypeSystemDocument;
use nitrogql_parser::parse_type_system_document;
use nitrogql_semantics::resolve_schema_extensions;

use crate::{ModelPlugin, Plugin, PluginHost};

mod checker {
    use insta::assert_debug_snapshot;
    use nitrogql_ast::TypeSystemDocument;
    use nitrogql_checker::{check_type_system_document, CheckError, CheckErrorMessage};

    use crate::{ModelPlugin, Plugin};

    use super::parse_to_type_system_document;

    #[test]
    fn model_on_fields() {
        let doc = parse_to_type_system_document(
            "
type User {
    id: ID! @model
    name: String! @model
    age: Int!
    posts: [Post!]!
}

type Post {
    id: ID! @model
    title: String! @model
    body: String!
}
        ",
        );
        let errors = check(&doc);
        assert_debug_snapshot!(errors);
    }

    #[test]
    fn model_on_object_type() {
        let doc = parse_to_type_system_document(
            r#"
type User @model(type: "string") {
    id: ID!
    name: String!
    age: Int!
    posts: [Post!]!
}

type Post @model(type: "import('somewhere').Post") {
    id: ID!
    title: String!
    body: String!
}
            "#,
        );
        let errors = check(&doc);
        assert_debug_snapshot!(errors);
    }

    #[test]
    fn model_without_type_on_object_type() {
        let doc = parse_to_type_system_document(
            r#"
type User @model {
    id: ID!
    name: String!
    age: Int!
    posts: [Post!]!
}

type Post @model {
    id: ID!
    title: String!
    body: String!
}
            "#,
        );
        let errors = check(&doc);
        assert_debug_snapshot!(errors);
    }

    #[test]
    fn model_with_type_on_fields() {
        let doc = parse_to_type_system_document(
            r#"
type User {
    id: ID! @model(type: "string")
    name: String! @model(type: "string")
    age: Int!
}
"#,
        );
        let errors = check(&doc);
        assert_debug_snapshot!(errors);
    }

    #[test]
    fn cannot_use_both() {
        let doc = parse_to_type_system_document(
            r#"
type User @model(type: "string") {
    id: ID! @model
    name: String! @model
    age: Int!
}
"#,
        );
        let errors = check(&doc);
        assert_debug_snapshot!(errors);
    }

    fn check(doc: &TypeSystemDocument) -> Vec<CheckError> {
        let model_plugin = Plugin::new(Box::new(ModelPlugin {}));
        let mut result = check_type_system_document(doc);
        result.extend(
            model_plugin
                .check_schema(doc)
                .errors
                .into_iter()
                .map(|error| CheckError {
                    position: error.position,
                    message: CheckErrorMessage::Plugin {
                        message: error.message,
                    },
                    additional_info: error
                        .additional_info
                        .into_iter()
                        .map(|(pos, message)| (pos, CheckErrorMessage::Plugin { message }))
                        .collect(),
                }),
        );
        result
    }
}

mod resolvers {
    use insta::assert_snapshot;
    use nitrogql_ast::TypeSystemDocument;
    use nitrogql_printer::{ResolverTypePrinter, ResolverTypePrinterOptions};
    use sourcemap_writer::JustWriter;

    use crate::{ModelPlugin, Plugin};

    use super::parse_to_type_system_document;

    #[test]
    fn model_on_fields() {
        let doc = parse_to_type_system_document(
            "
type User {
    id: ID! @model
    name: String! @model
    age: Int!
    posts: [Post!]!
}

type Post {
    id: ID! @model
    title: String! @model
    body: String!
}
        ",
        );
        let doc = print_document(&doc, ResolverTypePrinterOptions::default());
        assert_snapshot!(doc);
    }

    fn print_document(
        document: &TypeSystemDocument,
        options: ResolverTypePrinterOptions,
    ) -> String {
        let plugins = [Plugin::new(Box::new(ModelPlugin {}))];
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        let mut printer = ResolverTypePrinter::new(options, &mut writer);
        printer.print_document(document, &plugins).unwrap();
        result
    }
}

fn parse_to_type_system_document(source: &str) -> TypeSystemDocument {
    let mut doc = parse_type_system_document(source).unwrap();
    doc.extend(generate_builtins());

    let model_plugin = Plugin::new(Box::new(ModelPlugin {}));
    let mut host = TestHost {};
    doc.extend(
        model_plugin
            .schema_addition(&mut host)
            .unwrap()
            .into_iter()
            .flat_map(|d| d.definitions),
    );

    let doc = resolve_schema_extensions(doc).unwrap();
    doc
}

struct TestHost {}

impl PluginHost for TestHost {
    fn load_virtual_file(&mut self, content: String) -> &'static str {
        Box::leak(content.into_boxed_str())
    }
}
