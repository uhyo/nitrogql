#![cfg(test)]

use graphql_builtins::generate_builtins;
use nitrogql_ast::TypeSystemDocument;
use nitrogql_parser::parse_type_system_document;
use nitrogql_semantics::resolve_extensions;

use crate::{ModelPlugin, Plugin, PluginHost};

mod checker {
    use insta::assert_debug_snapshot;
    use nitrogql_checker::check_type_system_document;

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
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors);
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

    let doc = resolve_extensions(doc).unwrap();
    doc
}

struct TestHost {}

impl PluginHost<'static> for TestHost {
    fn load_virtual_file(&mut self, content: String) -> &'static str {
        Box::leak(content.into_boxed_str())
    }
}
