#![cfg(test)]

use graphql_builtins::generate_builtins;
use nitrogql_ast::{TypeSystemDocument, type_system::TypeSystemDefinitionOrExtension};
use nitrogql_parser::parse_type_system_document;
use nitrogql_semantics::resolve_schema_extensions;

use crate::{ModelPlugin, Plugin, PluginHost};

mod checker {
    use insta::assert_debug_snapshot;
    use nitrogql_ast::TypeSystemDocument;
    use nitrogql_checker::{CheckError, CheckErrorMessage, check_type_system_document};

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

    #[test]
    fn model_fields_of_object_type() {
        let doc = parse_to_type_system_document(
            r#"
type Comment {
    id: ID! @model
    "The body of the comment."
    body: String! @model
    author: User!
}

type User @model(type: "import('somewhere').User") {
    id: ID!
}

type PageInfo {
    hasNextPage: Boolean! @model
    endCursor: String @model
}

union SearchResult = Comment | User

type CommentConnection {
    nodes: [Comment!]! @model
    pageInfo: PageInfo! @model
    results: [SearchResult] @model
    totalCount: Int!
}
        "#,
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

fn parse_to_type_system_document(source: &str) -> TypeSystemDocument<'_> {
    let mut doc = parse_type_system_document(source, 0).unwrap();
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

    resolve_schema_extensions(doc).unwrap()
}

#[test]
fn schema_addition_positions_point_to_virtual_file() {
    let plugin = Plugin::new(Box::new(ModelPlugin {}));
    let mut host = TestHost {};
    let addition = plugin.schema_addition(&mut host).unwrap().unwrap();
    let TypeSystemDefinitionOrExtension::DirectiveDefinition(def) = &addition.definitions[0] else {
        panic!("expected a directive definition");
    };
    // TestHost assigns index 1 to the virtual file.
    assert_eq!(def.name.position.file, 1);
}

struct TestHost {}

impl PluginHost<'static> for TestHost {
    fn load_virtual_file(&mut self, content: String) -> (&'static str, usize) {
        (Box::leak(content.into_boxed_str()), 1)
    }
}
